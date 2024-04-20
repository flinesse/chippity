use std::{
    fmt::Write as _,
    io::{stdout, Read, Stdout, Write},
    time::Instant,
};

use bitvec::{bitarr, slice::BitSlice, BitArr};

use crate::{
    chip8::{DISPLAY_HEIGHT, DISPLAY_WIDTH, NUM_KEYS},
    driver::{
        AudioDevice, AudioInfo, DisplayDevice, DisplayInfo, InputDevice, InputInfo, InputMsg,
        KEY_DOWN, KEY_UP, PX_OFF, PX_ON,
    },
    emulator::Signal,
};

const DEBOUNCE_TIMEOUT: u32 = 100; // ms

pub struct Termion {
    // Input byte stream from tty stdin
    stdin: termion::AsyncReader,
    // TUI window - redirects all writes to an alternate screen and restores
    // existing terminal state upon being dropped. Raw mode is required because
    // in canonical mode, inputs are buffered until a newline or EOF is reached.
    // This means that users would have to manually hit return/enter for their
    // inputs to be received by the reader, which is not practical.
    //   - https://en.wikipedia.org/wiki/Terminal_mode
    //   - https://stackoverflow.com/questions/77397499
    screen: termion::screen::AlternateScreen<termion::raw::RawTerminal<Stdout>>,
    // Terminal width and height used to detect resizes and center accordingly
    term_size: (u16, u16), // (w, h)
    // Frame buffer used to write to screen. This is embedded within the struct
    // instead of created at each frame refresh because we get to reuse the
    // space allocated (which is roughly constant) with String::clear()
    framebuf: String,
    // Tx input buffer
    keybuf: BitArr!(for NUM_KEYS),
    // Since inputs come as a byte stream, we don't have convenient key up/down
    // states to relay; having corresponding timers to "expire" key presses will
    // serve that purpose and make inputs more predictable
    key_expire: [Instant; NUM_KEYS],
}

impl Termion {
    pub fn new() -> Self {
        use termion::raw::IntoRawMode;
        use termion::screen::IntoAlternateScreen;

        let mut t = Termion {
            stdin: termion::async_stdin(),
            screen: stdout()
                .into_raw_mode()
                .unwrap()
                .into_alternate_screen()
                .expect("TUI screen creation failed"),
            term_size: termion::terminal_size().unwrap(),
            framebuf: String::new(),
            keybuf: bitarr![0; NUM_KEYS],
            key_expire: [Instant::now(); NUM_KEYS],
        };

        write!(t.screen, "{}", termion::cursor::Hide).unwrap();
        t.screen.flush().unwrap();

        t
    }

    #[inline]
    fn expire_key_presses(&mut self) {
        for (mut key, timer) in self
            .keybuf
            .iter_mut()
            .zip(self.key_expire.iter_mut())
            .filter(|(key, timer)| {
                *key == KEY_DOWN && timer.elapsed().as_millis() >= DEBOUNCE_TIMEOUT as u128
            })
        {
            *key = KEY_UP;
            *timer = Instant::now();
        }
    }

    #[inline]
    fn set_and_time_key(&mut self, idx: usize) {
        self.keybuf.set(idx, KEY_DOWN);
        self.key_expire[idx] = Instant::now();
    }
}

impl InputDevice for Termion {
    //
    //    Keyboard                   CHIP-8
    //    +---+---+---+---+          +---+---+---+---+
    //    | 1 | 2 | 3 | 4 |          | 1 | 2 | 3 | C |
    //    +---+---+---+---+          +---+---+---+---+
    //    | Q | W | E | R |          | 4 | 5 | 6 | D |
    //    +---+---+---+---+    =>    +---+---+---+---+
    //    | A | S | D | F |          | 7 | 8 | 9 | E |
    //    +---+---+---+---+          +---+---+---+---+
    //    | Z | X | C | V |          | A | 0 | B | F |
    //    +---+---+---+---+          +---+---+---+---+
    //
    fn handle_inputs(&mut self) -> Signal {
        let prev_state = self.keybuf;
        self.expire_key_presses();

        let mut inputs = Vec::new();
        // Drain all inputs from stdin
        self.stdin.read_to_end(&mut inputs).unwrap();
        inputs.dedup();

        for byte in inputs {
            match byte {
                b'1' => self.set_and_time_key(0x1),
                b'2' => self.set_and_time_key(0x2),
                b'3' => self.set_and_time_key(0x3),
                b'4' => self.set_and_time_key(0xC),
                b'q' => self.set_and_time_key(0x4),
                b'w' => self.set_and_time_key(0x5),
                b'e' => self.set_and_time_key(0x6),
                b'r' => self.set_and_time_key(0xD),
                b'a' => self.set_and_time_key(0x7),
                b's' => self.set_and_time_key(0x8),
                b'd' => self.set_and_time_key(0x9),
                b'f' => self.set_and_time_key(0xE),
                b'z' => self.set_and_time_key(0xA),
                b'x' => self.set_and_time_key(0x0),
                b'c' => self.set_and_time_key(0xB),
                b'v' => self.set_and_time_key(0xF),
                // Esc (ASCII 0x1B) and ^C (ASCII 0x03) to signal program exit
                0x03 | 0x1B => {
                    write!(self.screen, "{}", termion::cursor::Show).unwrap();
                    return Signal::ProgramExit;
                }
                _ => (),
            }
        }

        if self.keybuf != prev_state {
            Signal::NewInputs
        } else {
            Signal::None
        }
    }

    fn send_inputs(&self) -> Option<InputMsg> {
        Some(self.keybuf)
    }

    fn device_info(&self) -> InputInfo {
        InputInfo::Termion
    }
}

impl DisplayDevice for Termion {
    fn receive_frame(&mut self, frame: &BitSlice<usize>) -> &mut dyn DisplayDevice {
        use termion::color;
        // Clear screen before sending next frame if terminal has resized
        // TODO: if-let chains (https://github.com/rust-lang/rust/issues/53667)
        if let Ok(term_size) = termion::terminal_size() {
            if self.term_size != term_size {
                self.term_size = term_size;
                write!(self.screen, "{}", termion::clear::All).unwrap();
            }
        }

        let (x_offset, y_offset) = (
            self.term_size.0.saturating_sub(DISPLAY_WIDTH as u16) / 2,
            self.term_size.1.saturating_sub(DISPLAY_HEIGHT as u16) / 2,
        );

        self.framebuf.clear();

        for (idx, pixel) in frame.iter().enumerate() {
            // TODO: dynamic scaling with self.term_size?
            if idx % DISPLAY_WIDTH == 0 {
                write!(
                    self.framebuf,
                    "{}",
                    termion::cursor::Goto(
                        x_offset + 1,
                        y_offset + (1 + idx / DISPLAY_WIDTH) as u16
                    )
                )
                .unwrap();
            }
            // https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
            match *pixel {
                PX_OFF => {
                    self.framebuf += &format!("{}█", color::Fg(color::Black));
                }
                PX_ON => {
                    self.framebuf += &format!("{}█", color::Fg(color::White));
                }
            }
        }

        self
    }

    fn drive_display(&mut self) {
        write!(self.screen, "{}", self.framebuf).unwrap();
    }

    fn device_info(&self) -> DisplayInfo {
        DisplayInfo::Termion
    }
}

impl AudioDevice for Termion {
    fn receive_signal(&mut self, data: bool) -> &mut dyn AudioDevice {
        if data {
            write!(self.screen, "\x07").unwrap();
        }

        self
    }

    fn play_audio(&mut self) {}

    fn device_info(&self) -> AudioInfo {
        AudioInfo::Termion
    }
}

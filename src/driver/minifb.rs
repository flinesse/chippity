use bitvec::{bitarr, slice::BitSlice, BitArr};

use crate::{
    chip8::{DISPLAY_HEIGHT, DISPLAY_WIDTH, NUM_KEYS},
    driver::{DisplayDevice, DisplayInfo, InputDevice, InputInfo, InputMsg},
    driver::{KEY_DOWN, KEY_UP, PX_OFF, PX_ON},
    emulator::Signal,
};

// minifb::Window pixels use ARGB encoding;
// alpha-channel (MSB) is ignored => 0RGB
const PX_OFF_COLOR: u32 = 0x1E1C2D;
const PX_ON_COLOR: u32 = 0xE0DEF4;

pub struct Minifb {
    // GUI window
    window: minifb::Window,
    // Auxiliary frame buffer to convert pixels to 32-bit format expected by minifb::Window
    framebuf: [u32; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    // Tx input buffer
    keybuf: BitArr!(for NUM_KEYS),
}

impl Minifb {
    pub fn new(name: &str) -> Self {
        Minifb {
            window: minifb::Window::new(
                &("CHIP-8: ".to_owned() + name),
                DISPLAY_WIDTH,
                DISPLAY_HEIGHT,
                minifb::WindowOptions {
                    resize: true,
                    scale: minifb::Scale::X16,
                    ..Default::default()
                },
            )
            .expect("GUI window creation failed"),

            framebuf: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            keybuf: bitarr![0; NUM_KEYS],
        }
    }
}

impl InputDevice for Minifb {
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
        self.keybuf.fill(KEY_UP);

        if !self.window.is_open() {
            return Signal::ProgramExit;
        }

        self.window.get_keys().iter().for_each(|key| match key {
            minifb::Key::Key1 => self.keybuf.set(0x1, KEY_DOWN),
            minifb::Key::Key2 => self.keybuf.set(0x2, KEY_DOWN),
            minifb::Key::Key3 => self.keybuf.set(0x3, KEY_DOWN),
            minifb::Key::Key4 => self.keybuf.set(0xC, KEY_DOWN),
            minifb::Key::Q => self.keybuf.set(0x4, KEY_DOWN),
            minifb::Key::W => self.keybuf.set(0x5, KEY_DOWN),
            minifb::Key::E => self.keybuf.set(0x6, KEY_DOWN),
            minifb::Key::R => self.keybuf.set(0xD, KEY_DOWN),
            minifb::Key::A => self.keybuf.set(0x7, KEY_DOWN),
            minifb::Key::S => self.keybuf.set(0x8, KEY_DOWN),
            minifb::Key::D => self.keybuf.set(0x9, KEY_DOWN),
            minifb::Key::F => self.keybuf.set(0xE, KEY_DOWN),
            minifb::Key::Z => self.keybuf.set(0xA, KEY_DOWN),
            minifb::Key::X => self.keybuf.set(0x0, KEY_DOWN),
            minifb::Key::C => self.keybuf.set(0xB, KEY_DOWN),
            minifb::Key::V => self.keybuf.set(0xF, KEY_DOWN),
            _ => (),
        });

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
        InputInfo::Minifb
    }
}

impl DisplayDevice for Minifb {
    fn receive_frame(&mut self, frame: &BitSlice<usize>) -> &mut dyn DisplayDevice {
        frame
            .iter()
            .enumerate()
            .for_each(|(idx, pixel)| match *pixel {
                PX_OFF => self.framebuf[idx] = PX_OFF_COLOR,
                PX_ON => self.framebuf[idx] = PX_ON_COLOR,
            });

        self
    }

    fn drive_display(&mut self) {
        self.window
            .update_with_buffer(&self.framebuf, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .unwrap();
    }

    fn device_info(&self) -> DisplayInfo {
        DisplayInfo::Minifb
    }
}

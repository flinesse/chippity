use std::cell::RefCell;
use std::{
    fs, thread,
    time::{Duration, Instant},
};

use crate::{
    chip8,
    chip8::Chip8,
    driver::{AudioDevice, DisplayDevice, InputDevice},
};

// Designs for controlling the flow of I/O can vary greatly in both layout
// and complexity depending on the environment. For our purposes, the emulator
// will act as a simple messaging interface between the guest system and
// connected peripheral devices while serving the host system loop.
//
// For more info:
//   - https://en.wikipedia.org/wiki/Emulator#Input/output_(I/O)
//   - https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O
//   - https://en.wikipedia.org/wiki/Autonomous_peripheral_operation
//
// A CHIP-8 emulator
pub struct Emulator<'a> {
    // The (guest) system being emulated
    system: Chip8,
    // Base clock speed of the emulator; this sets an upper bound on how fast our guest system runs
    clock_rate: f32,
    // --- Peripherals ---
    input: &'a RefCell<dyn InputDevice>,
    display: &'a RefCell<dyn DisplayDevice>,
    audio: &'a RefCell<dyn AudioDevice>,
}

const DEFAULT_CLOCK_FREQ: f32 = 960.0; // TEST: Tune this

// Emulator I/O signals; this is equivalent to ret codes / interrupts in lower level systems
// TODO: Could map subcomponent panics to this for better error handling
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Signal {
    None, // No new events
    ProgramExit,
    NewInputs,
    RefreshDisplay,
    SoundAudio,
}

impl<'a> Emulator<'a> {
    pub fn with_peripherals<'p: 'a>(
        input: &'p RefCell<dyn InputDevice>,
        display: &'p RefCell<dyn DisplayDevice>,
        audio: &'p RefCell<dyn AudioDevice>,
    ) -> Emulator<'a> {
        Emulator {
            system: Chip8::new(),
            clock_rate: DEFAULT_CLOCK_FREQ,
            input,
            display,
            audio,
        }
    }

    pub fn set_clock_speed(&mut self, freq: f32) {
        self.clock_rate = freq;
    }

    pub fn load_program(&mut self, filepath: &str) {
        self.system
            .load_rom(&fs::read(filepath).expect("Failed to read ROM file"));
    }

    // Run the emulator (single-threaded)
    pub fn run(&mut self) {
        // Emulator clock cycle duration
        let t_c = Duration::from_secs_f32(1.0 / self.clock_rate);
        // CHIP-8 timer cycle duration - 60Hz ~= 16ms
        let t_c8timer = Duration::from_secs_f32(1.0 / chip8::TIMER_FREQ as f32);
        // Whether or not to tick CHIP-8 timer
        let mut tick_next = false;

        // Master clock - this helps decouple all other frequency specifications from the main processing frequency
        let master = Instant::now();

        loop {
            ////// CYCLE START //////
            let start = Instant::now();
            let mut event = Signal::None;

            // --- CHIP-8 timers
            // Split current time into 60Hz to ms ~= 16 discrete possibilities and decrement
            // CHIP-8's timer if the elapsed time is a multiple of 16. `tick_next` is needed
            // because we don't want to tick the timer more than once in the span of that discrete
            // millisecond--the emulator could run for multiple cycles during that time
            // TODO: Use a timer crate?
            match (start - master).as_millis() % t_c8timer.as_millis() {
                0 => {
                    if tick_next {
                        event = self.system.tick_timers();
                        tick_next = false;
                    }
                }
                // 1 ms window to pull this flag back up should be sufficient
                // TEST: Is this better than just `_ => tick_next = true`?
                1 => tick_next = true,
                _ => (),
            }

            // --- Handle Audio
            if event == Signal::SoundAudio {
                self.audio
                    .borrow_mut()
                    .receive_signal(self.system.transmit_audio())
                    .play_sound();
            }

            // --- CHIP-8 instruction cycle
            event = self
                .system
                .exec_instruction(self.system.fetch_instruction());

            // --- Handle Display
            if event == Signal::RefreshDisplay {
                self.display
                    .borrow_mut()
                    .receive_frame(self.system.transmit_frame())
                    .drive_display();
            }

            // --- Handle Inputs
            event = self.input.borrow_mut().handle_inputs();

            match event {
                Signal::NewInputs => self.system.receive_input(self.input.borrow().send_inputs()),
                Signal::ProgramExit => break,
                _ => (),
            }

            let cycle_elapsed = start.elapsed();
            ////// CYCLE END //////

            // --- Emulator clock speed
            // Burn remaining cycle to fulfill clock speed requirement
            if let Some(rem) = t_c.checked_sub(cycle_elapsed) {
                thread::sleep(rem);
            }
        }
    }
}

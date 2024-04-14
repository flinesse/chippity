use std::cell::RefCell;
use std::{
    fs, process, thread,
    time::{Duration, Instant},
};

use crate::{
    chip8::{Chip8, TIMER_FREQ},
    driver::{AudioDevice, DisplayDevice, InputDevice},
};

// Designs for controlling the flow of I/O can vary greatly in both layout
// and complexity depending on the environment. For our purposes, the emulator
// will act as a simple messaging interface between the guest system and
// connected peripheral devices. while serving the host system loop
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
    clock_rate: f64,
    // --- Peripherals ---
    input: &'a RefCell<dyn InputDevice>,
    display: &'a RefCell<dyn DisplayDevice>,
    audio: &'a RefCell<dyn AudioDevice>,
}

const DEFAULT_CLOCK_FREQ: f64 = 960.0; // TODO: Tune this

// Emulator I/O signals; this is equivalent to ret codes / interrupts in lower level systems
// TODO: Could map subcomponent panics to this for better error handling
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

    pub fn set_clock_speed(&mut self, freq: f64) {
        self.clock_rate = freq;
    }

    pub fn load_program(&mut self, filepath: &str) {
        self.system
            .load_rom(&fs::read(filepath).expect("Failed to read ROM file"));
    }

    // Run the emulator (single-threaded)
    pub fn run(&mut self) {
        loop {
            ////// CYCLE START //////
            let start = Instant::now();

            let elapsed = start.elapsed();
            ////// CYCLE END //////

            // Burn remaining cycle to fulfill clock speed requirement
            if let Some(rem) = Duration::from_secs_f64(1.0 / self.clock_rate).checked_sub(elapsed) {
                thread::sleep(rem);
            }
        }
    }
}

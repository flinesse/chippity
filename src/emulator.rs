use crate::{
    chip8::Chip8,
    frontend::{Audio, Display, Input},
};

const DEFAULT_CLOCK_FREQ: f64 = 960.0; // TODO

// A CHIP-8 emulator
pub struct Emulator<'a> {
    // The (guest) system being emulated
    system: Chip8,
    // base clock speed the emulator
    clock_rate: f64,
    // --- Peripherals ---
    input: Input<'a>,
    display: Display<'a>,
    audio: Audio<'a>,
}

impl<'a> Emulator<'a> {
    // pub fn new() -> Emulator {
    //     Emulator::default()
    // }

    pub fn with_peripherals<'b: 'a>(
        input: Input<'b>,
        display: Display<'b>,
        audio: Audio<'b>,
    ) -> Emulator<'a> {
        Emulator {
            system: Chip8::new(),
            clock_rate: DEFAULT_CLOCK_FREQ,
            input,
            display,
            audio,
        }
    }
}

use crate::chip8::Chip8;
use crate::driver::{AudioDevice, DisplayDevice, InputDevice, NullDevice};

const NULL_INPUT: NullDevice = NullDevice::Input;
const NULL_DISPLAY: NullDevice = NullDevice::Display;
const NULL_SPEAKER: NullDevice = NullDevice::Audio;

pub struct Emulator<'i, 'd, 'a> {
    // The system we're emulating -- CHIP-8
    system: Chip8,

    input_device: &'i dyn InputDevice,
    display: &'d dyn DisplayDevice,
    speaker: &'a dyn AudioDevice,
}

impl<'i, 'd, 'a> Emulator<'i, 'd, 'a> {
    pub fn new() -> Emulator<'i, 'd, 'a> {
        Emulator::default()
    }

    pub fn with_peripherals(
        input: &'i dyn InputDevice,
        display: &'d dyn DisplayDevice,
        audio: &'a dyn AudioDevice,
    ) -> Emulator<'i, 'd, 'a> {
        Emulator {
            system: Chip8::new(),
            input_device: input,
            display: display,
            speaker: audio,
        }
    }
}

impl<'i, 'd, 'a> Default for Emulator<'i, 'd, 'a> {
    fn default() -> Emulator<'i, 'd, 'a> {
        Emulator {
            system: Chip8::new(),
            input_device: &NULL_INPUT,
            display: &NULL_DISPLAY,
            speaker: &NULL_SPEAKER,
        }
    }
}

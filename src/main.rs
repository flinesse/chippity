mod chip8;
mod instruction;
mod io_device;

use chip8::Chip8;
use io_device::{AudioDevice, DisplayDevice, InputDevice, NullDevice};

static NULL_HID: NullDevice = NullDevice::Input;
static NULL_DISPLAY: NullDevice = NullDevice::Display;
static NULL_SPEAKER: NullDevice = NullDevice::Audio;

pub struct Emulator<'i, 'd, 'a> {
    // The system we're emulating -- CHIP-8
    system: Chip8,

    hid: &'i dyn InputDevice,
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
            hid: input,
            display: display,
            speaker: audio,
        }
    }
}

impl<'i, 'd, 'a> Default for Emulator<'i, 'd, 'a> {
    fn default() -> Emulator<'i, 'd, 'a> {
        Emulator {
            system: Chip8::new(),
            hid: &NULL_HID,
            display: &NULL_DISPLAY,
            speaker: &NULL_SPEAKER,
        }
    }
}

fn main() {
    // TODO: Handle command line args

    // TODO: Setup frontend

    // TODO: Instantiate CHIP-8

    // TODO: Execute game loop
    todo!();
}

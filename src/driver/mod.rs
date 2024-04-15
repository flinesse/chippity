pub mod minifb;

use bitvec::{slice::BitSlice, BitArr};

use crate::chip8::NUM_KEYS;
use crate::emulator::Signal;

// A 16-bit CHIP-8 input message representing the incoming, updated key states
// where the nth bit corresponds to the (n as hex) key status
//
//   Example: 0b1000_0001_0000_1101
//         => keys 0, 1, 3, 8, and F are in the down state
//            and all other keys in the up state
//
pub type InputMsg = BitArr!(for NUM_KEYS);

pub const KEY_UP: bool = false;
pub const KEY_DOWN: bool = true;

// Model input device (e.g. keypad, keyboard, touchscreen, etc.) interfacing with our CHIP-8 system
pub trait InputDevice {
    fn device_info(&self) -> InputInfo;

    fn handle_inputs(&mut self) -> Signal;

    fn send_inputs(&self) -> Option<InputMsg>;
}

pub const PX_OFF: bool = false;
pub const PX_ON: bool = true;

// Model display device (e.g. UI library window, physical screen, etc.) interfacing with our CHIP-8 system
pub trait DisplayDevice {
    fn device_info(&self) -> DisplayInfo;

    fn receive_frame(&mut self, frame: &BitSlice<usize>) -> &mut dyn DisplayDevice;

    fn drive_display(&mut self);
}

// Model audio device (e.g. audio drivers, beeper, etc.) interfacing with our CHIP-8 system
pub trait AudioDevice {
    fn device_info(&self) -> AudioInfo;

    fn receive_signal(&mut self, data: bool) -> &mut dyn AudioDevice;

    fn play_sound(&mut self);
}

#[derive(Clone, Copy)]
pub enum InputInfo {
    Minifb,
    None,
}

#[derive(Clone, Copy)]
pub enum DisplayInfo {
    Minifb,
    None,
}

#[derive(Clone, Copy)]
pub enum AudioInfo {
    None,
}

// Model empty device -- puts `/dev/null` into perspective
#[derive(Clone, Copy)]
pub enum NullDevice {
    Input,
    Display,
    Audio,
}

impl InputDevice for NullDevice {
    fn device_info(&self) -> InputInfo {
        InputInfo::None
    }
    fn handle_inputs(&mut self) -> Signal {
        Signal::None
    }
    fn send_inputs(&self) -> Option<InputMsg> {
        None
    }
}

impl DisplayDevice for NullDevice {
    fn device_info(&self) -> DisplayInfo {
        DisplayInfo::None
    }
    fn receive_frame(&mut self, _frame: &BitSlice<usize>) -> &mut dyn DisplayDevice {
        self
    }
    fn drive_display(&mut self) {
        eprintln!("Nothing to display to!");
    }
}

impl AudioDevice for NullDevice {
    fn device_info(&self) -> AudioInfo {
        AudioInfo::None
    }
    fn receive_signal(&mut self, _data: bool) -> &mut dyn AudioDevice {
        self
    }
    fn play_sound(&mut self) {
        eprintln!("Nothing to play audio through!");
    }
}

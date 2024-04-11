use modular_bitfield::{
    bitfield,
    specifiers::{B3, B4},
    BitfieldSpecifier,
};

use crate::chip8::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

//    My custom CHIP-8 key input message format:
//
//   <-- msb                                                     lsb -->
//    +-----------------------+-------+-------------------------------+
//    |        unused         | state |            keycode            |
//    |       bits 5-7        | bit 4 |           bits 0-3            |
//    +-----------------------+-------+-------------------------------+
#[bitfield]
#[repr(u8)]
pub struct Input {
    #[skip(setters)]
    pub keycode: B4,
    #[skip(setters)]
    #[bits = 1]
    pub key_state: KeyState,
    #[skip]
    __: B3,
}

#[derive(BitfieldSpecifier)]
pub enum KeyState {
    Up,
    Down,
}

// Model input device (e.g. keypad, keyboard, touchscreen, etc.) interfacing with our CHIP-8 system
pub trait InputDevice {
    fn send_input(&self) -> Option<Input>;
}

// Model display device (e.g. UI library window, physical screen, etc.) interfacing with our CHIP-8 system
pub trait DisplayDevice {
    fn receive_frame(&self, framebuf: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]);

    fn drive_display(&self);
}

// Model audio device (e.g. audio drivers, beeper, etc.) interfacing with our CHIP-8 system
pub trait AudioDevice {
    fn receive_signal(&self, data: bool);

    fn play_sound(&self);
}

// Empty device -- puts `/dev/null` into perspective
pub enum NullDevice {
    Input,
    Display,
    Audio,
}

impl InputDevice for NullDevice {
    fn send_input(&self) -> Option<Input> {
        None
    }
}

impl DisplayDevice for NullDevice {
    fn receive_frame(&self, _framebuf: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {}

    fn drive_display(&self) {
        eprintln!(); // TODO
    }
}

impl AudioDevice for NullDevice {
    fn receive_signal(&self, _data: bool) {}

    fn play_sound(&self) {
        eprintln!(); // TODO
    }
}

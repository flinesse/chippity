use std::cell::RefCell;

use crate::driver::*;

pub struct Input<'i> {
    info: InputInfo,
    src: &'i RefCell<dyn InputDevice>,
}

impl<'i> Input<'i> {
    pub fn init(src: &'i RefCell<dyn InputDevice>) -> Self {
        Input {
            info: src.borrow().device_info(),
            src,
        }
    }
}

pub struct Display<'d> {
    info: DisplayInfo,
    src: &'d RefCell<dyn DisplayDevice>,
}

impl<'d> Display<'d> {
    pub fn init(src: &'d RefCell<dyn DisplayDevice>) -> Self {
        Display {
            info: src.borrow().device_info(),
            src,
        }
    }

    pub fn output(&self) {
        self.src.borrow_mut().drive_display()
    }
}

pub struct Audio<'a> {
    info: AudioInfo,
    src: &'a RefCell<dyn AudioDevice>,
}

impl<'a> Audio<'a> {
    pub fn init(src: &'a RefCell<dyn AudioDevice>) -> Self {
        Audio {
            info: src.borrow().device_info(),
            src,
        }
    }

    pub fn output(&self) {
        self.src.borrow_mut().play_sound()
    }
}

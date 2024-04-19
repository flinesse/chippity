use crate::driver::{AudioDevice, AudioInfo};

pub struct AnsiTerm;

impl AudioDevice for AnsiTerm {
    fn receive_signal(&mut self, data: bool) -> &mut dyn AudioDevice {
        if data {
            println!("\x07");
        }

        self
    }

    fn play_sound(&mut self) {}

    fn device_info(&self) -> AudioInfo {
        AudioInfo::AnsiTerm
    }
}

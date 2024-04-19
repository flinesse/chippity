mod chip8;
mod driver;
mod emulator;

use std::cell::RefCell;

use driver::{minifb::Minifb, NullDevice};
use emulator::*;

fn main() {
    // TODO: Handle command line args

    /*
     *  // CHIP-8 should be able to run with no peripherals hooked up to it!
     *  let f_input = RefCell::new(NullDevice::Input);
     *  let f_display = RefCell::new(NullDevice::Display);
     *  let f_audio = RefCell::new(NullDevice::Audio);
     *  let mut dummy = Emulator::with_peripherals(&f_input, &f_display, &f_audio);
     *  dummy.load_program("roms/retro/INVADERS");
     *  dummy.run();
     */

    // Setup frontend
    let minifb = RefCell::new(Minifb::new("test"));
    let f_audio = RefCell::new(NullDevice::Audio);

    // Instantiate CHIP-8 emulator and execute game loop
    let mut emu = Emulator::with_peripherals(&minifb, &minifb, &f_audio);
    emu.set_clock_speed(8.0);
    emu.load_program("roms/test/3-corax+.ch8");
    emu.run();
}

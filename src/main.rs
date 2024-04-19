mod chip8;
mod driver;

mod emulator;

use std::cell::RefCell;
use std::path::Path;

use driver::{ansiterm::AnsiTerm, minifb::Minifb, termion::Termion};
use emulator::Emulator;

// Command line arguments
struct Args {
    rom: String,
    gui: bool,
    native_audio: bool,
    emu_clock_hz: u32,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let help_msg = "\
USAGE:
    cargo run -- [OPTIONS] [ROM]

ARGS:
    <ROM>    Filepath to the CHIP-8 ROM to be read by the emulator. A list of 
             ROMs released to the public domain can be found at:
                 - https://zophar.net/pdroms/chip8/chip-8-games-pack.html
                 - https://johnearnest.github.io/chip8Archive/?sort=platform

OPTIONS:
    -h, --help          Print this help message.
    -g, --gui           GUI mode — run this program in a native window.
    -t, --tui           TUI mode — run this program in the terminal. (default)
    -a                  Use the native audio host API. You may want to enable
                          this if your terminal emulator does not support the
                          BEL control code. Enabled by default with --gui.
    -f, --freq=NUM      Set the clock rate of the emulator (Hz) to uint NUM
                          in the range 1–2000. (default: 720)

KEYMAP:
    +---+---+---+---+
    | 1 | 2 | 3 | 4 |
    +---+---+---+---+
    | Q | W | E | R |
    +---+---+---+---+
    | A | S | D | F |
    +---+---+---+---+
    | Z | X | C | V |
    +---+---+---+---+    ";

    let mut rom = None;
    let mut gui = false;
    let mut native_audio = false;
    let mut emu_clock_hz = emulator::DEFAULT_CLOCK_FREQ as u32;

    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Short('g') | Long("gui") => {
                gui = true;
                native_audio = true;
            }
            Short('t') | Long("tui") => {
                gui = false;
            }
            Short('a') => {
                native_audio = true;
            }
            Short('f') | Long("freq") => {
                emu_clock_hz = parser.value()?.parse()?;
                if !(1..=2000).contains(&emu_clock_hz) {
                    return Err("out of bounds value for option '--freq'".into());
                }
            }
            Value(path) if rom.is_none() => {
                rom = Some(path.string()?);
            }

            Short('h') | Long("help") => {
                println!("{}", help_msg);
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        rom: rom.ok_or(
            "missing argument <ROM>\n
  Refer to --help for more information",
        )?,
        gui,
        native_audio,
        emu_clock_hz,
    })
}

///
///  CHIP-8 should be able to run with no peripherals hooked up to it!
///
///  ```
///  let f_input = RefCell::new(NullDevice::Input);
///  let f_display = RefCell::new(NullDevice::Display);
///  let f_audio = RefCell::new(NullDevice::Audio);
///
///  let mut dummy = Emulator::with_peripherals(&f_input, &f_display, &f_audio);
///  dummy.load_program("roms/retro/INVADERS");
///  dummy.run();
///  ```
fn main() -> Result<(), lexopt::Error> {
    let args = parse_args()?;
    let program_name = Path::new(&args.rom).file_stem().unwrap();

    // Lazily evaluate our emulator frontend
    let termion = || RefCell::new(Termion::new());
    let minifb = || RefCell::new(Minifb::new(program_name.to_str().unwrap()));
    let ansiterm = RefCell::new(AnsiTerm);

    if args.gui {
        let gui = minifb();
        // TODO: native audio
        let mut emu = Emulator::with_peripherals(&gui, &gui, &ansiterm);
        emu.set_clock_speed(args.emu_clock_hz as f32);
        emu.load_program(&args.rom);
        emu.run();
    } else {
        let tui = termion();
        let mut emu = Emulator::with_peripherals(&tui, &tui, &tui);
        emu.set_clock_speed(args.emu_clock_hz as f32);
        emu.load_program(&args.rom);
        emu.run();
    }

    Ok(())
}

use std::{
    cell::RefCell,
    fs, thread,
    time::{Duration, Instant},
};

use crate::{
    chip8,
    chip8::Chip8,
    driver::{AudioDevice, DisplayDevice, InputDevice},
};

// Designs for controlling the flow of I/O can vary greatly in both layout
// and complexity depending on the environment. For our purposes, the emulator
// will act as a simple messaging interface between the guest system and
// connected peripheral devices while serving the host system loop.
//
// For more info:
//   - https://en.wikipedia.org/wiki/Emulator#Input/output_(I/O)
//   - https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O
//   - https://en.wikipedia.org/wiki/Autonomous_peripheral_operation
//
// A CHIP-8 emulator
pub struct Emulator<'a, I, D, A>
where
    I: InputDevice,
    D: DisplayDevice,
    A: AudioDevice,
{
    // The (guest) system being emulated
    system: Chip8,
    // Base clock speed of the emulator; this sets an upper bound on how fast the guest system runs
    clock_rate: f32,
    // --- Peripherals ---
    input: &'a RefCell<I>,
    display: &'a RefCell<D>,
    audio: &'a RefCell<A>,
}

const DEFAULT_CLOCK_FREQ: f32 = 600.0;

// Emulator I/O signals; this is equivalent to ret codes / interrupts in embedded environments
// TODO: Could map subcomponent panics to this for better error handling
#[derive(PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Signal {
    None, // No new events
    ProgramExit,
    NewInputs,
    RefreshDisplay,
    SoundAudio,
}

impl<'a, I, D, A> Emulator<'a, I, D, A>
where
    I: InputDevice,
    D: DisplayDevice,
    A: AudioDevice,
{
    pub fn with_peripherals<'p: 'a>(
        input: &'p RefCell<I>,
        display: &'p RefCell<D>,
        audio: &'p RefCell<A>,
    ) -> Emulator<'a, I, D, A> {
        Emulator {
            system: Chip8::new(),
            clock_rate: DEFAULT_CLOCK_FREQ,
            input,
            display,
            audio,
        }
    }

    pub fn set_clock_speed(&mut self, freq: f32) {
        self.clock_rate = freq;
    }

    pub fn load_program(&mut self, filepath: &str) {
        self.system
            .load_rom(&fs::read(filepath).expect("Failed to read ROM file"));
    }

    // Run the emulator (single-threaded)
    pub fn run(&mut self) {
        // Emulator clock cycle duration
        let t_c = Duration::from_secs_f32(1.0 / self.clock_rate);
        // CHIP-8 timer cycle duration - 60Hz ~= 16ms
        let t_c8timer = Duration::from_secs_f32(1.0 / chip8::TIMER_FREQ).as_millis();
        // Whether or not to tick CHIP-8 timer
        let mut tick_next = false;

        // Master clock - this helps decouple all other frequency specifications from the primary clock frequency
        let master = Instant::now();

        loop {
            ////// CYCLE START //////
            let start = Instant::now();

            // --- Handle Inputs
            let mut event = self.input.borrow_mut().handle_inputs();

            match event {
                Signal::NewInputs => self.system.receive_input(self.input.borrow().send_inputs()),
                Signal::ProgramExit => break,
                _ => (),
            }

            // --- CHIP-8 instruction cycle
            event = self
                .system
                .exec_instruction(self.system.fetch_instruction());

            // --- Handle Display
            if event == Signal::RefreshDisplay {
                self.display
                    .borrow_mut()
                    .receive_frame(self.system.transmit_frame())
                    .drive_display();
            }

            // --- CHIP-8 timers
            // Split current time into 60Hz to ms ~= 16 discrete possibilities and decrement
            // CHIP-8's timer if the elapsed time is a multiple of 16. `tick_next` is needed
            // because we don't want to tick the timer more than once in the span of that discrete
            // millisecond--the emulator could run for multiple cycles during that time if
            // the set clock rate is high enough.
            // TODO: Use a timer crate?
            match (start - master).as_millis() % t_c8timer {
                0 => {
                    if tick_next {
                        event = self.system.tick_timers();
                        tick_next = false;
                    }
                }
                _ => tick_next = true,
            }

            // --- Handle Audio
            if event == Signal::SoundAudio {
                self.audio
                    .borrow_mut()
                    .receive_signal(self.system.transmit_audio())
                    .play_sound();
            }

            let cycle_elapsed = start.elapsed();
            ////// CYCLE END //////

            // --- Emulator clock speed
            // Burn remaining cycle to fulfill clock speed requirement
            thread::sleep(t_c.saturating_sub(cycle_elapsed));
        }
    }
}

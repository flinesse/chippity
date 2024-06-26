mod instruction;

use bitvec::{bitarr, order::Msb0, slice::BitSlice, view::BitView, BitArr};
use smallvec::SmallVec;

use crate::driver::InputMsg;
use crate::emulator;
use instruction::Instruction;

//    CHIP-8 Virtual Machine memory layout:
//    +-----------------------------------+= 0xFFF (4095) End of CHIP-8 RAM
//    |                                   |
//    |                                   |
//    |                                   |
//    |                                   |
//    |                                   |
//    |           0x200 to 0xFFF          |
//    |        CHIP-8 Program / Data      |
//    |                                   |
//    |                 .                 |
//    /                 .                 /
//    /                 .                 /
//    |                                   |
//    +-----------------------------------+= 0x200 (512) Start of _most_ CHIP-8 programs
//    |           0x000 to 0x1FF          |
//    |        Reserved for CHIP-8        |
//    |            interpreter            |
//    + - - - - - - - - - - - - - - - - - += 0x50 (80)* End of conventional CHIP-8 font set
//    |            0x00 to 0x50           |
//    |          CHIP-8 Font Data         |
//    |             '0' - 'F'             |
//    +-----------------------------------+= 0x000 (0) Start of CHIP-8 RAM
//
//  NOTE:
//    Modern implementations are not restricted around the lower 512 bytes since the
//    interpreter runs outside of CHIP-8's specified 4KiB memory space (i.e., our Rust
//    executable instructions for the interpreter exists outside of Chip8.memory).
//    Instead, it is common to store font data representing the hexadecimal digits there.

const RAM_SIZE: usize = 4096;
const FONT_START: u16 = 0x000; // Starting addr of fonts (== RAM_START)
const ROM_START: u16 = 0x200; // Starting addr of CHIP-8 programs
const ROM_END: u16 = 0xFFF; // Upper bounds addr of CHIP-8 programs (== RAM_SIZE)
const STACK_SIZE: usize = 12;
const NUM_DATA_REGS: usize = 16;
const PC_STEP: u16 = 2; // mem::size_of::<Instruction>() / chip8_addressable_unit = 2

// Pre-defined "static" font data that will occupy memory reserved for the interpreter (<0x200)
const FONT_SPRITES: [[u8; FONT_PX_HEIGHT]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];
const FONT_PX_HEIGHT: usize = 5;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const NUM_KEYS: usize = 16;
pub const TIMER_FREQ: f32 = 60.0;

pub struct Chip8 {
    // RAM of the CHIP-8 VM
    memory: [u8; RAM_SIZE],
    // Program Counter
    pc: u16,
    // CHIP-8 call stack; its only purpose is to push/pop any callers' return address
    //   "The original RCA 1802 version allowed up to 12 levels of
    //   nesting; _modern implementations may wish to allocate more_"
    stack: SmallVec<[u16; STACK_SIZE]>,
    // I - the address register
    i_reg: u16,
    // V - general purpose data registers
    v_reg: [u8; NUM_DATA_REGS],

    //  Output device: 64x32-pixel monochrome display
    //    +--------------------+
    //    |(0, 0)       (63, 0)|
    //    |                    |
    //    |                    |
    //    |(0, 31)     (63, 31)|
    //    +--------------------+
    //  Modeled in 1-D as: 0, 1, 2, ... , w-1
    //                     w, w+1,  ... , 2w-1
    //                     ...      ... , nw-1
    //                     w(h-1),  ... , wh-1
    //          and stored as a 2048-bit array
    display_bus: BitArr!(for DISPLAY_WIDTH * DISPLAY_HEIGHT),

    //  Input device: 16-key keypad (0x0-0xF)
    //    +------------+
    //    | 1  2  3  C |
    //    | 4  5  6  D |
    //    | 7  8  9  E |
    //    | A  0  B  F |
    //    +------------+
    //  Stored as a 16-bit array with the (n as hex)th bit
    //  corresponding to the key state; KEY_UP = 0, KEY_DOWN = 1
    input_bus: BitArr!(for NUM_KEYS),
    // General timer used for game events
    delay_timer: u8,
    // Timer for sound effects; a beep is made when the value is nonzero
    sound_timer: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut sys = Chip8 {
            memory: [0; RAM_SIZE],
            pc: ROM_START,
            stack: SmallVec::new(),
            i_reg: 0,
            v_reg: [0; NUM_DATA_REGS],
            display_bus: bitarr![0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            input_bus: bitarr![0; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };

        sys.load_fonts();
        sys
    }

    fn load_fonts(&mut self) {
        for (i, font) in FONT_SPRITES.iter().flatten().enumerate() {
            self.memory[(FONT_START as usize) + i] = *font;
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let rom_size = data.len();
        if rom_size > (ROM_END - ROM_START) as usize {
            panic!("Insufficient memory: invalid ROM");
        }

        let start = ROM_START as usize;
        let end = (ROM_START as usize) + rom_size;
        self.memory[start..end].copy_from_slice(data);
    }

    pub fn tick_timers(&mut self) -> emulator::Signal {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            emulator::Signal::SoundAudio
        } else {
            emulator::Signal::None
        }
    }

    pub fn fetch_instruction(&self) -> Instruction {
        // Program Counter is monotonically non-decreasing starting at 0x200;
        // it is up to the ROM to ensure that the PC remains within valid bounds
        if self.pc < ROM_START || self.pc >= ROM_END {
            panic!("Segfault: invalid ROM");
        }

        // CHIP-8 instructions are stored big-endian
        let hb = self.memory[self.pc as usize];
        let lb = self.memory[(self.pc + 1) as usize];
        // Instruction (`modular_bitfield::bitfield`) is constructed lsb -> msb
        Instruction::from_bytes([lb, hb])
    }

    pub fn exec_instruction(&mut self, instr: Instruction) -> emulator::Signal {
        // Whether to step the PC at the end of cycle - true; false if any jumps are issued
        let mut incr_pc = true;
        // I/O ret code
        let mut status = emulator::Signal::None;

        /*
        println!(
            "Instruction: {:X}{:X} {:X}{:X} --- PC: {:X}",
            instr.get_o(), instr.get_x(), instr.get_y(), instr.get_n(), self.pc,
        );
        */

        // Decode and excute instruction
        match (instr.get_o(), instr.get_x(), instr.get_y(), instr.get_n()) {
            // 00E0 - CLRS
            (0x0, 0x0, 0xE, 0x0) => {
                self.display_bus.fill(false);

                status = emulator::Signal::RefreshDisplay;
            }
            // 00EE - RET
            (0x0, 0x0, 0xE, 0xE) => {
                let ret_addr = self.stack.pop().expect("Segfault: invalid ROM");
                self.pc = ret_addr;
            }
            // 0NNN - SYSC addr (Ignored by modern interpreters)
            (0x0, _n1, _n2, _n3) => {
                eprintln!(
                    "Encountered unsupported instruction - {:#04X}",
                    u16::from(instr)
                );
            }
            // 1NNN - JMP addr
            (0x1, _n1, _n2, _n3) => {
                let addr = instr.get_nnn();
                self.pc = addr;
                incr_pc = false;
            }
            // 2NNN - CALL addr
            (0x2, _n1, _n2, _n3) => {
                let addr = instr.get_nnn();
                self.stack.push(self.pc);
                self.pc = addr;
                incr_pc = false;
            }
            // 3XNN - SKE Vx, byte
            (0x3, x, _n2, _n3) => {
                if self.v_reg[x as usize] == instr.get_nn() {
                    self.pc += PC_STEP;
                }
            }
            // 4XNN - SKNE Vx, byte
            (0x4, x, _n2, _n3) => {
                if self.v_reg[x as usize] != instr.get_nn() {
                    self.pc += PC_STEP;
                }
            }
            // 5XY0 - SKE Vx, Vy
            (0x5, x, y, 0x0) => {
                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += PC_STEP;
                }
            }
            // 6XNN - LD Vx, byte
            (0x6, x, _n2, _n3) => {
                self.v_reg[x as usize] = instr.get_nn();
            }
            // 7XNN - ADD Vx, byte
            (0x7, x, _n2, _n3) => {
                self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(instr.get_nn());
            }
            // 8XY0 - LD Vx, Vy
            (0x8, x, y, 0x0) => {
                self.v_reg[x as usize] = self.v_reg[y as usize];
            }
            // 8XY1 - OR Vx, Vy
            (0x8, x, y, 0x1) => {
                self.v_reg[x as usize] |= self.v_reg[y as usize];
            }
            // 8XY2 - AND Vx, Vy
            (0x8, x, y, 0x2) => {
                self.v_reg[x as usize] &= self.v_reg[y as usize];
            }
            // 8XY3 - XOR Vx, Vy
            (0x8, x, y, 0x3) => {
                self.v_reg[x as usize] ^= self.v_reg[y as usize];
            }
            // 8XY4 - ADD Vx, Vy; set VF
            (0x8, x, y, 0x4) => {
                let (vx, carry) = self.v_reg[x as usize].overflowing_add(self.v_reg[y as usize]);
                self.v_reg[x as usize] = vx;
                self.v_reg[0xF] = carry as u8;
            }
            // 8XY5 - SUB Vx, Vy; set VF
            (0x8, x, y, 0x5) => {
                let (vx, borrow) = self.v_reg[x as usize].overflowing_sub(self.v_reg[y as usize]);
                self.v_reg[x as usize] = vx;
                self.v_reg[0xF] = !borrow as u8;
            }
            // 8XY6 - SHR Vx {, Vy}; set VF
            //   WARN: There is conflicting info on whether Vx = { Vx >> 1 or Vy >> 1 }
            (0x8, x, _y, 0x6) => {
                let lsb = self.v_reg[x as usize] & 0x1;
                self.v_reg[x as usize] >>= 1;
                self.v_reg[0xF] = lsb;
            }
            // 8XY7 - SUBN Vx, Vy; set VF
            (0x8, x, y, 0x7) => {
                let (vx, borrow) = self.v_reg[y as usize].overflowing_sub(self.v_reg[x as usize]);
                self.v_reg[x as usize] = vx;
                self.v_reg[0xF] = !borrow as u8;
            }
            // 8XYE - SHL Vx {, Vy}; set VF
            //   WARN: There is conflicting info on whether Vx = { Vx << 1 or Vy << 1 }
            (0x8, x, _y, 0xE) => {
                let msb = (self.v_reg[x as usize] >> (u8::BITS - 1)) & 0x1;
                self.v_reg[x as usize] <<= 1;
                self.v_reg[0xF] = msb;
            }
            // 9XY0 - SKNE Vx, Vy
            (0x9, x, y, 0x0) => {
                if self.v_reg[x as usize] != self.v_reg[y as usize] {
                    self.pc += PC_STEP;
                }
            }
            // ANNN - LD I, addr
            (0xA, _n1, _n2, _n3) => {
                let addr = instr.get_nnn();
                self.i_reg = addr;
            }
            // BNNN - JMP V0, addr
            (0xB, _n1, _n2, _n3) => {
                let addr = instr.get_nnn();
                self.pc = addr + (self.v_reg[0x0] as u16);
                incr_pc = false;
            }
            // CXNN - RAND Vx, byte
            (0xC, x, _n2, _n3) => {
                self.v_reg[x as usize] = fastrand::u8(..) & instr.get_nn();
            }
            // DXYN - DRAW Vx, Vy, nibble; set VF
            //   Read an n-byte sprite from memory starting at addr I and display onto coordinates (Vx, Vy)
            //   Detect collision and set VF accordingly; pixels positioned offscreen are wrapped around the display
            (0xD, x, y, n) => {
                let sprite = &self.memory[self.i_reg as usize..(self.i_reg + n as u16) as usize];
                let coord = (self.v_reg[x as usize], self.v_reg[y as usize]);
                self.v_reg[0xF] = 0;

                for (dy, byte) in sprite.iter().enumerate() {
                    let coord_y = (coord.1 as usize + dy) % DISPLAY_HEIGHT;
                    for (dx, bit) in byte.view_bits::<Msb0>().iter().enumerate() {
                        let coord_x = (coord.0 as usize + dx) % DISPLAY_WIDTH;
                        let idx = coord_y * DISPLAY_WIDTH + coord_x;
                        let display_bit = self.display_bus[idx];

                        // Collided if any corresponding sprite and display bits are HIGH (bitwise AND)
                        self.v_reg[0xF] |= (display_bit & *bit) as u8;
                        self.display_bus.set(idx, display_bit ^ *bit);
                    }
                }

                status = emulator::Signal::RefreshDisplay;
            }
            // EX9E - SKP Vx
            (0xE, x, 0x9, 0xE) => {
                let key_down = self.input_bus[self.v_reg[x as usize] as usize];
                if key_down {
                    self.pc += PC_STEP;
                }
            }
            // EXA1 - SKNP Vx
            (0xE, x, 0xA, 0x1) => {
                let key_down = self.input_bus[self.v_reg[x as usize] as usize];
                if !key_down {
                    self.pc += PC_STEP;
                }
            }
            // FX07 - LD Vx, DT
            (0xF, x, 0x0, 0x7) => {
                self.v_reg[x as usize] = self.delay_timer;
            }
            // FX0A - LD Vx, K
            (0xF, x, 0x0, 0xA) => {
                // Randomly select a pressed key instead of one with the lowest index; avoids having
                // a key always taking precedence over another when both are simulatneously pressed
                let rand = fastrand::usize(0..NUM_KEYS);
                if let Some(k_idx) = self
                    .input_bus
                    .iter()
                    .skip(rand)
                    .position(|key_down| *key_down)
                {
                    self.v_reg[x as usize] = (rand + k_idx) as u8;
                } else {
                    // Block execution (no-op and repeat instr next cycle) until input detected
                    incr_pc = false;
                }
            }
            // FX15 - LD DT, Vx
            (0xF, x, 0x1, 0x5) => {
                self.delay_timer = self.v_reg[x as usize];
            }
            // FX18 - LD ST, Vx
            (0xF, x, 0x1, 0x8) => {
                self.sound_timer = self.v_reg[x as usize];
            }
            // FX1E - ADD I, Vx
            (0xF, x, 0x1, 0xE) => {
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x as usize] as u16);
            }
            // FX29 - LEA I, F(Vx)
            (0xF, x, 0x2, 0x9) => {
                // Address for font sprite representing hex digit '{Vx}'
                //             = FONT_START + Vx * bytes_per_font_sprite
                self.i_reg = FONT_START + (self.v_reg[x as usize] as u16) * (FONT_PX_HEIGHT as u16);
            }
            // FX33 - LD [I], D2(Vx)
            //           [I + 1], D1(Vx)
            //           [I + 2], D0(Vx)
            (0xF, x, 0x3, 0x3) => {
                let vx = self.v_reg[x as usize];
                // Extracts the n-th decimal digit (inline? https://godbolt.org/z/scffbPj7s)
                let d = |val, n| val / u8::pow(10, n) % 10;
                self.memory[self.i_reg as usize] = d(vx, 2);
                self.memory[self.i_reg as usize + 1] = d(vx, 1);
                self.memory[self.i_reg as usize + 2] = d(vx, 0);
            }
            // FX55 - LD [I], V0
            //           [I + 1], V1
            //             ...
            //           [I + x], Vx
            //   WARN: There is conflicting info on whether I = {I or I + x + 1}
            (0xF, x, 0x5, 0x5) => {
                for offset in 0..=(x as usize) {
                    self.memory[self.i_reg as usize + offset] = self.v_reg[offset];
                }
            }
            // FX65 - LD Vx, [I]
            //           V1, [I + 1]
            //             ...
            //           Vx, [I + x]
            //   WARN: There is conflicting info on whether I = {I or I + x + 1}
            (0xF, x, 0x6, 0x5) => {
                for offset in 0..=(x as usize) {
                    self.v_reg[offset] = self.memory[self.i_reg as usize + offset];
                }
            }
            (_, _, _, _) => {
                panic!(
                    "Encountered unrecognized instruction - {:#04X}: invalid ROM",
                    u16::from(instr)
                );
            }
        }

        if incr_pc {
            self.pc += PC_STEP;
        }

        status
    }

    // Rx 16-bit input key state
    pub fn receive_input(&mut self, msg: Option<InputMsg>) {
        if let Some(input) = msg {
            self.input_bus = input;
        }
    }

    // Tx 1-bit sound channel
    pub fn transmit_audio(&self) -> bool {
        self.sound_timer > 0
    }

    // Tx 2048 (64x32) bit display out
    pub fn transmit_frame(&self) -> &BitSlice<usize> {
        self.display_bus.as_bitslice()
    }
}

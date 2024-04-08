use crate::instruction::Instruction;

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
const RAM_START: u16 = 0x000; // Starting addr of RAM
const ROM_START: u16 = 0x200; // Starting addr of CHIP-8 programs
const STACK_SIZE: usize = 12;
const NUM_DATA_REGS: usize = 16;
const PC_STEP: u16 = 2; // mem::size_of::<Instruction>() / chip8_addressable_unit = 2

// Pre-defined "static" font data that will occupy memory reserved for the interpreter (<0x200)
const FONT_SPRITES: [[u8; 5]; 16] = [
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

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct Chip8 {
    // RAM of the CHIP-8 VM
    memory: [u8; RAM_SIZE],
    // Program Counter
    pc: u16,
    // CHIP-8 call stack; its only purpose is to push/pop any callers' return address
    stack: Vec<u16>,
    // Stack Pointer
    sp: u16,
    // I - the address register
    i_reg: u16,
    // V - general purpose data registers
    v_reg: [u8; NUM_DATA_REGS],

    //  64x32-pixel monochrome display
    //    +--------------------+
    //    |(0, 0)       (63, 0)|
    //    |                    |
    //    |                    |
    //    |(0, 31)     (63, 31)|
    //    +--------------------+
    //  Modeled as 1-D array: 0, 1, 2, ... , w-1
    //                        w, w+1,  ... , 2w-1
    //                        ...      ... , nw-1
    //                        w(h-1),  ... , wh-1
    display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],

    // Key Input (0x00-0x0F); upper 4 bits are ignored
    key_pressed: Option<u8>,
    // General timer used for game events
    delay_timer: u8,
    // Timer for sound effects; a beep is made when the value is nonzero
    sound_timer: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut emu = Chip8 {
            memory: [0; RAM_SIZE],
            pc: ROM_START,
            // The original RCA 1802 version allowed up to 12 levels of nesting
            // _Modern implementations may wish to allocate more_
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
            i_reg: 0,
            v_reg: [0; NUM_DATA_REGS],
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            key_pressed: None,
            delay_timer: 0,
            sound_timer: 0,
        };

        emu.load_fonts();
        emu
    }

    fn load_fonts(&mut self) {
        for (i, font) in FONT_SPRITES.iter().flatten().enumerate() {
            self.memory[(RAM_START as usize) + i] = *font;
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let start = ROM_START as usize;
        let end = (ROM_START as usize) + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    pub fn fetch_instruction(&mut self) -> Instruction {
        // Program Counter monotonically increases starting at 0x200;
        // it is up to the ROM to ensure that the PC remains within valid bounds
        // if self.pc < ROM_START || self.pc >= (RAM_SIZE as u16) {
        //     panic!("Bad ROM!!")
        // }

        // CHIP-8 instructions are stored big-endian
        let hb = self.memory[self.pc as usize];
        let lb = self.memory[(self.pc + 1) as usize];
        // Instruction (`modular_bitfield::bitfield`) is constructed lsb -> msb
        let instr = Instruction::from_bytes([lb, hb]);
        self.pc += PC_STEP;

        instr
    }

    pub fn exec_instruction(&mut self, instr: Instruction) {
        let (o, x, y, n) = (instr.get_o(), instr.get_x(), instr.get_y(), instr.get_n());

        // Decode and excute instruction
        match (o, x, y, n) {
            // 00E0 - CLRS
            (0x0, 0x0, 0xE, 0x0) => {
                self.display.fill(false);
            }
            // 00EE - RET
            (0x0, 0x0, 0xE, 0xE) => {
                let ret_addr = self.stack.pop().expect(""); // TODO
                self.pc = ret_addr;
            }
            // 0NNN - SYSC addr (Ignored by modern interpreters)
            (0x0, _n1, _n2, _n3) => {
                eprintln!(""); // TODO
            }
            // 1NNN - JMP addr
            (0x1, _n1, _n2, _n3) => {
                let addr = instr.get_nnn();
                self.pc = addr;
            }
            // 2NNN - CALL addr
            (0x2, _n1, _n2, _n3) => {
                let addr = instr.get_nnn();
                self.stack.push(self.pc);
                self.pc = addr;
            }
            // 3XNN - SKE Vx, byte
            (0x3, _x, _n2, _n3) => {
                if self.v_reg[instr.get_x() as usize] == instr.get_nn() {
                    self.pc += PC_STEP;
                }
            }
            // 4XNN - SKNE Vx, byte
            (0x4, _x, _n2, _n3) => {
                if self.v_reg[instr.get_x() as usize] != instr.get_nn() {
                    self.pc += PC_STEP;
                }
            }
            // 5XY0 - SKE Vx, Vy
            (0x5, _x, _y, 0x0) => {
                if self.v_reg[instr.get_x() as usize] == self.v_reg[instr.get_y() as usize] {
                    self.pc += PC_STEP;
                }
            }
            // 6XNN - LD Vx, byte
            (0x6, _x, _n2, _n3) => {
                self.v_reg[instr.get_x() as usize] = instr.get_nn();
            }
            // 7XNN - ADD Vx, byte
            (0x7, _x, _n2, _n3) => {
                self.v_reg[instr.get_x() as usize] += instr.get_nn();
            }
            // 8XY0 - LD Vx, Vy
            (0x8, _x, _y, 0x0) => todo!(),
            // 8XY1 - OR Vx, Vy
            (0x8, _x, _y, 0x1) => todo!(),
            // 8XY2 - AND Vx, Vy
            (0x8, _x, _y, 0x2) => todo!(),
            // 8XY3 - XOR Vx, Vy
            (0x8, _x, _y, 0x3) => todo!(),
            // 8XY4 - ADD Vx, Vy
            (0x8, _x, _y, 0x4) => todo!(),
            // 8XY5 - SUB Vx, Vy
            (0x8, _x, _y, 0x5) => todo!(),
            // 8XY6 - SHR Vx {, Vy}
            (0x8, _x, _y, 0x6) => todo!(),
            // 8XY7 - SUBN Vx, Vy
            (0x8, _x, _y, 0x7) => todo!(),
            // 8XYE - SHL Vx {, Vy}
            (0x8, _x, _y, 0xE) => todo!(),
            // 9XY0 - SNE Vx, Vy
            (0x9, _x, _y, 0x0) => todo!(),
            // ANNN - LD I, addr
            (0xA, _n1, _n2, _n3) => todo!(),
            // BNNN - JP V0, addr
            (0xB, _n1, _n2, _n3) => todo!(),
            // CXNN - RND Vx, byte
            (0xC, _x, _n2, _n3) => todo!(),
            // DXYN - DRW Vx, Vy, nibble
            (0xD, _x, _y, _n3) => todo!(),
            // EX9E - SKP Vx
            (0xE, _x, 0x9, 0xE) => todo!(),
            // EXA1 - SKNP Vx
            (0xE, _x, 0xA, 0x1) => todo!(),
            // FX07 - LD Vx, DT
            (0xF, _x, 0x0, 0x7) => todo!(),
            // FX0A - LD Vx, K
            (0xF, _x, 0x0, 0xA) => todo!(),
            // FX15 - LD DT, Vx
            (0xF, _x, 0x1, 0x5) => todo!(),
            // FX18 - LD ST, Vx
            (0xF, _x, 0x1, 0x8) => todo!(),
            // FX1E - ADD I, Vx
            (0xF, _x, 0x1, 0xE) => todo!(),
            // FX29 - LD F, Vx
            (0xF, _x, 0x2, 0x9) => todo!(),
            // FX33 - LD B, Vx
            (0xF, _x, 0x3, 0x3) => todo!(),
            // FX55 - LD [I], Vx
            (0xF, _x, 0x5, 0x5) => todo!(),
            // FX65 - LD Vx, [I]
            (0xF, _x, 0x6, 0x5) => todo!(),
            (_, _, _, _) => panic!(),
        }
    }
}

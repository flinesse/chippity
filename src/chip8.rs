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
    //   The original RCA 1802 version allowed up to 12 levels of
    //   nesting; modern implementations may wish to allocate more
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

    pub fn run(&self) {
        todo!()
    }
}

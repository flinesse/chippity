use modular_bitfield::{bitfield, specifiers::B4};

//    CHIP-8 Instruction Set format:
//
//   <-- msb                                                     lsb -->
//                    |---    x    ---|---    y    ---|
//    +---------------+---------------+---------------+---------------+
//    |      n0       |      n1       |      n2       |      n3       |
//    |  bits 12-15   |   bits 8-11   |   bits 4-7    |   bits 0-3    |
//    +---------------+---------------+---------------+---------------+
//    |---    o    ---|---                   nnn                   ---|
//                                    |---           nn            ---|
//                                                    |---    n    ---|
//

// Ordering of `bitfield` is from lsb to msb: https://docs.rs/modular-bitfield/latest/modular_bitfield/index.html#example
#[bitfield(bits = 16)]
#[repr(u16)]
pub struct Instruction {
    #[skip(setters)]
    n3: B4,
    #[skip(setters)]
    n2: B4,
    #[skip(setters)]
    n1: B4,
    #[skip(setters)]
    n0: B4,
}

impl Instruction {
    // o - Opcode header; uppermost 4 bits of instruction
    pub fn get_o(&self) -> u8 {
        self.n0()
    }

    // nnn - For memory addressing CHIP-8 VM (4096 = 2^12); lowest 12 bits of instruction
    pub fn get_nnn(&self) -> u16 {
        (self.n1() as u16) << 8 | (self.n2() as u16) << 4 | (self.n3() as u16)
    }

    // nn - Lowest 8 bits of instruction
    pub fn get_nn(&self) -> u8 {
        self.n2() << 4 | self.n3()
    }

    // n - Lowest 4 bits of instruction
    pub fn get_n(&self) -> u8 {
        self.n3()
    }

    // x - Lower 4 bits of the high byte of the instruction
    pub fn get_x(&self) -> u8 {
        self.n1()
    }

    // y - Upper 4 bits of the lower byte of the instruction
    pub fn get_y(&self) -> u8 {
        self.n2()
    }
}

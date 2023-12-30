use kaiseki_core::cpu::{opcode::Opcode16, Instruction};

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Chip8Instruction {
    ClearScreen, // 0x00E0
    Return,      // 0x00EE
    ExecuteMachineSubroutine {
        addr: u16,
    }, // 0x0NNN except 0x00E0 and 0x00EE
    Jump {
        addr: u16,
    }, // 0x1NNN
    Call {
        addr: u16,
    }, // 0x2NNN
    SkipIfEqual {
        vx_idx: u8,
        value: u8,
    }, // 0x3XNN
    SkipIfNotEqual {
        vx_idx: u8,
        value: u8,
    }, // 0x4XNN
    SkipIfRegEqual {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x5XY0
    SetReg {
        vx_idx: u8,
        value: u8,
    }, // 0x6XNN
    AddReg {
        vx_idx: u8,
        value: u8,
    }, // 0x7XNN
    SetRegReg {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XY0
    OrRegReg {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XY1
    AndRegReg {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XY2
    XorRegReg {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XY3
    AddRegReg {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XY4
    SubRegReg {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XY5
    ShiftRightReg {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XY6
    SubRegRegReverse {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XY7
    ShiftLeftReg {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x8XYE
    SkipIfRegNotEqual {
        vx_idx: u8,
        vy_idx: u8,
    }, // 0x9XY0
    SetVI {
        addr: u16,
    }, // 0xANNN
    JumpPlusV0 {
        addr: u16,
    }, // 0xBNNN
    Random {
        vx_idx: u8,
        mask: u8,
    }, // 0xCXNN
    Draw {
        vx_idx: u8,
        vy_idx: u8,
        num_bytes: u8,
    }, // 0xDXYN
    SkipIfKeyPressed {
        vx_idx: u8,
    }, // 0xEX9E
    SkipIfKeyNotPressed {
        vx_idx: u8,
    }, // 0xEXA1
    GetDelayTimer {
        vx_idx: u8,
    }, // 0xFX07
    WaitForKey {
        vx_idx: u8,
    }, // 0xFX0A
    SetDelayTimer {
        vx_idx: u8,
    }, // 0xFX15
    SetSoundTimer {
        vx_idx: u8,
    }, // 0xFX18
    AddRegVI {
        vx_idx: u8,
    }, // 0xFX1E
    SetVIDigit {
        digit: u8,
    }, // 0xFX29
    StoreBCD {
        vx_idx: u8,
    }, // 0xFX33
    StoreRegs {
        vx_idx: u8,
    }, // 0xFX55
    LoadRegs {
        vx_idx: u8,
    }, // 0xFX65
}

impl Instruction for Chip8Instruction {
    type Opcode = Opcode16;

    fn len_bytes(&self) -> usize {
        2
    }
}

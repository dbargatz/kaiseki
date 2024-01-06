use kaiseki_core::cpu::{Instruction, InstructionDefinition, InstructionId, InstructionSet};
use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Chip8InstructionId {
    ClearScreen,              // 0x00E0
    Return,                   // 0x00EE
    ExecuteMachineSubroutine, // 0x0NNN except 0x00E0 and 0x00EE
    Jump,                     // 0x1NNN
    Call,                     // 0x2NNN
    SkipIfEqual,              // 0x3XNN
    SkipIfNotEqual,           // 0x4XNN
    SkipIfRegEqual,           // 0x5XY0
    SetReg,                   // 0x6XNN
    AddReg,                   // 0x7XNN
    SetRegReg,                // 0x8XY0
    OrRegReg,                 // 0x8XY1
    AndRegReg,                // 0x8XY2
    XorRegReg,                // 0x8XY3
    AddRegReg,                // 0x8XY4
    SubRegReg,                // 0x8XY5
    ShiftRightReg,            // 0x8XY6
    SubRegRegReverse,         // 0x8XY7
    ShiftLeftReg,             // 0x8XYE
    SkipIfRegNotEqual,        // 0x9XY0
    SetVI,                    // 0xANNN
    JumpPlusV0,               // 0xBNNN
    Random,                   // 0xCXNN
    Draw,                     // 0xDXYN
    SkipIfKeyPressed,         // 0xEX9E
    SkipIfKeyNotPressed,      // 0xEXA1
    GetDelayTimer,            // 0xFX07
    WaitForKey,               // 0xFX0A
    SetDelayTimer,            // 0xFX15
    SetSoundTimer,            // 0xFX18
    AddRegVI,                 // 0xFX1E
    SetVIDigit,               // 0xFX29
    StoreBCD,                 // 0xFX33
    StoreRegs,                // 0xFX55
    LoadRegs,                 // 0xFX65
}

impl InstructionId for Chip8InstructionId {}

pub struct Chip8InstructionDefinition {
    id: Chip8InstructionId,
    mnemonic: &'static str,
    opcodes: &'static [<Self as InstructionDefinition>::OpcodeValue],
    width_bits: usize,
}

impl InstructionDefinition for Chip8InstructionDefinition {
    type Id = Chip8InstructionId;
    type OpcodeValue = u16;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn mnemonic(&self) -> &'static str {
        self.mnemonic
    }

    fn valid_opcodes(&self) -> &'static [Self::OpcodeValue] {
        self.opcodes
    }

    fn width_bits(&self) -> usize {
        self.width_bits
    }
}

const fn exclusions_contain(value: u16, exclusions: &[u16]) -> bool {
    let mut i = 0;
    while i < exclusions.len() {
        if exclusions[i] == value {
            return true;
        }
        i += 1;
    }
    false
}

const fn gen_opcodes<const N: usize>(start: u16, exclusions: &[u16]) -> [u16; N] {
    let mut opcodes: [u16; N] = [0; N];
    let mut i = 0;
    let mut cur = start;
    while i < N {
        if exclusions_contain(cur, exclusions) {
            cur += 1;
            continue;
        }

        opcodes[i] = cur;
        i += 1;
        cur += 1;
    }
    opcodes
}

pub struct Chip8InstructionSet {}

impl Chip8InstructionSet {
    const CLEAR_SCREEN: Chip8InstructionDefinition = Chip8InstructionDefinition {
        id: Chip8InstructionId::ClearScreen,
        mnemonic: "CLS",
        opcodes: &[0x00E0],
        width_bits: 16,
    };
    const RETURN: Chip8InstructionDefinition = Chip8InstructionDefinition {
        id: Chip8InstructionId::Return,
        mnemonic: "RET",
        opcodes: &[0x00EE],
        width_bits: 16,
    };
    const EXECUTE_MACHINE_SUBROUTINE: Chip8InstructionDefinition = Chip8InstructionDefinition {
        id: Chip8InstructionId::ExecuteMachineSubroutine,
        mnemonic: "SYS",
        opcodes: &gen_opcodes::<0xFFE>(0, &[0x00E0, 0x00EE]),
        width_bits: 16,
    };
    const JUMP: Chip8InstructionDefinition = Chip8InstructionDefinition {
        id: Chip8InstructionId::Jump,
        mnemonic: "JP",
        opcodes: &gen_opcodes::<0x1000>(0x1000_u16, &[]),
        width_bits: 16,
    };
    const CALL: Chip8InstructionDefinition = Chip8InstructionDefinition {
        id: Chip8InstructionId::Call,
        mnemonic: "CALL",
        opcodes: &gen_opcodes::<0x1000>(0x2000_u16, &[]),
        width_bits: 16,
    };
    const SKIP_IF_EQUAL: Chip8InstructionDefinition = Chip8InstructionDefinition {
        id: Chip8InstructionId::SkipIfEqual,
        mnemonic: "SE",
        opcodes: &gen_opcodes::<0x1000>(0x3000_u16, &[]),
        width_bits: 16,
    };
    const SKIP_IF_NOT_EQUAL: Chip8InstructionDefinition = Chip8InstructionDefinition {
        id: Chip8InstructionId::SkipIfNotEqual,
        mnemonic: "SNE",
        opcodes: &gen_opcodes::<0x1000>(0x4000_u16, &[]),
        width_bits: 16,
    };
}

impl InstructionSet for Chip8InstructionSet {
    fn min_width_bits() -> usize {
        16
    }

    fn max_width_bits() -> usize {
        16
    }
}

pub trait Chip8Instruction: Instruction<u16> {
    fn address(&self) -> u16 {
        self.opcode() & 0x0FFF
    }
}

// ------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct ClearScreen {
    value: u16,
}

impl Chip8Instruction for ClearScreen {}

impl Instruction<u16> for ClearScreen {
    type Id = Chip8InstructionId;

    fn id(&self) -> Self::Id {
        Chip8InstructionId::ClearScreen
    }

    fn create(value: u16) -> Self {
        Self { value }
    }

    fn opcode(&self) -> u16 {
        self.value
    }
}

impl fmt::Display for ClearScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Chip8InstructionSet::CLEAR_SCREEN.mnemonic)
    }
}

// ------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Return {
    value: u16,
}

impl Chip8Instruction for Return {}

impl Instruction<u16> for Return {
    type Id = Chip8InstructionId;

    fn id(&self) -> Self::Id {
        Chip8InstructionId::Return
    }

    fn create(value: u16) -> Self {
        Self { value }
    }

    fn opcode(&self) -> u16 {
        self.value
    }
}

impl fmt::Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Chip8InstructionSet::RETURN.mnemonic)
    }
}

// ------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct ExecuteMachineSubroutine {
    value: u16,
}

impl ExecuteMachineSubroutine {
    pub fn address(&self) -> u16 {
        self.value & 0x0FFF
    }
}

impl Chip8Instruction for ExecuteMachineSubroutine {}

impl Instruction<u16> for ExecuteMachineSubroutine {
    type Id = Chip8InstructionId;

    fn id(&self) -> Self::Id {
        Chip8InstructionId::ExecuteMachineSubroutine
    }

    fn create(value: u16) -> Self {
        Self { value }
    }

    fn opcode(&self) -> u16 {
        self.value
    }
}

impl fmt::Display for ExecuteMachineSubroutine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{0} 0x{1:04X}",
            Chip8InstructionSet::EXECUTE_MACHINE_SUBROUTINE.mnemonic,
            self.address()
        ))
    }
}

// ------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Jump {
    value: u16,
}

impl Jump {
    pub fn address(&self) -> u16 {
        self.value & 0x0FFF
    }
}

impl Chip8Instruction for Jump {}

impl Instruction<u16> for Jump {
    type Id = Chip8InstructionId;

    fn id(&self) -> Self::Id {
        Chip8InstructionId::Jump
    }

    fn create(value: u16) -> Self {
        Self { value }
    }

    fn opcode(&self) -> u16 {
        self.value
    }
}

impl fmt::Display for Jump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{0} 0x{1:04X}",
            Chip8InstructionSet::JUMP.mnemonic,
            self.address()
        ))
    }
}

// ------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Call {
    value: u16,
}

impl Call {
    pub fn address(&self) -> u16 {
        self.value & 0x0FFF
    }
}

impl Chip8Instruction for Call {}

impl Instruction<u16> for Call {
    type Id = Chip8InstructionId;

    fn id(&self) -> Self::Id {
        Chip8InstructionId::Call
    }

    fn create(value: u16) -> Self {
        Self { value }
    }

    fn opcode(&self) -> u16 {
        self.value
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{0} 0x{1:04X}",
            Chip8InstructionSet::CALL.mnemonic,
            self.address()
        ))
    }
}

// ------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct SkipIfEqual {
    value: u16,
}

impl SkipIfEqual {
    pub fn register_index(&self) -> u8 {
        ((self.value & 0x0F00) >> 8).try_into().unwrap()
    }

    pub fn immediate(&self) -> u8 {
        (self.value & 0x00FF).try_into().unwrap()
    }
}

impl Chip8Instruction for SkipIfEqual {}

impl Instruction<u16> for SkipIfEqual {
    type Id = Chip8InstructionId;

    fn id(&self) -> Self::Id {
        Chip8InstructionId::SkipIfEqual
    }

    fn create(value: u16) -> Self {
        Self { value }
    }

    fn opcode(&self) -> u16 {
        self.value
    }
}

impl fmt::Display for SkipIfEqual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{0} V{1}, 0x{2:02X}",
            Chip8InstructionSet::SKIP_IF_EQUAL.mnemonic,
            self.register_index(),
            self.immediate()
        ))
    }
}

// ------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct SkipIfNotEqual {
    value: u16,
}

impl SkipIfNotEqual {
    pub fn register_index(&self) -> u8 {
        ((self.value & 0x0F00) >> 8).try_into().unwrap()
    }

    pub fn immediate(&self) -> u8 {
        (self.value & 0x00FF).try_into().unwrap()
    }
}

impl Chip8Instruction for SkipIfNotEqual {}

impl Instruction<u16> for SkipIfNotEqual {
    type Id = Chip8InstructionId;

    fn id(&self) -> Self::Id {
        Chip8InstructionId::SkipIfNotEqual
    }

    fn create(value: u16) -> Self {
        Self { value }
    }

    fn opcode(&self) -> u16 {
        self.value
    }
}

impl fmt::Display for SkipIfNotEqual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{0} V{1}, 0x{2:02X}",
            Chip8InstructionSet::SKIP_IF_NOT_EQUAL.mnemonic,
            self.register_index(),
            self.immediate()
        ))
    }
}

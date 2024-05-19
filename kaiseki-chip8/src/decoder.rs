use crate::arch::instructions::{
    AddReg, AddRegReg, AddRegVI, AndRegReg, Call, Chip8Instruction, Chip8InstructionId,
    ClearScreen, Draw, ExecuteMachineSubroutine, GetDelayTimer, Jump, JumpPlusV0, LoadRegs,
    OrRegReg, Random, Return, SetDelayTimer, SetReg, SetRegReg, SetSoundTimer, SetVI, SetVIDigit,
    ShiftLeftReg, ShiftRightReg, SkipIfEqual, SkipIfKeyNotPressed, SkipIfKeyPressed,
    SkipIfNotEqual, SkipIfRegEqual, SkipIfRegNotEqual, StoreBCD, StoreRegs, SubRegReg,
    SubRegRegReverse, WaitForKey, XorRegReg,
};
use kaiseki_core::arch::instruction::Instruction;
use kaiseki_core::cpu::decoder::{DecodeError, DecodeOne, Result};
use kaiseki_core::cpu::opcode::Opcode16;

#[derive(Debug)]
pub struct Chip8Decoder {}

impl Chip8Decoder {
    pub fn new() -> Self {
        Chip8Decoder {}
    }
}

impl DecodeOne for Chip8Decoder {
    type Instruction = Box<dyn Chip8Instruction<Id = Chip8InstructionId>>;

    fn decode_one(&self, bytes: &[u8]) -> Result<Self::Instruction> {
        let opcode = Opcode16::from_be_bytes(bytes);

        // let ins: Self::Instruction = match opcode.value() {
        //     0x0000..=0x0FFF => match opcode.value() {
        //         0x00E0 => Box::new(ClearScreen::create(opcode.value())),
        //         0x00EE => Box::new(Return::create(opcode.value())),
        //         _ => Box::new(ExecuteMachineSubroutine::create(opcode.value())),
        //     },
        //     0x1000..=0x1FFF => Box::new(Jump::create(opcode.value())),
        //     0x2000..=0x2FFF => Box::new(Call::create(opcode.value())),
        //     0x3000..=0x3FFF => Box::new(SkipIfEqual::create(opcode.value())),
        //     0x4000..=0x4FFF => Box::new(SkipIfNotEqual::create(opcode.value())),
        //     _ => Err(DecodeError::UnimplementedOpcode)?,
        // };

        let ins: Self::Instruction = match opcode.get_nybble(3) {
            0x0 => match opcode.value() {
                0x00E0 => Box::new(ClearScreen::create(opcode.value())),
                0x00EE => Box::new(Return::create(opcode.value())),
                _ => Box::new(ExecuteMachineSubroutine::create(opcode.value())),
            },
            0x1 => Box::new(Jump::create(opcode.value())),
            0x2 => Box::new(Call::create(opcode.value())),
            0x3 => Box::new(SkipIfEqual::create(opcode.value())),
            0x4 => Box::new(SkipIfNotEqual::create(opcode.value())),
            0x5 => match opcode.get_nybble(0) {
                0x0 => Box::new(SkipIfRegEqual::create(opcode.value())),
                _ => Err(DecodeError::UnimplementedOpcode)?,
            },
            0x6 => Box::new(SetReg::create(opcode.value())),
            0x7 => Box::new(AddReg::create(opcode.value())),
            0x8 => match opcode.get_nybble(0) {
                0x0 => Box::new(SetRegReg::create(opcode.value())),
                0x1 => Box::new(OrRegReg::create(opcode.value())),
                0x2 => Box::new(AndRegReg::create(opcode.value())),
                0x3 => Box::new(XorRegReg::create(opcode.value())),
                0x4 => Box::new(AddRegReg::create(opcode.value())),
                0x5 => Box::new(SubRegReg::create(opcode.value())),
                0x6 => Box::new(ShiftRightReg::create(opcode.value())),
                0x7 => Box::new(SubRegRegReverse::create(opcode.value())),
                0xE => Box::new(ShiftLeftReg::create(opcode.value())),
                _ => Err(DecodeError::UnimplementedOpcode)?,
            },
            0x9 => match opcode.get_nybble(0) {
                0x0 => Box::new(SkipIfRegNotEqual::create(opcode.value())),
                _ => Err(DecodeError::UnimplementedOpcode)?,
            },
            0xA => Box::new(SetVI::create(opcode.value())),
            0xB => Box::new(JumpPlusV0::create(opcode.value())),
            0xC => Box::new(Random::create(opcode.value())),
            0xD => Box::new(Draw::create(opcode.value())),
            0xE => match opcode.get_byte(0) {
                0x9E => Box::new(SkipIfKeyPressed::create(opcode.value())),
                0xA1 => Box::new(SkipIfKeyNotPressed::create(opcode.value())),
                _ => Err(DecodeError::UnimplementedOpcode)?,
            },
            0xF => match opcode.get_byte(0) {
                0x07 => Box::new(GetDelayTimer::create(opcode.value())),
                0x0A => Box::new(WaitForKey::create(opcode.value())),
                0x15 => Box::new(SetDelayTimer::create(opcode.value())),
                0x18 => Box::new(SetSoundTimer::create(opcode.value())),
                0x1E => Box::new(AddRegVI::create(opcode.value())),
                0x29 => Box::new(SetVIDigit::create(opcode.value())),
                0x33 => Box::new(StoreBCD::create(opcode.value())),
                0x55 => Box::new(StoreRegs::create(opcode.value())),
                0x65 => Box::new(LoadRegs::create(opcode.value())),
                _ => Err(DecodeError::UnimplementedOpcode)?,
            },
            _ => Err(DecodeError::UnimplementedOpcode)?,
        };
        Ok(ins)
    }
}

#[cfg(test)]
mod tests {
    use crate::arch::instructions::Chip8InstructionId;

    use super::*;

    fn basic_harness(opcode: u16) -> Result<<Chip8Decoder as DecodeOne>::Instruction> {
        let bytes = &opcode.to_be_bytes();
        let decoder = Chip8Decoder {};
        decoder.decode_one(bytes)
    }

    #[test]
    fn test_valid_opcodes_0x0nnn() {
        for opcode in 0x0000u16..=0x0FFFu16 {
            let result = basic_harness(opcode);
            assert!(result.is_ok());
            let instruction = result.unwrap();
            match instruction.id() {
                Chip8InstructionId::ClearScreen => {
                    assert_eq!(opcode, 0x00E0);
                    assert_eq!(ClearScreen::valid_opcodes(), &[0x00E0]);
                }
                Chip8InstructionId::Return => {
                    assert_eq!(opcode, 0x00EE);
                    assert_eq!(Return::valid_opcodes(), &[0x00EE]);
                }
                Chip8InstructionId::ExecuteMachineSubroutine => {
                    assert_eq!(opcode & 0x0FFF, instruction.address());
                    assert_ne!(opcode & 0x0FFF, 0x00E0);
                    assert_ne!(opcode & 0x0FFF, 0x00EE);
                    assert_eq!(ExecuteMachineSubroutine::valid_opcodes().len(), 0x1000 - 2);
                }
                _ => panic!("unexpected instruction: {:?}", instruction),
            }
        }
    }

    #[test]
    fn test_valid_opcodes_0x1nnn() {
        for opcode in 0x1000u16..=0x1FFFu16 {
            let result = basic_harness(opcode);
            assert!(result.is_ok());
            let instruction = result.unwrap();
            match instruction.id() {
                Chip8InstructionId::Jump => {
                    assert_eq!(opcode & 0x0FFF, instruction.address());
                }
                _ => panic!("unexpected instruction: {:?}", instruction),
            }
        }
    }

    #[test]
    fn test_valid_opcodes_0x5xy0() {
        for opcode in 0x5000u16..=0x5FFFu16 {
            let result = basic_harness(opcode);
            if opcode & 0x000F != 0x0 {
                assert!(result.is_err());
            } else {
                assert!(result.is_ok());
                let instruction = result.unwrap();
                assert_eq!(Chip8InstructionId::SkipIfRegEqual, instruction.id());
            }
            // assert!(result.is_ok());
            // let instruction = result.unwrap();
            // match instruction {
            //     Chip8Instruction::SkipIfRegEqual { vx_idx, vy_idx } => {
            //         assert_eq!(opcode & 0x0F00, vx_idx as u16);
            //         assert_eq!(opcode & 0x00F0, vy_idx as u16);
            //         assert_eq!(opcode & 0x000F, 0x0);
            //     }
            //     _ => panic!("unexpected instruction: {:?}", instruction),
            // }
        }
    }
}

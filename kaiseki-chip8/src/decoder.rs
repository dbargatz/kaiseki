use crate::instructions::{Chip8Instruction, Chip8Opcode};
use kaiseki_core::cpu::decoder::{DecodeError, DecodeOne, Result};

pub struct Chip8Decoder {}

impl DecodeOne for Chip8Decoder {
    type Instruction = Chip8Instruction;

    fn decode_one(&self, bytes: &[u8]) -> Result<Self::Instruction> {
        let opcode = Chip8Opcode::from_be_bytes(bytes);

        let ins = match opcode.value() {
            0x0000..=0x0FFF => match opcode.value() {
                0x00E0 => Chip8Instruction::ClearScreen,
                0x00EE => Chip8Instruction::Return,
                _ => Chip8Instruction::ExecuteMachineSubroutine {
                    addr: opcode.value() & 0x0FFF,
                },
            },
            0x1000..=0x1FFF => Chip8Instruction::Jump {
                addr: opcode.value() & 0x0FFF,
            },
            0x2000..=0x2FFF => Chip8Instruction::Call {
                addr: opcode.value() & 0x0FFF,
            },
            0x3000..=0x3FFF => Chip8Instruction::SkipIfEqual {
                vx_idx: ((opcode.value() & 0x0F00) >> 2) as u8,
                value: (opcode.value() & 0x00FF) as u8,
            },
            0x4000..=0x4FFF => Chip8Instruction::SkipIfNotEqual {
                vx_idx: ((opcode.value() & 0x0F00) >> 2) as u8,
                value: (opcode.value() & 0x00FF) as u8,
            },
            _ => Err(DecodeError::UnimplementedOpcode)?,
        };
        Ok(ins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_harness(opcode: u16) -> Result<Chip8Instruction> {
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
            match instruction {
                Chip8Instruction::ClearScreen => assert_eq!(opcode, 0x00E0),
                Chip8Instruction::Return => assert_eq!(opcode, 0x00EE),
                Chip8Instruction::ExecuteMachineSubroutine { addr } => {
                    assert_eq!(opcode & 0x0FFF, addr);
                    assert_ne!(opcode & 0x0FFF, 0x00E0);
                    assert_ne!(opcode & 0x0FFF, 0x00EE);
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
            match instruction {
                Chip8Instruction::Jump { addr } => {
                    assert_eq!(opcode & 0x0FFF, addr);
                }
                _ => panic!("unexpected instruction: {:?}", instruction),
            }
        }
    }

    #[test]
    fn test_valid_opcodes_0x5xy0() {
        for opcode in 0x5000u16..=0x5FFFu16 {
            let result = basic_harness(opcode);
            assert_eq!(result, Err(DecodeError::UnimplementedOpcode));
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

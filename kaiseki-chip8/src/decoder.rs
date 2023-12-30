use crate::instructions::Chip8Instruction;
use kaiseki_core::cpu::Decode;

pub struct Chip8Decoder {}

impl Decode for Chip8Decoder {
    type Instruction = Chip8Instruction;

    fn decode(&self, bytes: &[u8]) -> Vec<Self::Instruction> {
        let mut res: Vec<Chip8Instruction> = vec![];
        let slice: [u8; 2] = bytes[0..2]
            .try_into()
            .expect("couldn't convert Vec<u8> to [u8; 2]");
        let opcode = u16::from_be_bytes(slice);

        let ins = match opcode {
            0x0000..=0x0FFF => match opcode {
                0x00E0 => Chip8Instruction::ClearScreen,
                0x00EE => Chip8Instruction::Return,
                _ => Chip8Instruction::ExecuteMachineSubroutine {
                    addr: opcode & 0x0FFF,
                },
            },
            0x1000..=0x1FFF => Chip8Instruction::Jump {
                addr: opcode & 0x0FFF,
            },
            0x2000..=0x2FFF => Chip8Instruction::Call {
                addr: opcode & 0x0FFF,
            },
            0x3000..=0x3FFF => Chip8Instruction::SkipIfEqual {
                vx_idx: ((opcode & 0x0F00) >> 2) as u8,
                value: (opcode & 0x00FF) as u8,
            },
            0x4000..=0x4FFF => Chip8Instruction::SkipIfNotEqual {
                vx_idx: ((opcode & 0x0F00) >> 2) as u8,
                value: (opcode & 0x00FF) as u8,
            },
            _ => panic!("unhandled instruction 0x{:04X}", opcode),
        };
        res.push(ins);

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_harness(opcode: u16) -> Chip8Instruction {
        let bytes = &opcode.to_be_bytes();
        let decoder = Chip8Decoder {};
        let mut instructions = decoder.decode(bytes);
        assert_eq!(instructions.len(), 1);
        instructions.pop().unwrap()
    }

    #[test]
    fn test_valid_opcodes_0x0nnn() {
        for opcode in 0x0000u16..=0x0FFFu16 {
            let instruction = basic_harness(opcode);
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
            let instruction = basic_harness(opcode);
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
            let instruction = basic_harness(opcode);
            match instruction {
                Chip8Instruction::SkipIfRegEqual { vx_idx, vy_idx } => {
                    assert_eq!(opcode & 0x0F00, vx_idx as u16);
                    assert_eq!(opcode & 0x00F0, vy_idx as u16);
                    assert_eq!(opcode & 0x000F, 0x0);
                }
                _ => panic!("unexpected instruction: {:?}", instruction),
            }
        }
    }
}

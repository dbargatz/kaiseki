use kaiseki_core::isa::instruction::{Instruction, InstructionDef, OperandDef};

const N: OperandDef = OperandDef::Register {
    pattern: "n",
    format: "Vn",
    width_bits: 4,
};
const X: OperandDef = OperandDef::Register {
    pattern: "x",
    format: "Vx",
    width_bits: 4,
};
const Y: OperandDef = OperandDef::Register {
    pattern: "y",
    format: "Vy",
    width_bits: 4,
};
const KK: OperandDef = OperandDef::ImmediateAddress {
    pattern: "kk",
    width_bits: 8,
};
const VF: OperandDef = OperandDef::RegisterImplicit { name: "VF" };
const NNN: OperandDef = OperandDef::ImmediateAddress {
    pattern: "nnn",
    width_bits: 12,
};

const SYS: InstructionDef = InstructionDef::new("SYS", "0x0nnn", &[NNN]);
const CLS: InstructionDef = InstructionDef::new("CLS", "0x00E0", &[]);
const RET: InstructionDef = InstructionDef::new("RET", "0x00EE", &[]);
const JP_ABS: InstructionDef = InstructionDef::new("JP", "0x1nnn", &[NNN]);
const CALL: InstructionDef = InstructionDef::new("CALL", "0x2nnn", &[NNN]);
const SE_IMM: InstructionDef = InstructionDef::new("SE", "0x3xkk", &[X, KK]);
const SNE_IMM: InstructionDef = InstructionDef::new("SNE", "0x4xkk", &[X, KK]);
const SE_REG: InstructionDef = InstructionDef::new("SE", "0x5xy0", &[X, Y]);
const LD_IMM: InstructionDef = InstructionDef::new("LD", "0x6xkk", &[X, KK]);
const ADD_IMM: InstructionDef = InstructionDef::new("ADD", "0x7xkk", &[X, KK]);
const LD_REG: InstructionDef = InstructionDef::new("LD", "0x8xy0", &[X, Y]);
const OR_REG: InstructionDef = InstructionDef::new("OR", "0x8xy1", &[X, Y]);
const AND_REG: InstructionDef = InstructionDef::new("AND", "0x8xy2", &[X, Y]);
const XOR_REG: InstructionDef = InstructionDef::new("XOR", "0x8xy3", &[X, Y]);
const ADD_REG: InstructionDef = InstructionDef::new("ADD", "0x8xy4", &[X, Y]);
const SUB_REG: InstructionDef = InstructionDef::new("SUB", "0x8xy5", &[X, Y]);
const SHR_REG: InstructionDef = InstructionDef::new("SHR", "0x8xy6", &[X, Y]);
const SUBN_REG: InstructionDef = InstructionDef::new("SUBN", "0x8xy7", &[X, Y]);
const SHL_REG: InstructionDef = InstructionDef::new("SHL", "0x8xyE", &[X, Y]);
const SNE_REG: InstructionDef = InstructionDef::new("SNE", "0x9xy0", &[X, Y]);
const LD_IDX: InstructionDef = InstructionDef::new("LD", "0xAnnn", &[NNN]);
const JP_OFF: InstructionDef = InstructionDef::new("JP", "0xBnnn", &[NNN]);
const RND: InstructionDef = InstructionDef::new("RND", "0xCxkk", &[NNN]);
const DRW: InstructionDef = InstructionDef::new("DRW", "0xDxyn", &[X, Y, N]);
const SKP: InstructionDef = InstructionDef::new("SKP", "0xEx9E", &[X]);
const SKNP: InstructionDef = InstructionDef::new("SKNP", "0xExA1", &[X]);
const LD_RDT: InstructionDef = InstructionDef::new("LD", "0xFx07", &[X]);
const LD_KEY: InstructionDef = InstructionDef::new("LD", "0xFx0A", &[X]);
const LD_WDT: InstructionDef = InstructionDef::new("LD", "0xFx15", &[X]);
const LD_WST: InstructionDef = InstructionDef::new("LD", "0xFx18", &[X]);
const ADD_IDX: InstructionDef = InstructionDef::new("ADD", "0xFx1E", &[X]);
const LD_SPR: InstructionDef = InstructionDef::new("LD", "0xFx29", &[X]);
const LD_BCD: InstructionDef = InstructionDef::new("LD", "0xFx33", &[X]);
const LD_PSH: InstructionDef = InstructionDef::new("LD", "0xFx55", &[X]);
const LD_POP: InstructionDef = InstructionDef::new("LD", "0xFx65", &[X]);

// Operand characteristics:
//  - Type:   is it a constant/immediate, register, address, offset, ...?
//  - Source: is it embedded in the instruction, implicit, something else?

// 0x0nnn
// SYS nnn:
//   Operands: 1
//     - nnn: Explicit. 12-bit address (immediate) encoded in instruction
//   InstructionDef { "SYS", "0x0nnn", Operand::Immediate("n") }

// 0x00E0
// CLS:
//   Operands: 0
//   InstructionDef { "CLS", "0x00E0" }

// 0x6xnn
// LD Vx, nn: Vx := nn
//   Operands: 2
//     - x : Explicit. General purpose register encoded in instruction
//     - nn: Explicit. 8-bit constant (immediate) encoded in instruction
//   InstructionDef { "LD", "0x6xnn", Operand::Register("x"), Operand::Immediate("n") }

// 0x8xy2
// AND Vx, Vy: Vx := Vx & Vy
//   Operands: 2
//     - x : Explicit. General purpose register encoded in instruction
//     - y : Explicit. General purpose register encoded in instruction
//   InstructionDef { "AND", "0x8xy2", Operand::Register("x"), Operand::Register("y") }
//     decode(cpu_context??, bytes??) -> Result<Instruction>
//   Instruction { ^def, address, bytes, Register::Vx, Register::Vy }
//     interpret(cpu_context)

// 0x8xy6
// SHR Vx, {Vy}: VF := Vy & 0x01; Vy := Vy >> 1; Vx := Vy
//   Operands: 3
//     - x : Explicit. General purpose register encoded in instruction
//     - y : Explicit. General purpose register encoded in instruction
//     - VF: Implicit. Flags register always modified by instruction
//   InstructionDef { "SHR", "0x8xy6", Operand::Register("Vx"), Operand::Register("Vy"), Operand::ImplicitRegister("VF") }

// 0nnn
// 00E0
// 00EE
// 1nnn
// 2nnn
// 3xkk
// 4xkk
// 5xy0
// 6xkk
// 7xkk
// 8xy0
// 8xy1
// 8xy2
// 8xy3
// 8xy4
// 8xy5
// 8xy6
// 8xy7
// 8xyE
// 9xy0
// Annn
// Bnnn
// Cxkk
// Dxyn
// Ex9E
// ExA1
// Fx07
// Fx0A
// Fx15
// Fx18
// Fx1E
// Fx29
// Fx33
// Fx55
// Fx65
pub fn decode_one(address: u16, bytes: &[u8]) -> Instruction {
    assert!(bytes.len() >= 2);

    let instr = ((bytes[0] as u16) << 8) + (bytes[1] as u16);
    let nybble3 = bytes[0] >> 4;
    let nybble2 = bytes[0] & 0x0F;
    let nybble1 = bytes[1] >> 4;
    let nybble0 = bytes[1] & 0x0F;

    match nybble3 {
        // 0? t-> 0E?  t-> 0? t-> 00E0
        //                    f-> E? t-> 00EE
        //                           f-> invalid
        //             f-> 0nnn
        0x0 => match nybble2 {
            0x0 => match nybble1 {
                0xE => match nybble0 {
                    0x0 => CLS.decode(address, bytes),
                    0xE => RET.decode(address, bytes),
                    _ => SYS.decode(address, bytes),
                },
                _ => SYS.decode(address, bytes),
            },
            _ => SYS.decode(address, bytes),
        },

        // 1? t-> 1nnn
        0x1 => JP_ABS.decode(address, bytes),

        // 2? t-> 2nnn
        0x2 => CALL.decode(address, bytes),

        // 3? t-> 3xkk
        0x3 => SE_IMM.decode(address, bytes),

        // 4? t-> 4xkk
        0x4 => SNE_IMM.decode(address, bytes),

        // 5? t-> xy0? t-> 5xy0
        //             f-> invalid
        0x5 => match nybble0 {
            0x0 => SE_REG.decode(address, bytes),
            _ => panic!("invalid instruction 0x{:x} at 0x{:x}", address, instr),
        },

        // 6? t-> 6xkk
        0x6 => LD_IMM.decode(address, bytes),

        // 7? t-> 7xkk
        0x7 => ADD_IMM.decode(address, bytes),

        // 8? t-> Skipped for now
        // 0x8 => todo!(),

        // 9? t-> xy0? t-> 9xy0
        //             f-> invalid
        0x9 => match nybble0 {
            0x0 => SNE_REG.decode(address, bytes),
            _ => panic!("invalid instruction 0x{:x} at 0x{:x}", address, instr),
        },

        // A? t-> Annn
        0xA => LD_IDX.decode(address, bytes),

        // B? t-> Bnnn
        0xB => JP_OFF.decode(address, bytes),

        // C? t-> Cxkk
        0xC => RND.decode(address, bytes),

        // D? t-> Dxyn
        0xD => DRW.decode(address, bytes),

        // E? t-> x9E? t-> Ex9E
        //             f-> xA1? t-> ExA1
        //                      f-> invalid
        //0xE => todo!(),

        // F? t-> x0?  t-> 7?   t-> Fx07
        //                      f-> A?   t-> Fx0A
        //                               f-> invalid
        //             f-> x1?  t-> 5?   t-> Fx15
        //                               f-> 8?   t-> Fx18
        //                                        f-> E?   t-> Fx1E
        //                                                 f-> invalid
        //                      f-> x29? t -> Fx29
        //                               f-> x33? t-> Fx33
        //                                        f-> x55? t-> Fx55
        //                                                 f-> x65? t-> Fx65
        //                                                          f-> invalid
        //    f-> invalid
        //0xF => todo!(),
        _ => panic!("invalid instruction 0x{:x} at 0x{:x}", address, instr),
    }
}

// use super::instruction::{Cpu, DisassembleResult, Instruction, InstructionSet, InterpretResult, Isa};

// pub enum Chip8Register {
//     PC,
//     SP,

//     V0,
//     V1,
//     V2,
//     V3,
//     V4,
//     V5,
//     V6,
//     V7,
//     V8,
//     V9,
//     VA,
//     VB,
//     VC,
//     VD,
//     VE,
//     VF,

//     DT,
//     ST,
// }

// #[allow(non_snake_case)]
// pub struct Chip8RegisterSet {
//     PC: u16,
//     SP: u8,

//     V0: u16,
//     V1: u16,
//     V2: u16,
//     V3: u16,
//     V4: u16,
//     V5: u16,
//     V6: u16,
//     V7: u16,
//     V8: u16,
//     V9: u16,
//     VA: u16,
//     VB: u16,
//     VC: u16,
//     VD: u16,
//     VE: u16,
//     VF: u16,

//     DT: u16,
//     ST: u16,
// }

// #[allow(non_camel_case_types)]
// pub enum Chip8InstructionSet {
//     CLS,
//     RET,
//     SYS,
//     JP_A,
//     CALL,
//     SE_RC,
//     SNE_RC,
//     SE_RR,
//     LD_RC,
//     ADD_RC,
//     LD_RR,
//     OR,
//     AND,
//     XOR,
//     ADD_RR,
//     SUB,
//     SHR,
//     SUBN,
//     SHL,
//     SNE_RR,
//     LD_IA,
//     JP_RC,
//     RND,
//     DRW,
//     SKP,
//     SKNP,
//     LD_RD,
//     LD_RK,
//     LD_DR,
//     LD_SR,
//     ADD_IR,
//     LD_IR_SPRITE,
//     LD_IR_BCD,
//     LD_IA_R,
//     LD_R_IA,
// }

// impl InstructionSet<Chip8Cpu> for Chip8InstructionSet {
//     fn disassemble(bytes: &[u8], cpu: &Chip8Cpu) -> DisassembleResult<Box<dyn Instruction<Chip8Cpu>>> {
//         let opcode = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
//         match opcode {

//         }
//     }
// }

// pub struct Chip8CallInstruction {
//     bytes: u16,
// }
// impl Instruction<Chip8Cpu> for Chip8CallInstruction {
//     fn interpret(&self, cpu: &mut Chip8Cpu) -> InterpretResult {
//         todo!()
//     }
// }

// impl Chip8CallInstruction {
//     pub fn disassemble(bytes: &[u8]) -> Option<Self> {
//         if bytes[0] & 0xF0 == 0x20 {
//             let bytes: u16 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
//             Some(Self { bytes })
//         } else {
//             None
//         }
//     }
// }

// pub struct Chip8RetInstruction {
//     bytes: u16,
// }
// impl Instruction<Chip8Cpu> for Chip8RetInstruction {
//     fn interpret(&self, cpu: &mut Chip8Cpu) -> InterpretResult {
//         todo!()
//     }
// }

// impl Chip8RetInstruction {
//     pub fn disassemble(bytes: &[u8]) -> Option<Self> {
//         let opcode = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
//         if opcode == 0x00EE {
//             Some(Self { bytes: opcode })
//         } else {
//             None
//         }
//     }
// }

// // pub struct Chip8InstructionSet {}

// // impl Chip8InstructionSet {
// //     pub fn disassemble(bytes: &[u8]) -> Option<impl Instruction> {
// //         if let Some(x) = Chip8CallInstruction::disassemble(bytes) {
// //             return Some(x);
// //         }

// //         if let Some(x) = Chip8RetInstruction::disassemble(bytes) {
// //             return Some(x);
// //         }

// //         return None;
// //     }
// // }

// pub struct Chip8Cpu {
//     regs: <Self as Isa>::RegisterSet,
//     mem: [u8; 4096]
// }

// impl Isa for Chip8Cpu {
//     type RegisterSet = Chip8RegisterSet;
//     type InstructionSet = Chip8InstructionSet;
// }

// impl Cpu for Chip8Cpu {
//     fn execute_one(&mut self) {
//         let pc = self.regs.PC as usize;
//         let bytes = &self.mem[pc..=pc+1];
//         self.regs.PC += 2;

//         let inst = <Self as Isa>::InstructionSet::disassemble(bytes, self);
//     }
// }

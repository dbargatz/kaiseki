pub mod decoder;
pub mod opcode;

use opcode::OpcodeValue;
use std::fmt;

pub trait InstructionSet {
    fn min_width_bits() -> usize;
    fn max_width_bits() -> usize;
}

pub trait Instruction<V: OpcodeValue>: fmt::Debug {
    type Id: InstructionId;

    fn id(&self) -> Self::Id;
    fn create(value: V) -> Self
    where
        Self: Sized;
    fn opcode(&self) -> V;
}

pub trait InstructionDefinition {
    type Id: InstructionId;
    type OpcodeValue: OpcodeValue;

    fn id(&self) -> Self::Id;
    fn mnemonic(&self) -> &'static str;
    fn valid_opcodes(&self) -> &'static [Self::OpcodeValue];
    fn width_bits(&self) -> usize;
}

pub trait InstructionId {}

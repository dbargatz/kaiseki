use crate::cpu::Instruction;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DecodeError {
    #[error("invalid opcode")]
    InvalidOpcode,
    #[error("unimplemented opcode")]
    UnimplementedOpcode,
}

pub type Result<T> = std::result::Result<T, DecodeError>;

pub trait DecodeOne {
    type Instruction: Instruction;

    fn decode_one(&self, bytes: &[u8]) -> Result<Self::Instruction>;
}

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DisassemblyError {
    #[error("sequence of bytes does not map to a known instruction")]
    UndefinedInstruction,
}

pub type Result<T> = std::result::Result<T, DisassemblyError>;

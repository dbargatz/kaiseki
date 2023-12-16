use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DisassemblyError {
    #[error("value {0} violates field {1} constraint")]
    ConstraintViolated(&'static str, usize),

    #[error("no field with name {0} has been defined")]
    NoSuchField(String),

    #[error("sequence of bytes does not map to a known instruction")]
    UndefinedInstruction,
}

pub type Result<T> = std::result::Result<T, DisassemblyError>;

pub mod decoder;
pub mod opcode;

pub trait Instruction {
    fn len_bytes(&self) -> usize;
}

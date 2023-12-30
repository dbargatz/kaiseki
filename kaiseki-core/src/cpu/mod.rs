pub mod decoder;
pub mod opcode;

pub trait Instruction {
    type Opcode;
    fn len_bytes(&self) -> usize;
}

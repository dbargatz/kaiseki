pub trait Instruction {
    fn len_bytes(&self) -> usize;
}

pub trait Decode {
    type Instruction: Instruction;

    fn decode(&self, bytes: &[u8]) -> Vec<Self::Instruction>;
}

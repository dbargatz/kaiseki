use std::{fmt, ops};

pub trait RegisterDef {}

pub struct Register<'a, T: num::Unsigned> {
    name: &'a str,
    value: T,
}

impl<'a, T: num::Unsigned> Register<'a, T> {
    pub fn new(name: &'a str, value: T) -> Self {
        Self { name, value }
    }
}

impl<'a, T: num::Unsigned + fmt::UpperHex> fmt::Debug for Register<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{} 0x{:04X}", self.name, self.value))
    }
}

impl<'a, T: num::Unsigned> ops::Deref for Register<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T: num::Unsigned> ops::DerefMut for Register<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'a, T: num::Unsigned> RegisterDef for Register<'a, T> {}

pub type R8<'a> = Register<'a, u8>;
pub type R16<'a> = Register<'a, u16>;
pub type R32<'a> = Register<'a, u32>;
pub type R64<'a> = Register<'a, u64>;

pub trait RegisterSetDef {}

enum Chip8RegisterSet<'a> {
    V0 { value: R8<'a> },
    V1 { value: R8<'a> },
    V2 { value: R8<'a> },
    V3 { value: R8<'a> },
    V4 { value: R8<'a> },
    V5 { value: R8<'a> },
    V6 { value: R8<'a> },
    V7 { value: R8<'a> },
    V8 { value: R8<'a> },
    V9 { value: R8<'a> },
    VA { value: R8<'a> },
    VB { value: R8<'a> },
    VC { value: R8<'a> },
    VD { value: R8<'a> },
    VE { value: R8<'a> },
    VF { value: R8<'a> },

    I { value: R16<'a> }, // can only be loaded with 12-bit addrs

    DT { value: R8<'a> },
    ST { value: R8<'a> },

    PC { value: R16<'a> },
    SP { value: R8<'a> },
}

impl<'a> RegisterSetDef for Chip8RegisterSet<'a> {}

impl<'a> ops::Deref for Chip8RegisterSet<'a> {
    type Target = R8<'a>;

    fn deref(&self) -> &Self::Target {
        match self {
            Chip8RegisterSet::V0 { value } => value,
            Chip8RegisterSet::V1 { value } => value,
            Chip8RegisterSet::V2 { value } => value,
            Chip8RegisterSet::V3 { value } => value,
            Chip8RegisterSet::V4 { value } => value,
            Chip8RegisterSet::V5 { value } => value,
            Chip8RegisterSet::V6 { value } => value,
            Chip8RegisterSet::V7 { value } => value,
            Chip8RegisterSet::V8 { value } => value,
            Chip8RegisterSet::V9 { value } => value,
            Chip8RegisterSet::VA { value } => value,
            Chip8RegisterSet::VB { value } => value,
            Chip8RegisterSet::VC { value } => value,
            Chip8RegisterSet::VD { value } => value,
            Chip8RegisterSet::VE { value } => value,
            Chip8RegisterSet::VF { value } => value,
            // Chip8RegisterSet::I  { value } => value,
            Chip8RegisterSet::DT { value } => value,
            Chip8RegisterSet::ST { value } => value,
            // Chip8RegisterSet::PC { value } => value,
            Chip8RegisterSet::SP { value } => value,
            _ => panic!("how you get here?"),
        }
    }
}

impl<'a> ops::DerefMut for Chip8RegisterSet<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Chip8RegisterSet::V0 { value } => value,
            Chip8RegisterSet::V1 { value } => value,
            Chip8RegisterSet::V2 { value } => value,
            Chip8RegisterSet::V3 { value } => value,
            Chip8RegisterSet::V4 { value } => value,
            Chip8RegisterSet::V5 { value } => value,
            Chip8RegisterSet::V6 { value } => value,
            Chip8RegisterSet::V7 { value } => value,
            Chip8RegisterSet::V8 { value } => value,
            Chip8RegisterSet::V9 { value } => value,
            Chip8RegisterSet::VA { value } => value,
            Chip8RegisterSet::VB { value } => value,
            Chip8RegisterSet::VC { value } => value,
            Chip8RegisterSet::VD { value } => value,
            Chip8RegisterSet::VE { value } => value,
            Chip8RegisterSet::VF { value } => value,
            // Chip8RegisterSet::I { value } => value,
            Chip8RegisterSet::DT { value } => value,
            Chip8RegisterSet::ST { value } => value,
            // Chip8RegisterSet::PC { value } => value,
            Chip8RegisterSet::SP { value } => value,
            _ => panic!("how you get here?"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read() {
        let a = R8::new("a", 17);
        assert_eq!(*a, 17);
    }

    #[test]
    fn can_write() {
        let mut a = R8::new("a", 0);
        *a = 255;
        assert_eq!(*a, 255);
    }
}

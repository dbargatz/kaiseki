use std::fmt;

#[derive(Debug)]
pub enum OperandDef<'a> {
    ImmediateAddress {
        pattern: &'a str,
        width_bits: usize,
    },
    ImmediateConstant {
        pattern: &'a str,
        width_bits: usize,
    },
    ImmediateOffset {
        pattern: &'a str,
        width_bits: usize,
    },
    Register {
        pattern: &'a str,
        format: &'a str,
        width_bits: usize,
    },
    RegisterImplicit {
        name: &'a str,
    },
}

impl<'a> OperandDef<'a> {
    pub fn is_explicit(&self) -> bool {
        !self.is_implicit()
    }

    pub fn is_implicit(&self) -> bool {
        match self {
            OperandDef::RegisterImplicit { .. } => true,
            _ => false,
        }
    }
}

impl<'a> fmt::Display for OperandDef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt = match self {
            OperandDef::Register { format, .. } => format,
            OperandDef::RegisterImplicit { name, .. } => name,
            _ => panic!("unhandled fmt::Display for OperandDef"),
        };
        f.write_str(fmt)
    }
}

pub struct InstructionDef<'a> {
    mnemonic: &'a str,
    pattern: &'a str,
    operands: &'a [OperandDef<'a>],
}

impl<'a> InstructionDef<'a> {
    pub const fn new(mnemonic: &'a str, pattern: &'a str, operands: &'a [OperandDef<'a>]) -> Self {
        Self {
            mnemonic,
            pattern,
            operands,
        }
    }

    pub fn decode(&'a self, address: u16, bytes: &'a [u8]) -> Instruction<'a> {
        // TODO: ensure it matches the pattern!
        Instruction::new(address, bytes, &self)
    }
}

impl<'a> fmt::Debug for InstructionDef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstructionDef")
            .field("mnemonic", &self.mnemonic)
            .field("pattern", &self.pattern)
            .field("operands", &self.operands)
            .finish()
    }
}

impl<'a> fmt::Display for InstructionDef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt_str = format!("{}", self.mnemonic);
        for op in self.operands {
            if op.is_explicit() {
                fmt_str.push_str(&format!(" {},", op));
            }
        }
        if fmt_str.ends_with(',') {
            fmt_str.remove(fmt_str.len() - 1);
        }
        f.write_str(&fmt_str)
    }
}

pub struct Instruction<'a> {
    address: u16,
    data: Vec<u8>,
    def: &'a InstructionDef<'a>,
}

impl<'a> Instruction<'a> {
    pub fn new(address: u16, bytes: &'a [u8], def: &'a InstructionDef<'a>) -> Self {
        Self {
            address,
            data: Vec::from(bytes),
            def,
        }
    }
}

impl<'a> fmt::Debug for Instruction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instruction")
            .field("address", &self.address)
            .field("data", &self.data)
            .field("def", &self.def)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_build() {
        let op1 = OperandDef::Register {
            pattern: "x",
            format: "Vx",
            width_bits: 4,
        };
        let op2 = OperandDef::Register {
            pattern: "y",
            format: "Vy",
            width_bits: 4,
        };
        let op3 = OperandDef::RegisterImplicit { name: "VF" };
        let operands = [op1, op2, op3];
        let shr = InstructionDef::new("SHR", "0x8xy6", &operands);
        assert_eq!(&format!("{}", shr), "SHR Vx, Vy");
    }
}

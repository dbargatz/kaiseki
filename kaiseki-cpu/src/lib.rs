use std::fmt;

use bitvec::prelude::*;

mod error;
use error::{DisassemblyError, Result};

mod field;
use field::Field;

#[derive(PartialEq)]
struct InstructionVariantDefinition {
    name: String,
    fields: Vec<Field>,
}

impl InstructionVariantDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            fields: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    pub fn try_disassemble(&self, data: &[u8]) -> Result<()> {
        let mut stream = data.view_bits::<Lsb0>();
        for field in &self.fields {
            if let Err(err) = field.try_disassemble(stream) {
                return Err(err);
            }
            stream = &stream[field.width_bits()..];
        }
        Ok(())
    }
}

impl fmt::Debug for InstructionVariantDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut substr = String::new();
        for field in &self.fields {
            substr.push_str(&format!("[{:?}] ", field));
        }
        f.write_fmt(format_args!("{}: {}", self.name, substr.trim_end()))
    }
}

#[derive(Debug, Default)]
struct InstructionSet {
    instructions: Vec<InstructionDefinition>,
}

impl InstructionSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_instruction_definition(&mut self, instruction: InstructionDefinition) {
        self.instructions.push(instruction);
    }

    pub fn disassemble(&self, data: &[u8]) -> Result<()> {
        for instruction in &self.instructions {
            if let Ok(_) = instruction.disassemble(data) {
                return Ok(());
            }
        }
        Err(DisassemblyError::UndefinedInstruction)
    }
}

#[derive(Debug, PartialEq)]
struct InstructionDefinition {
    name: String,
    variants: Vec<InstructionVariantDefinition>,
}

impl InstructionDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            variants: Vec::new(),
        }
    }

    pub fn add_variant_definition(&mut self, variant: InstructionVariantDefinition) {
        self.variants.push(variant);
    }

    pub fn disassemble(&self, data: &[u8]) -> Result<()> {
        for variant in &self.variants {
            if let Ok(_) = variant.try_disassemble(data) {
                return Ok(());
            }
        }
        Err(DisassemblyError::UndefinedInstruction)
    }
}


#[cfg(test)]
mod tests {
    use rangemap::RangeInclusiveMap;

    use crate::{DisassemblyError, InstructionDefinition, InstructionSet, InstructionVariantDefinition, field::{Field, FieldDefinition}};

    const PUSH_IMM_16: &[u8] = &[0xFF, 0b00_110_000, 0xAA, 0xBB];
    const RESERVED_OPCODE: &[u8] = &[0x0F, 0x04]; // 2-byte reserved opcode

    fn create_push_imm16(opcode8: &FieldDefinition, modrm: &FieldDefinition, r#mod: &FieldDefinition, reg_opcode: &FieldDefinition, reg_memory: &FieldDefinition, imm16: &FieldDefinition) -> InstructionVariantDefinition {
        let mut push_imm_16 = InstructionVariantDefinition::new("push [imm16]");
        let f1 = Field::SpecificValue(opcode8.clone(), 0xFF);
        let mut f2 = Field::Subfields(modrm.clone(), RangeInclusiveMap::new());
        f2.add_subfield(6..=7, Field::AnyValue(r#mod.clone()));
        f2.add_subfield(3..=5, Field::SpecificValue(reg_opcode.clone(), 0b110));
        f2.add_subfield(0..=2, Field::AnyValue(reg_memory.clone()));
        let f3 = Field::AnyValue(imm16.clone());

        push_imm_16.add_field(f1);
        push_imm_16.add_field(f2);
        push_imm_16.add_field(f3);
        push_imm_16
    }

    fn create_isa() -> InstructionSet {
        // regex pattern: "[prefix]* [opcode]{1,3} [modrm]? [sib]? [imm]? [disp]?";
        // prefix = 0x26, 0x36, 0x64, 0x65, 0x66, 0x67, 0xF0, 0xF2, 0xF3, 
        // opcode = 1-byte:                0x00 - 0x0E, 0x10 - 0xFF
        //          2-byte: 0x0F      then 0x00 - 0x37, 0x3B - 0xFF
        //          3-byte: 0x0F 0x38 then 0x00 - 0xFF
        //                  0x0F 0x3A then 0x00 - 0xFF
        // modrm pattern: "xx yyy zzz", x: mod, y: reg/opcode, z: reg/mem
        // sib pattern:   "xx yyy zzz", x: scale, y: index, z: base
        // imm pattern:   1, 2, 4, or 8 bytes
        // disp pattern:  1, 2, 4, or 8 bytes
        let mut x86_isa = InstructionSet::new();

        let modrm = FieldDefinition::new("mod_rm", 8);
        let r#mod = FieldDefinition::new("mod", 2);
        let reg_opcode = FieldDefinition::new("reg_opcode", 3);
        let reg_memory = FieldDefinition::new("reg_memory", 3);
        // modrm.add_subfield_definition(6..=7, r#mod);
        // modrm.add_subfield_definition(3..=5, reg_opcode);
        // modrm.add_subfield_definition(0..=2, reg_memory);

        let opcode8 = FieldDefinition::new("opcode8", 8);
        let imm16 = FieldDefinition::new("imm16", 16);

        let push_imm_16 = create_push_imm16(&opcode8, &modrm, &r#mod, &reg_opcode, &reg_memory, &imm16);

        let mut push = InstructionDefinition::new("push");

        push.add_variant_definition(push_imm_16);

        x86_isa.add_instruction_definition(push);
        x86_isa
    }

    #[test]
    fn reserved_opcode_fails() {
        let isa = create_isa();
        println!("{:#?}", isa);
        assert_eq!(isa.disassemble(RESERVED_OPCODE), Err(DisassemblyError::UndefinedInstruction));
    }

    #[test]
    fn push_imm_16() {
        let isa = create_isa();
        println!("{:#?}", isa);
        assert!(isa.disassemble(PUSH_IMM_16).is_ok());
    }
}
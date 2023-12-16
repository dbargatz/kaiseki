// use bitvec::prelude::*;

mod error;
mod field;

#[cfg(test)]
mod tests {
    // use std::collections::HashMap;

    // use crate::{DisassemblyError, field::{Field, FieldValueConstraint}};

    // const PUSH_IMM_16: &[u8] = &[0xFF, 0b00_110_000, 0xAA, 0xBB];
    // const RESERVED_OPCODE: &[u8] = &[0x0F, 0x04]; // 2-byte reserved opcode

    // fn create_push_imm16(fields: &HashMap<&str, FieldDefinition>) -> InstructionVariantDefinition {
    //     let mut push_imm_16 = InstructionVariantDefinition::new("push [imm16]");
    //     push_imm_16.add_field(fields, "opcode8", FieldValueRequirement::Specific(0xFF));
    //     push_imm_16.add_field(fields, "mod_rm", FieldValueRequirement::Any);
    //     push_imm_16.restrict_field(fields, "mod_rm.reg_opcode", FieldValueRequirement::Specific(0b110));
    //     push_imm_16.add_field(fields, "imm16", FieldValueRequirement::Any);
    //     push_imm_16
    // }

    // fn create_isa() -> (HashMap<&'static str, FieldDefinition>, InstructionVariantDefinition) {
    //     // regex pattern: "[prefix]* [opcode]{1,3} [modrm]? [sib]? [imm]? [disp]?";
    //     // prefix = 0x26, 0x36, 0x64, 0x65, 0x66, 0x67, 0xF0, 0xF2, 0xF3, 
    //     // opcode = 1-byte:                0x00 - 0x0E, 0x10 - 0xFF
    //     //          2-byte: 0x0F      then 0x00 - 0x37, 0x3B - 0xFF
    //     //          3-byte: 0x0F 0x38 then 0x00 - 0xFF
    //     //                  0x0F 0x3A then 0x00 - 0xFF
    //     // modrm pattern: "xx yyy zzz", x: mod, y: reg/opcode, z: reg/mem
    //     // sib pattern:   "xx yyy zzz", x: scale, y: index, z: base
    //     // imm pattern:   1, 2, 4, or 8 bytes
    //     // disp pattern:  1, 2, 4, or 8 bytes
        
    //     //let mut x86_isa = InstructionSet::new();

    //     let mut modrm = FieldDefinition::new("mod_rm", 8);
    //     let r#mod = FieldDefinition::new("mod", 2);
    //     let reg_opcode = FieldDefinition::new("reg_opcode", 3);
    //     let reg_memory = FieldDefinition::new("reg_memory", 3);
    //     modrm.add_subfield_definition(6..=7, r#mod);
    //     modrm.add_subfield_definition(3..=5, reg_opcode);
    //     modrm.add_subfield_definition(0..=2, reg_memory);

    //     let opcode8 = FieldDefinition::new("opcode8", 8);
    //     let imm16 = FieldDefinition::new("imm16", 16);

    //     let mut fields = HashMap::new();
    //     fields.insert("opcode8", opcode8);
    //     fields.insert("mod_rm", modrm);
    //     fields.insert("imm16", imm16);

    //     // x86_isa.add_field_definition("opcode8", opcode8);
    //     // x86_isa.add_field_definition("mod_rm", modrm);
    //     // x86_isa.add_field_definition("imm16", imm16);

    //     let push_imm_16 = create_push_imm16(&fields);

    //     //let mut push = InstructionDefinition::new("push");

    //     //push.add_variant_definition(push_imm_16);

    //     //x86_isa.add_instruction_definition(push);
    //     //x86_isa
    //     (fields, push_imm_16)
    // }

    // #[test]
    // fn reserved_opcode_fails() {
    //     let (fields, push_imm_16) = create_isa();
    //     println!("{:#?}", fields);
    //     println!("{:#?}", push_imm_16);
    //     assert_eq!(push_imm_16.try_disassemble(RESERVED_OPCODE), Err(DisassemblyError::UndefinedInstruction));
    // }

    // #[test]
    // fn push_imm_16() {
    //     let (fields, push_imm_16) = create_isa();
    //     println!("{:#?}", fields);
    //     println!("{:#?}", push_imm_16);
    //     assert!(push_imm_16.try_disassemble(PUSH_IMM_16).is_ok());
    // }
}
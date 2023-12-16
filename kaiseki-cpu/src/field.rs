use std::{ops::{Index, RangeInclusive, IndexMut}, collections::HashMap};

// use bitvec::prelude::*;

// use crate::error::{Result, DisassemblyError};

// REP (prefix):
// segment_override (prefix):

// MOV:
// PUSH:
// POP:
// XCHG:
// IN:
// OUT:
// LEA:
// LDS:
// LES:
// ADD:
// ADC:
// INC:
// SUB:
// SBB:
// DEC:
// NEG:
// CMP:
// MUL:
// IMUL:
// AAM:
// DIV:
// IDIV:
// AAD:
// NOT:
// SHL/SAL:
// SHR:
// SAR:
// ROL:
// ROR:
// RCL:
// RCR:
// AND:
// TEST:
// OR
// XOR:
// MOVS:
// CMPS:
// SCAS:
// LODS:
// STDS:
// CALL:
// JMP:
// RET (within seg adding immediate to SP):
// RET (intersegment adding immediate to SP):
// JE/JZ:
// JL/JNGE:
// JLE/JNG:
// JB/JNAE:
// JBE/JNA:
// JP/JPE:
// JO:
// JS:
// JNE/JNZ:
// JNL/JGE:
// JNLE/JG:
// JNB/JAE:
// JNBE/JA:
// JNP/JPO:
// JNO:
// JNS:
// LOOP:
// LOOPZ/LOOPE:
// LOOPNZ/LOOPNE:
// JCXZ:
// INT (type specified):
// ESC:

// LOCK (prefix): 0xF0
// XLAT: 0xD7
// LAHF: 0x9F
// SAHF: 0x9E
// PUSHF: 0x9C
// POPF: 0x9D
// AAA: 0x37
// DAA: 0x27
// AAS: 0x3F
// DAS: 0x2F
// CBW: 0x98
// CWD: 0x99
// RET (within segment): 0xC3
// RET (intersegment): 0xCB
// INT (Type 3): 0xCC
// INTO: 0xCE
// IRET: 0xCF
// CLC: 0xF8
// CMC: 0xF5
// STC: 0xF9
// CLD: 0xFC
// STD: 0xFD
// CLI: 0xFA
// STI: 0xFB
// HLT: 0xF4
// WAIT: 0x9B



// field opcode8: 8b:
//     field w: 1b: this[0]
//     field d: 1b: this[1]

// AAA: [opcode8 == 0x37]


#[derive(Clone, Debug)]
pub(crate) struct FieldDefinition {
    name: &'static str,
    bitrange: Option<RangeInclusive<usize>>,
    subfields: HashMap<&'static str, FieldDefinition>,
}

impl FieldDefinition {
    pub fn new(name: &'static str, bitrange: Option<RangeInclusive<usize>>) -> Self {
        Self { name, bitrange, subfields: HashMap::new() }
    }

    pub fn add_subfield(&mut self, field: FieldDefinition) {
        self.subfields.insert(field.name(), field);
    }

    pub fn get(&self, field_path: &str) -> Option<&FieldDefinition> {
        let path_elements: Vec<&str> = field_path.split(".").collect();
        let mut cur_path = String::new();
        let mut cur_field = self;
        for field_name in path_elements {
            cur_path.push_str(field_name);
            cur_path.push('.');
            if cur_field.subfields.contains_key(field_name) {
                cur_field = &cur_field.subfields.get(field_name).unwrap();
            } else {
                return None
            }
        }
        Some(cur_field)
    }

    pub fn get_mut(&mut self, field_path: &str) -> Option<&mut FieldDefinition> {
        let path_elements: Vec<&str> = field_path.split(".").collect();
        let mut cur_path = String::new();
        let mut cur_field = self;
        for field_name in path_elements {
            cur_path.push_str(field_name);
            cur_path.push('.');
            if cur_field.subfields.contains_key(field_name) {
                cur_field = cur_field.subfields.get_mut(field_name).unwrap();
            } else {
                return None
            }
        }
        Some(cur_field)
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Index<&'static str> for FieldDefinition {
    type Output = FieldDefinition;

    fn index(&self, index: &'static str) -> &Self::Output {
        self.get(index).expect(&format!("field definition '{}' does not exist", index))
    }
}

impl IndexMut<&'static str> for FieldDefinition {
    fn index_mut(&mut self, index: &'static str) -> &mut Self::Output {
        self.get_mut(index).expect(&format!("field definition '{}' does not exist", index))
    }
}

// impl Field {
//     pub fn try_disassemble(&self, data: &BitSlice<u8>) -> Result<()> {
//         let value: usize = data[0..self.width_bits].load();
//         if let Err(err) = self.constraint.matches(self.name, value) {
//             return Err(err);
//         }

//         for (bit_range, subfield) in self.subfields.values() {
//             let start = *bit_range.start();
//             let end = *bit_range.end();
//             let value_field = &data[start..=end];
//             match subfield.try_disassemble(value_field) {
//                 Ok(_) => continue,
//                 Err(err) => return Err(err),
//             }
//         }

//         Ok(())
//     }
// }


#[cfg(test)]
mod tests {
    // use bitvec::slice::BitSlice;

    use super::FieldDefinition;

    fn create_opcode8_field() -> FieldDefinition {
        let mut opcode8 = FieldDefinition::new("opcode8", None);
        let width = FieldDefinition::new("w", Some(0..=0));
        opcode8.add_subfield(width);
        opcode8
    }

    #[test]
    fn can_add_subfields() {
        let _ = create_opcode8_field();
    }

    #[test]
    fn index_works() {
        let field = create_opcode8_field();
        assert_eq!(field.name(), "AAA");
        let opcode8 = &field["opcode8"];
        assert_eq!(opcode8.name(), "opcode8");
        let width = &field["opcode8.w"];
        assert_eq!(width.name(), "w");
    }

    #[test]
    fn index_mut_works() {
        let mut field = create_opcode8_field();
        let opcode8 = &mut field["opcode8"];
        let direction = FieldDefinition::new("d", Some(1..=1));
        opcode8.add_subfield(direction);

        let new_direction = &field["opcode8.d"];
        assert_eq!(new_direction.name(), "d");
    }

    #[test]
    #[should_panic]
    fn index_panics_on_invalid_field_name() {
        let field = create_opcode8_field();
        let _ = &field["nonexistent"];
    }

    #[test]
    fn new_works() {
        let _ = FieldDefinition::new("field", None);
    }
}
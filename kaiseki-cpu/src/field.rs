use std::fmt;
use std::ops::RangeInclusive;

use bitvec::prelude::*;
use rangemap::RangeInclusiveMap;

use crate::error::{DisassemblyError, Result};

#[derive(Clone, Eq, PartialEq)]
pub(crate) enum Field {
    AnyValue(FieldDefinition),
    SpecificValue(FieldDefinition, usize),
    SingleRangeInclusiveValue(FieldDefinition, RangeInclusive<usize>),
    Subfields(FieldDefinition, RangeInclusiveMap<usize, Field>),
}

impl Field {
    pub fn add_subfield(&mut self, bit_range: RangeInclusive<usize>, subfield: Field) {
        match self {
            Field::Subfields(def, subfields) => subfields.insert(bit_range, subfield),
            _ => panic!("only Field::Subfields variants can call add_subfields()"),
        }
    }

    pub fn try_disassemble(&self, data: &BitSlice<u8>) -> Result<()> {
        match &self {
            Field::AnyValue(def) => {
                let field_value: usize = data[0..def.width_bits].load_le();
                println!("{} has value 0x{:X}, matches AnyValue", def.path, field_value);
                Ok(())
            }
            Field::SpecificValue(def, required_value) => {
                let field_value: usize = data[0..def.width_bits].load_le();
                if required_value == &field_value {
                    println!("{} has value 0x{:X}, matches Specific(0x{:X})", def.path, field_value, required_value);
                    Ok(())
                } else {
                    println!("{} has value 0x{:X}, does NOT match Specific(0x{:X})", def.path, field_value, required_value);
                    Err(DisassemblyError::UndefinedInstruction)
                }
            },
            Field::SingleRangeInclusiveValue(def, range) => {
                let field_value: usize = data[0..def.width_bits].load_le();
                if range.contains(&field_value) {
                    println!("{} has value 0x{:X}, matches SingleRangeInclusive({:?})", def.path, field_value, range);
                    Ok(())
                } else {
                    println!("{} has value 0x{:X}, does NOT match SingleRangeInclusive({:?})", def.path, field_value, range);
                    Err(DisassemblyError::UndefinedInstruction)
                }
            },
            Field::Subfields(def, subfields) => {
                let field_value: usize = data[0..def.width_bits].load_le();
                for (range, field) in subfields.iter() {
                    let subfield_bits = &data[range.clone()].to_bitvec();
                    if field.try_disassemble(subfield_bits).is_err() {
                        println!("{} has value 0x{:X}, does NOT match Subfields(..)", def.path, field_value);
                        return Err(DisassemblyError::UndefinedInstruction);
                    }
                }
                println!("{} has value 0x{:X}, matches Subfields(..)", def.path, field_value);
                Ok(())
            },
        }
    }

    pub fn width_bits(&self) -> usize {
        match &self {
            Field::AnyValue(def) => def.width_bits,
            Field::SpecificValue(def, _) => def.width_bits,
            Field::SingleRangeInclusiveValue(def, _) => def.width_bits,
            Field::Subfields(def, _) => def.width_bits,
        }
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AnyValue(def) => f.write_fmt(format_args!("{} == Any", def.name)),
            Self::SpecificValue(def, val) => f.write_fmt(format_args!("{} == 0x{:X}", def.name, val)),
            Self::SingleRangeInclusiveValue(def, range) => f.write_fmt(format_args!("0x{:X} <= {} <= 0x{:X}", range.start(), def.name, range.end())),
            Self::Subfields(def, subfields) => {
                let mut substr = String::new();
                for (range, subfield) in subfields.iter() {
                    substr.insert_str(0, &format!("[{:?}({}..{})]", subfield, range.end(), range.start()));
                }
                f.write_fmt(format_args!("{}.{}", def.name, substr))
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct FieldDefinition {
    name: String,
    path: String,
    pub width_bits: usize,
}

impl FieldDefinition {
    pub fn new(path: &str, width_bits: usize) -> Self {
        Self {
            name: String::from(path.split(".").last().unwrap_or(path)),
            path: String::from(path),
            width_bits,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}
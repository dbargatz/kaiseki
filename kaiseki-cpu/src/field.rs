use std::fmt;
use std::ops::RangeInclusive;

use bitvec::prelude::*;
use rangemap::RangeInclusiveMap;

use crate::error::{DisassemblyError, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Field {
    AnyValue,
    SpecificValue(usize),
    SingleRangeInclusiveValue(RangeInclusive<usize>),
    Subfields(RangeInclusiveMap<usize, FieldDefinition>),
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct FieldDefinition {
    name: String,
    path: String,
    pub width_bits: usize,
    pub valid_values: Field,
}

impl FieldDefinition {
    pub fn new(path: &str, width_bits: usize, valid_values: Field) -> Self {
        Self {
            name: String::from(path.split(".").last().unwrap_or(path)),
            path: String::from(path),
            width_bits,
            valid_values,
        }
    }

    pub fn add_subfield_definition(&mut self, bits: RangeInclusive<usize>, mut subfield: FieldDefinition) {
        // TODO: sanity checking on added field wrt parent field size
        // TODO: sanity checking on added field wrt start/end bits < parent field size
        // TODO: sanity checking on added field wrt no overlaps with other subfields
        subfield.path = format!("{}.{}", self.path, subfield.path);
        match &mut self.valid_values {
            Field::Subfields(subfields) => {
                subfields.insert(bits, subfield);
            }
            _ => {
                let mut subfields = RangeInclusiveMap::new();
                subfields.insert(bits, subfield);
                self.valid_values = Field::Subfields(subfields);
            }
        };
    }

    pub fn try_disassemble(&self, data: &BitSlice<u8>) -> Result<()> {
        let field_value: usize = data[0..self.width_bits].load_le();
        match &self.valid_values {
            Field::AnyValue => {
                println!("{} has value {}, matches AnyValue", self.path, field_value);
                Ok(())
            }
            Field::SpecificValue(required_value) => {
                if required_value == &field_value {
                    println!("{} has value {}, matches Specific({})", self.path, field_value, required_value);
                    Ok(())
                } else {
                    println!("{} has value {}, does NOT match Specific({})", self.path, field_value, required_value);
                    Err(DisassemblyError::UndefinedInstruction)
                }
            },
            Field::SingleRangeInclusiveValue(range) => {
                if range.contains(&field_value) {
                    println!("{} has value {}, matches SingleRangeInclusive({:?})", self.path, field_value, range);
                    Ok(())
                } else {
                    println!("{} has value {}, does NOT match SingleRangeInclusive({:?})", self.path, field_value, range);
                    Err(DisassemblyError::UndefinedInstruction)
                }
            },
            Field::Subfields(subfields) => {
                for (range, field) in subfields.iter() {
                    let subfield_bits = &data[range.clone()].to_bitvec();
                    if field.try_disassemble(subfield_bits).is_err() {
                        println!("{} has value {}, does NOT match Subfields(..)", self.path, field_value);
                        return Err(DisassemblyError::UndefinedInstruction);
                    }
                }
                println!("{} has value {}, matches Subfields(..)", self.path, field_value);
                Ok(())
            },
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}

impl fmt::Debug for FieldDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.valid_values {
            Field::AnyValue => f.write_fmt(format_args!("{} == Any", self.name)),
            Field::SpecificValue(val) => f.write_fmt(format_args!("{} == 0x{:X}", self.name, val)),
            Field::SingleRangeInclusiveValue(range) => f.write_fmt(format_args!("0x{:X} <= {} <= 0x{:X}", range.start(), self.name, range.end())),
            Field::Subfields(subfields) => {
                let mut substr = String::new();
                for (range, subfield) in subfields.iter() {
                    substr.insert_str(0, &format!("[{:?}({}..{})]", subfield, range.end(), range.start()));
                }
                f.write_fmt(format_args!("{}.{}", self.name, substr))
            }
        }
    }
}
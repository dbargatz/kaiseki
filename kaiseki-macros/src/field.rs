use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, bracketed, Ident, PatRange, Token, Type};

pub struct FieldDefinitionList {
    root_fields: Vec<FieldDefinition>,
}

impl Parse for FieldDefinitionList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut root_fields = Vec::new();

        while !input.is_empty() {
            let punc = input.parse_terminated(FieldDefinition::parse, Token![,])?;
            punc.into_pairs().for_each(|pair| {
                root_fields.push(pair.into_value());
            });
        }

        Ok(FieldDefinitionList { root_fields })
    }
}

impl ToTokens for FieldDefinitionList {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.root_fields {
            field.to_tokens(tokens);
        }
    }
}

// NAME => syn::Ident
//  - required
// TYPE => syn::Type
//  - required
//  - unknowns:
//    - what types should this be restricted to, if any? just u8/u16/u32/u64/u128, or...?
// RANGE => [ syn::PatRange ]
//  - forbidden for root fields, required for subfields that don't have a block
//  - only valid for child fields of a parent field
//  - indices must fit within parent field type (i.e. 0..=15 is not OK for a u8 parent field)
//  - okay for sibling fields to overlap partially or completely
// SUBFIELDS => { <FieldDefinition ,>* }
//  - optional

// syntax of the form:
// - `name: type`                     (top-level only)
// - `name: type = { ... }`           (top-level or subfield)
// - `name: type = self.value[X..Y]`  (subfield only)
// - `name: type = self.value[X..=Y]` (subfield only)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldDefinition {
    name: Ident,
    typ: Type,
    range: Option<PatRange>,
    subfields: Vec<FieldDefinition>,
}

impl Parse for FieldDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse out the field name and type, which are required.
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let typ = input.parse()?;

        // We have a few options for what comes next:
        //   - nothing
        //   - a full range pattern: `= $[X..Y]` or `= $[X..=Y]`
        //   - an extractor block: `= |name: type| { ... }`
        //   - a block of subfields: `{ ... }`
        let mut range = None;
        if input.peek(Token![=]) {
            // Must be a range or extractor block.
            input.parse::<Token![=]>()?;

            if input.peek(syn::token::Dollar) && input.peek2(syn::token::Bracket) {
                input.parse::<Token![$]>()?;
                let content;
                let _ = bracketed!(content in input);
                range = Some(content.parse()?);
            } else if input.peek(syn::token::Or) {
                // TODO: Extractor block
            }
        }

        let mut subfields = Vec::new();
        if input.peek(syn::token::Brace) {
            let content;
            let _ = braced!(content in input);

            while !content.is_empty() {
                let punc = content.parse_terminated(FieldDefinition::parse, Token![,])?;
                punc.into_pairs().for_each(|pair| {
                    subfields.push(pair.into_value());
                });
            }
        }

        Ok(FieldDefinition {
            name,
            typ,
            range,
            subfields,
        })
    }
}

impl ToTokens for FieldDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let typ = &self.typ;
        let range = &self.range;
        let docstring = format!(
            "/// Range: {}",
            range
                .as_ref()
                .map_or("None".to_string(), |r| r.to_token_stream().to_string()),
        );

        let mut extractors = TokenStream2::new();
        for subfield in &self.subfields {
            let sf_name = &subfield.name;
            let sf_type = &subfield.typ;
            let sf_mask: u16 = 0xFF;
            extractors.extend(quote! {
                pub fn #sf_name(&self) -> #sf_type {
                    (self.value & #sf_mask) as #sf_type
                }
            });
        }

        tokens.extend(quote! {
            #[doc = #docstring]
            pub struct #name {
                value: #typ,
            }

            impl #name {
                pub fn new(value: #typ) -> Self {
                    Self { value }
                }

                pub fn value(&self) -> #typ {
                    self.value
                }

                #extractors
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    fn validate_field(
        field: &FieldDefinition,
        name: &str,
        typ: &str,
        range: Option<TokenStream2>,
        num_subfields: usize,
    ) {
        assert_eq!(field.name.to_string(), name);
        assert_eq!(field.typ.to_token_stream().to_string(), typ);

        assert_eq!(
            field
                .range
                .as_ref()
                .map(|r| r.to_token_stream().to_string()),
            range.as_ref().map(|r| r.to_string())
        );
        assert_eq!(field.subfields.len(), num_subfields);
    }

    #[test]
    fn test_parse_field_definition_basic() {
        let input = quote! {
            BasicField: u8
        };
        let field = syn::parse2::<FieldDefinition>(input).unwrap();
        validate_field(&field, "BasicField", "u8", None, 0);
    }

    #[test]
    fn test_parse_field_definition_with_range_exclusive() {
        let input = quote! {
            RangeField: u8 = $[0..3]
        };
        let field = syn::parse2::<FieldDefinition>(input).unwrap();
        validate_field(&field, "RangeField", "u8", Some(quote! { 0..3 }), 0);
    }

    #[test]
    fn test_parse_field_definition_with_range_inclusive() {
        let input = quote! {
            RangeInclusiveField: u8 = $[0..=3]
        };
        let field = syn::parse2::<FieldDefinition>(input).unwrap();
        validate_field(
            &field,
            "RangeInclusiveField",
            "u8",
            Some(quote! { 0..=3 }),
            0,
        );
    }

    #[test]
    fn test_parse_field_definition_with_subfields() {
        let input = quote! {
            FieldWithSubfields: u32 {
                SubBasic: u8, // is this ever valid as a subfield? what does this mean?
                SubRange: u8 = $[0..4], // becomes FieldWithSubfields.SubRange() -> u8 { (self.value & 0x0000000F) as u8 }
                SubRangeInclusive: u8 = $[4..=7],  // becomes FieldWithSubfields.SubRangeInclusive() -> u8 { ((self.value & 0x000000F0) >> 4) as u8 }
                SubWithSubfields: u16 = $[8..=23] {
                    SubSub1: u8 = $[0..8],
                    SubSub2: u8 = $[8..=15],
                },
                SubTop: u8 = $[24..=31],
            }
        };
        let field = syn::parse2::<FieldDefinition>(input).unwrap();
        validate_field(&field, "FieldWithSubfields", "u32", None, 5);
        validate_field(&field.subfields[0], "SubBasic", "u8", None, 0);
        validate_field(
            &field.subfields[1],
            "SubRange",
            "u8",
            Some(quote! { 0..4 }),
            0,
        );
        validate_field(
            &field.subfields[2],
            "SubRangeInclusive",
            "u8",
            Some(quote! { 4..=7 }),
            0,
        );
        validate_field(
            &field.subfields[3],
            "SubWithSubfields",
            "u16",
            Some(quote! { 8..=23 }),
            2,
        );
        validate_field(
            &field.subfields[3].subfields[0],
            "SubSub1",
            "u8",
            Some(quote! { 0..8 }),
            0,
        );
        validate_field(
            &field.subfields[3].subfields[1],
            "SubSub2",
            "u8",
            Some(quote! { 8..=15 }),
            0,
        );
        validate_field(
            &field.subfields[4],
            "SubTop",
            "u8",
            Some(quote! { 24..=31 }),
            0,
        );
    }
}
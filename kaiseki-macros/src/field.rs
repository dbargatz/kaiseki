use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, bracketed, Block, Ident, PatRange, Stmt, Token, Type};

pub struct FieldDefinitionList {
    root_fields: Vec<RootFieldDefinition>,
}

impl Parse for FieldDefinitionList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut root_fields = Vec::new();

        while !input.is_empty() {
            let punc = input.parse_terminated(RootFieldDefinition::parse, Token![,])?;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootFieldDefinition {
    name: Ident,
    typ: Type,
    subfields: Vec<SubfieldDefinition>,
}

impl Parse for RootFieldDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse out the field name and type, which are required.
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let typ = input.parse()?;

        let content;
        let _ = braced!(content in input);
        let mut subfields = Vec::new();
        while !content.is_empty() {
            let punc = content.parse_terminated(SubfieldDefinition::parse, Token![,])?;
            punc.into_pairs().for_each(|pair| {
                subfields.push(pair.into_value());
            });
        }

        Ok(RootFieldDefinition {
            name,
            typ,
            subfields,
        })
    }
}

impl ToTokens for RootFieldDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let typ = &self.typ;
        let docstring = format!("/// TODO: Document this field");

        let mut trait_tokens = TokenStream2::new();
        let mut subfield_tokens = TokenStream2::new();
        for field in &self.subfields {
            match field {
                SubfieldDefinition::Range { name, typ, var_name: _, range: _ } => {
                    trait_tokens.extend(quote! {
                        fn #name(&self) -> #typ;
                    });
                },
                SubfieldDefinition::Extractor { name, typ, brace_token: _, stmts: _ } => {
                    trait_tokens.extend(quote! {
                        fn #name(&self) -> #typ;
                    });
                },
            }
            field.to_tokens(&mut subfield_tokens);
        }

        tokens.extend(quote! {
            #[doc = #docstring]
            pub trait #name {
                #trait_tokens
            }

            impl #name for #typ {
                #subfield_tokens
            }
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubfieldDefinition {
    Range {
        name: Ident,
        typ: Type,
        var_name: Ident,
        range: PatRange,
    },
    Extractor {
        name: Ident,
        typ: Type,
        brace_token: Brace,
        stmts: Vec<Stmt>,
    },
}

impl Parse for SubfieldDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let typ = input.parse()?;

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let var_name = input.parse::<Ident>()?;
            let content;
            let _ = bracketed!(content in input);
            let range = content.parse()?;

            Ok(SubfieldDefinition::Range {
                name,
                typ,
                var_name,
                range,
            })
        } else {
            let content;
            let brace_token = braced!(content in input);
            let stmts = content.call(Block::parse_within)?;

            Ok(SubfieldDefinition::Extractor {
                name,
                typ,
                brace_token,
                stmts,
            })
        } 
    }
}

impl ToTokens for SubfieldDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            SubfieldDefinition::Range { name, typ, var_name: _, range: _ } => {
                let mask = 0xFFu16;
                tokens.extend(quote! {
                    fn #name(&self) -> #typ {
                        (self.value & #mask) as #typ
                    }
                });
            },
            SubfieldDefinition::Extractor { name, typ, brace_token: _, stmts } => {
                tokens.extend(quote! {
                    fn #name(&self) -> #typ {
                        #(#stmts)*
                    }
                });
            },
        }
    }
}




// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct FieldDefinition {
//     name: Ident,
//     typ: Type,
//     range: Option<PatRange>,
//     extractor: Option<ExtractorDefinition>,
//     subfields: Vec<FieldDefinition>,
// }

// impl Parse for FieldDefinition {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         // Parse out the field name and type, which are required.
//         let name = input.parse()?;
//         input.parse::<Token![:]>()?;
//         let typ = input.parse()?;

//         input.parse::<Token![=]>()?;

//         // We have a few options for what comes next:
//         //   - a full range pattern: `= $[X..Y]` or `= $[X..=Y]`
//         //   - an extractor block: `= |name| { ... }`
//         let mut range = None;
//         let mut extractor = None;
//         let mut subfields = Vec::new();
//         if input.peek(Token![|]) {
//             input.parse::<Token![|]>()?;
//             // TODO: need to pass the closure variable name to subfields; just using "op" for now
//             let _inner_name = input.parse::<Ident>()?;
//             input.parse::<Token![|]>()?;

//             let content;
//             let _ = braced!(content in input);
//             while !content.is_empty() {
//                 let punc = content.parse_terminated(FieldDefinition::parse, Token![,])?;
//                 punc.into_pairs().for_each(|pair| {
//                     subfields.push(pair.into_value());
//                 });
//             }
//         } else if input.peek(syn::token::Brace) {
//             extractor = Some(input.parse_terminated(ExtractorDefinition::parse, Token![,])?);
//         } else {
//             let var_name = input.parse::<Ident>()?;
//             // TODO: need to track the closure variable name for use with subfields; just using "op" for now
//             if var_name != "op" {
//                 return Err(syn::Error::new(
//                     var_name.span(),
//                     "Expected `op` as the variable name for a range pattern",
//                 ));
//             }

//             let content;
//             let _ = bracketed!(content in input);
//             range = Some(content.parse()?);
//         }

//         Ok(FieldDefinition {
//             name,
//             typ,
//             range,
//             extractor,
//             subfields,
//         })
//     }
// }

// impl ToTokens for FieldDefinition {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         let name = &self.name;
//         let typ = &self.typ;
//         let range = &self.range;
//         let docstring = format!(
//             "/// Range: {}",
//             range
//                 .as_ref()
//                 .map_or("None".to_string(), |r| r.to_token_stream().to_string()),
//         );

//         let mut extractors = TokenStream2::new();
//         for subfield in &self.subfields {
//             let sf_name = &subfield.name;
//             let sf_type = &subfield.typ;
//             if let Some(sf_extractor) = &self.extractor {
//                 extractors.extend(quote! {
//                     pub fn #sf_name(&self) -> #sf_type {
//                         #sf_extractor
//                     }
//                 });
//             } else {
//                 let sf_mask: u16 = 0xFF;
//                 extractors.extend(quote! {
//                     pub fn #sf_name(&self) -> #sf_type {
//                         (self.value & #sf_mask) as #sf_type
//                     }
//                 });
//             }
//         }

//         tokens.extend(quote! {
//             #[doc = #docstring]
//             pub struct #name {
//                 value: #typ,
//             }

//             impl #name {
//                 pub fn new(value: #typ) -> Self {
//                     Self { value }
//                 }

//                 pub fn value(&self) -> #typ {
//                     self.value
//                 }

//                 #extractors
//             }
//         });
//     }
// }


// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct ExtractorDefinition {
//     brace_token: syn::token::Brace,
//     stmts: Vec<syn::Stmt>,
// }

// impl Parse for ExtractorDefinition {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let content;
//         let brace_token = braced!(content in input);
//         let mut stmts = Vec::new();
//         while !content.is_empty() {
//             stmts.push(content.parse()?);
//         }

//         Ok(ExtractorDefinition { brace_token, stmts })
//     }
// }

// impl ToTokens for ExtractorDefinition {
//     fn to_tokens(&self, tokens: &mut TokenStream2) {
//         let stmts = &self.stmts;
//         tokens.extend(quote! {
//             #(#stmts)*
//         });
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use pretty_assertions::assert_eq;

//     fn validate_field(
//         field: &FieldDefinition,
//         name: &str,
//         typ: &str,
//         range: Option<TokenStream2>,
//         num_subfields: usize,
//     ) {
//         assert_eq!(field.name.to_string(), name);
//         assert_eq!(field.typ.to_token_stream().to_string(), typ);

//         assert_eq!(
//             field
//                 .range
//                 .as_ref()
//                 .map(|r| r.to_token_stream().to_string()),
//             range.as_ref().map(|r| r.to_string())
//         );
//         assert_eq!(field.subfields.len(), num_subfields);
//     }

//     #[test]
//     fn test_parse_field_definition_with_subfields() {
//         let input = quote! {
//             FieldWithSubfields: u32 = |op| {
//                 SubRange: u8 = op[0..4], // becomes FieldWithSubfields.SubRange() -> u8 { (self.value & 0x0000000F) as u8 }
//                 SubRangeInclusive: u8 = op[4..=7],  // becomes FieldWithSubfields.SubRangeInclusive() -> u8 { ((self.value & 0x000000F0) >> 4) as u8 }
//                 SubWithSubfields: u16 = |op| {
//                     SubSub1: u8 = op[0..8],
//                     SubSub2: u8 = op[8..=15],
//                 },
//                 SubTop: u8 = op[24..=31],
//             }
//         };
//         let field = syn::parse2::<FieldDefinition>(input).unwrap();
//         validate_field(&field, "FieldWithSubfields", "u32", None, 4);
//         validate_field(
//             &field.subfields[0],
//             "SubRange",
//             "u8",
//             Some(quote! { 0..4 }),
//             0,
//         );
//         validate_field(
//             &field.subfields[1],
//             "SubRangeInclusive",
//             "u8",
//             Some(quote! { 4..=7 }),
//             0,
//         );
//         validate_field(&field.subfields[2], "SubWithSubfields", "u16", None, 2);
//         validate_field(
//             &field.subfields[2].subfields[0],
//             "SubSub1",
//             "u8",
//             Some(quote! { 0..8 }),
//             0,
//         );
//         validate_field(
//             &field.subfields[2].subfields[1],
//             "SubSub2",
//             "u8",
//             Some(quote! { 8..=15 }),
//             0,
//         );
//         validate_field(
//             &field.subfields[3],
//             "SubTop",
//             "u8",
//             Some(quote! { 24..=31 }),
//             0,
//         );
//     }
// }

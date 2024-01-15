use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, Type};

pub struct RegisterDefinitionList {
    pub registers: Vec<RegisterDefinition>,
}

impl Parse for RegisterDefinitionList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut registers = Vec::new();

        while !input.is_empty() {
            let punc = input.parse_terminated(RegisterDefinition::parse, Token![,])?;
            punc.into_pairs().for_each(|pair| {
                registers.push(pair.into_value());
            });
        }

        Ok(RegisterDefinitionList { registers })
    }
}

impl ToTokens for RegisterDefinitionList {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let regs = &self.registers;
        let enum_variants = regs.iter().map(|reg| reg.get_enum_variant());
        let index_matches = regs.iter().map(|reg| reg.get_index_match());
        let struct_fields = regs.iter().map(|reg| reg.get_struct_field());
        tokens.extend(quote! {
            pub mod registers {
                use kaiseki_core::register::Register;

                #[allow(non_snake_case)]
                pub enum RegisterId {
                    #(#enum_variants),*,
                }

                impl RegisterId {
                    pub fn get_by_index(index: u8) -> RegisterId {
                        match index {
                            #(#index_matches),*,
                            _ => panic!("Invalid register index: {}", index),
                        }
                    }
                }

                #[allow(non_snake_case)]
                pub struct RegisterSet {
                    #(#struct_fields),*
                }
            }
        })
    }
}

pub struct RegisterDefinition {
    name: Ident,
    typ: Type,
}

impl RegisterDefinition {
    pub fn get_enum_variant(&self) -> TokenStream2 {
        let name = &self.name;
        quote! {
            #name
        }
    }

    pub fn get_index_match(&self) -> TokenStream2 {
        let name = &self.name;
        quote! {
            #name => RegisterId::#name
        }
    }

    pub fn get_struct_field(&self) -> TokenStream2 {
        let name = &self.name;
        let typ = &self.typ;
        quote! {
            pub #name: Register<#typ>
        }
    }
}

impl Parse for RegisterDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let typ = input.parse()?;

        Ok(RegisterDefinition { name, typ })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_register_definition() {
        let input = quote! {
            V0: u8
        };

        let reg = syn::parse2::<RegisterDefinition>(input).unwrap();
        assert_eq!(reg.name.to_string(), "V0");
        assert_eq!(reg.typ.to_token_stream().to_string(), "u8");
    }

    #[test]
    fn test_parse_register_definition_list() {
        let input = quote! {
            V0: u8,
            V1: u8,
            V2: u8,
            VI: u16,
            PC: u16,
            SP: u8,
        };

        let regs = syn::parse2::<RegisterDefinitionList>(input).unwrap();
        let mut expected_id = 0;
        for reg in &regs.registers {
            match expected_id {
                0 | 1 | 2 => {
                    assert_eq!(reg.name.to_string(), format!("V{}", expected_id));
                    assert_eq!(reg.typ.to_token_stream().to_string(), "u8");
                }
                3 => {
                    assert_eq!(reg.name.to_string(), "VI");
                    assert_eq!(reg.typ.to_token_stream().to_string(), "u16");
                }
                4 => {
                    assert_eq!(reg.name.to_string(), "PC");
                    assert_eq!(reg.typ.to_token_stream().to_string(), "u16");
                }
                5 => {
                    assert_eq!(reg.name.to_string(), "SP");
                    assert_eq!(reg.typ.to_token_stream().to_string(), "u8");
                }
                _ => panic!("unexpected register name: {}", reg.name),
            }

            expected_id += 1;
        }
    }
}

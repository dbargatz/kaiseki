use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token};

pub struct InstructionDefinitionList {
    pub fields: Vec<InstructionDefinition>,
}

impl Parse for InstructionDefinitionList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut fields = Vec::new();
        while !input.is_empty() {
            fields.push(input.parse()?);
        }
        Ok(InstructionDefinitionList { fields })
    }
}

impl ToTokens for InstructionDefinitionList {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.fields {
            field.to_tokens(tokens);
        }
    }
}

pub struct InstructionDefinition {
    pub name: Ident,
    pub mnemonic: syn::LitStr,
    pub valid_opcodes: Vec<syn::PatRange>,
}

impl Parse for InstructionDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let mnemonic = input.parse()?;
        input.parse::<Token![,]>()?;
        let valid_opcodes = input.parse_terminated(syn::PatRange::parse, Token![,])?;
        Ok(InstructionDefinition {
            name,
            mnemonic,
            valid_opcodes: valid_opcodes.into_iter().collect(),
        })
    }
}

impl ToTokens for InstructionDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let mnemonic = &self.mnemonic;
        let _opcodes = &self.valid_opcodes;

        let struct_name = format_ident!("{}Instruction", name);
        tokens.extend(quote! {
            pub struct #struct_name {
                mnemonic: #mnemonic,
            }

            impl kaiseki_core::arch::instruction::Instruction for #struct_name {
                fn mnemonic(&self) -> &str {
                    &self.mnemonic
                }
            }
        });
    }
}

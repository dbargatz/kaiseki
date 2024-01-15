use proc_macro::TokenStream;
use quote::quote;

mod field;
use field::FieldDefinitionList;

mod register;
use register::RegisterDefinitionList;

#[proc_macro]
pub fn fields(input: TokenStream) -> TokenStream {
    let fields = syn::parse_macro_input!(input as FieldDefinitionList);
    TokenStream::from(quote!(#fields))
}

#[proc_macro]
pub fn registers(input: TokenStream) -> TokenStream {
    let regs = syn::parse_macro_input!(input as RegisterDefinitionList);
    TokenStream::from(quote!(#regs))
}

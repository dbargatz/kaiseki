use proc_macro::TokenStream;
use quote::quote;

mod field;
use field::FieldDefinitionList;

#[proc_macro]
pub fn fields(input: TokenStream) -> TokenStream {
    let fields = syn::parse_macro_input!(input as FieldDefinitionList);
    TokenStream::from(quote!(#fields))
}

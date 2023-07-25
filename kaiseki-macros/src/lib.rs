use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
// use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, Ident, ItemStruct, Token, ImplItemFn, Attribute};
use syn::{fold::Fold, parse_macro_input, Field};

fn get_attribute_name(attr: &syn::Attribute) -> Option<String> {
    attr.meta
        .path()
        .segments
        .first()
        .map(|s| s.ident.to_string())
}

fn remove_attribute(name: &str, field: &mut Field) -> Option<syn::Attribute> {
    fn attr_matches(name: &str, attr: &syn::Attribute) -> bool {
        if let Some(cur_name) = get_attribute_name(attr) {
            name == cur_name
        } else {
            false
        }
    }

    if let Some(idx) = field.attrs.iter().position(|a| attr_matches(name, a)) {
        Some(field.attrs.remove(idx))
    } else {
        None
    }
}

struct ComponentFolder {
    pub handlers: std::collections::HashMap<syn::Ident, TokenStream2>,
}

// impl syn::parse::Parse for ComponentFolder {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let di = syn::DeriveInput::parse(input)?;
//         Ok(Self { state: di })
//     }
// }

impl Fold for ComponentFolder {
    fn fold_item_struct(&mut self, i: syn::ItemStruct) -> syn::ItemStruct {
        eprintln!("item struct: {}", i.ident);
        syn::fold::fold_item_struct(self, i)
    }

    fn fold_impl_item(&mut self, i: syn::ImplItem) -> syn::ImplItem {
        eprintln!("impl item: {:?}", i);
        syn::fold::fold_impl_item(self, i)
    }

    fn fold_fields(&mut self, i: syn::Fields) -> syn::Fields {
        let mut new_fields = i;
        for field in new_fields.iter_mut() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            if let Some(handler_attr) = remove_attribute("handler", field) {
                let handler_id: syn::Ident = handler_attr.parse_args().unwrap();
                let wrapper_name = format!("{}_wrapper", handler_id);
                let wrapper_id = syn::Ident::new(&wrapper_name, handler_id.span());
                eprintln!("field {} is handled by {:#?}", field_name, &handler_id);
                let call_expr = quote! {
                    pub fn #wrapper_id (&mut self, msg: AddMessage) -> usize {
                        self.#handler_id (msg.a, msg.b)
                    }
                };
                self.handlers.insert(handler_id, call_expr);
            }
        }
        syn::fold::fold_fields(self, new_fields)
    }
}

#[proc_macro_attribute]
pub fn component(_attr_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident.clone();
    let mut fold = ComponentFolder {
        handlers: std::collections::HashMap::new(),
    };
    let item = fold.fold_item_struct(item);
    let mut initial_stream = quote!();
    for v in fold.handlers.values() {
        initial_stream.extend(v.clone());
    }

    TokenStream::from(quote! {
        #item

        impl #item_name {
            #initial_stream
        }
    })
}

#[proc_macro_derive(Component2, attributes(handler))]
pub fn derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::ItemStruct);
    let mut fold = ComponentFolder {
        handlers: std::collections::HashMap::new(),
    };
    let item = fold.fold_item_struct(item);
    // let struct_name = &item.ident;
    // eprintln!("name: {} {{", struct_name.to_string());
    // match &item.data {
    //     Data::Struct(data) => {
    //         for field in data.fields.iter() {
    //             let name = field.ident.as_ref().unwrap().to_string();
    //             let has_handler = has_attribute("handler", field);
    //             if has_handler {
    //                 eprintln!("  HANDLER field: {}", name);
    //             } else {
    //                 eprintln!("  field: {}", name);
    //             }
    //         }
    //     },
    //     _ => unimplemented!("only structs so far"),
    // }
    // eprintln!("}}");

    TokenStream::from(quote! {
        #item
    })
}

// struct ComponentAttributeArgs {
//     message_type: Ident,
// }

// impl Parse for ComponentAttributeArgs {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
//         Ok(ComponentAttributeArgs {
//             message_type: vars.first().unwrap().clone(),
//         })
//     }
// }

// #[proc_macro_attribute]
// pub fn component(attr_args: TokenStream, item: TokenStream) -> TokenStream {
//     // let args = parse_macro_input!(attr_args as ComponentAttributeArgs);
//     // let mt = args.message_type;

//     let input = parse_macro_input!(item as ItemStruct);
//     let struct_name = &input.ident;
//     let ref_name = Ident::new(format!("{}Ref", struct_name.to_string()).as_str(), struct_name.span());

//     let handler_fields = input.fields.iter().filter(|f| f.attrs.iter().filter(|a| a.style.))
//     let output = quote! {
//         #input

//         impl #struct_name {
//             // #[tracing::instrument]
//             pub fn create() -> #ref_name {
//                 let (send, recv) = tokio::sync::mpsc::channel(8);
//                 let mut component = #struct_name {
//                     invocations: 0,
//                     request_receiver: recv,
//                 };
//                 tokio::spawn(async move { component.run().await });
//                 #ref_name {
//                     request_sender: send,
//                 }
//             }

//             // #[tracing::instrument]
//             pub async fn run(&mut self) {
//                 while let Some(msg) = self.request_receiver.recv().await {
//                     // tracing::info!("received message: {:?}", msg);
//                     match msg {
//                         SampleMessage::Double {
//                             num,
//                             response_sender,
//                         } => {
//                             let response = self.double(num);
//                             // tracing::info!("response is {:?}", response);
//                             response_sender.send(response).unwrap();
//                         }
//                     }
//                 }
//             }
//         }

//         pub struct #ref_name {
//             request_sender: tokio::sync::mpsc::Sender<#mt>
//         }

//         impl #ref_name {
//             pub async fn double(&self, num: usize) -> usize {
//                 let (send, recv) = oneshot::channel::<usize>();
//                 let msg = SampleMessage::Double {
//                     num,
//                     response_sender: send,
//                 };

//                 if let Err(err) = self.request_sender.send(msg).await {
//                     panic!("could not send request message to component: {:?}", err);
//                 };

//                 match recv.await {
//                     Ok(val) => val,
//                     Err(err) => {
//                         panic!("could not receive response message from component: {:?}", err);
//                     }
//                 }
//             }
//         }
//     };

//     output.into()
// }

// #[proc_macro_attribute]
// pub fn handler(_attr_args: TokenStream, item: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(item as ImplItemFn);
//     eprintln!("fn: {:#?}", input.sig);
//     TokenStream::from(quote! { #input })
// }

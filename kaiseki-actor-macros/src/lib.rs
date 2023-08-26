use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, ItemImpl};

#[allow(dead_code)]
fn unparse(input: &TokenStream) -> syn::Result<String> {
    let file = syn::parse_str(&input.to_string())?;
    Ok(prettyplease::unparse(&file))
}

#[proc_macro_derive(Actor)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(input as DeriveInput);
    TokenStream::new()
}

fn get_ident_from_type(ty: &syn::Type) -> Option<syn::Ident> {
    if let syn::Type::Path(p) = ty {
        return p.path.get_ident().cloned();
    }
    None
}

fn get_method(item: &syn::ImplItem) -> Option<syn::ImplItemFn> {
    if let syn::ImplItem::Fn(f) = item {
        if f.sig.receiver().is_some() {
            // is a method, NOT an assoc fn
            return Some(f.clone());
        }
    }
    None
}

enum ActorTypeArg {
    Thread,
    Tokio,
}

impl syn::parse::Parse for ActorTypeArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // If no actor type argument was provided, default to "thread".
        let actor_str = if input.is_empty() {
            String::from("thread")
        } else {
            syn::Ident::parse(input)?.to_string()
        };

        match actor_str.as_str() {
            "thread" => Ok(ActorTypeArg::Thread),
            "tokio" => Ok(ActorTypeArg::Tokio),
            _ => Err(input.error(format!("unknown actor type '{}'", actor_str))),
        }
    }
}

struct AttributeArgs {
    actor_type: proc_macro2::TokenStream,
    handle_type: proc_macro2::TokenStream,
    use_async: bool,
}

impl syn::parse::Parse for AttributeArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let actor_type_arg: ActorTypeArg = input.parse()?;
        let parsed_args = match actor_type_arg {
            ActorTypeArg::Thread => AttributeArgs {
                actor_type: quote!(kaiseki_actor::ThreadActor),
                handle_type: quote!(kaiseki_actor::ThreadActorHandle),
                use_async: false,
            },
            ActorTypeArg::Tokio => AttributeArgs {
                actor_type: quote!(kaiseki_actor::TokioActor),
                handle_type: quote!(kaiseki_actor::TokioActorHandle),
                use_async: true,
            },
        };
        Ok(parsed_args)
    }
}

#[proc_macro_attribute]
pub fn actor(attr_args: TokenStream, item: TokenStream) -> TokenStream {
    // Determine the actor and handle type from the args passed to the attribute.
    let args = parse_macro_input!(attr_args as AttributeArgs);
    let actor_type = args.actor_type;
    let handle_type = args.handle_type;

    // TODO: Allow this macro to be used on struct def AND impl block, fork to different handlers based on type
    // TODO: Rename original struct type as "_<Name>InnerState", must match impl type
    // TODO: Rename original impl type as "_<Name>InnerState", must match struct type
    // TODO: Generate correct Actor type wrapping "_<Name>InnerState", name with original type name "<Name>"
    //       - figure out new() situation - what if original type has new()? what if it doesn't?
    // TODO: Generate correct ActorHandle wrapping "_<Name>InnerState", name as "<Name>Handle"
    //       - derive Debug
    //       - with Clone impl
    // TODO: Fix enum variant names, convert to camel-case
    // TODO: Figure out sync and async methods on ActorHandles. For each original method:
    //       - what if original method is sync? should matching passthrough method on ActorHandle be sync or async?
    //       - what if original method is async? should matching passthrough method on ActorHandle be sync or async?

    // TODO: Figure out error situation to give good compile-time errors
    let item = parse_macro_input!(item as ItemImpl);
    let item_type = &item.self_ty;
    let methods = item.items.iter().filter_map(get_method);
    let type_name = get_ident_from_type(item_type).expect("impl type had no identifier");
    let actor_name = format_ident!("{}Actor", type_name);
    let enum_name = format_ident!("_{}ActorEnum", type_name);
    let handle_name = format_ident!("{}ActorHandle", type_name);
    let trait_name = format_ident!("{}ActorTrait", type_name);
    let mut variants = Vec::new();
    let mut matchers = Vec::new();
    let mut handle_methods = Vec::new();
    let mut handle_method_sigs = Vec::new();
    for method in methods {
        let orig_method_sig = method.sig.clone();
        let orig_method_name = &orig_method_sig.ident;
        let variant_name = &orig_method_sig.ident;
        let mut orig_method_invoke_fields = Vec::new();
        let mut variant_decl_fields = Vec::new();
        let mut variant_init_fields = Vec::new();
        let mut variant_match_arm_fields = Vec::new();
        for arg in &orig_method_sig.inputs {
            if let syn::FnArg::Typed(p) = arg {
                let pat = p.pat.clone();
                match *pat {
                    syn::Pat::Ident(pid) => {
                        variant_init_fields.push(quote!(#pid));
                        variant_match_arm_fields.push(quote!(#pid));
                        orig_method_invoke_fields.push(quote!(#pid));
                    }
                    _ => panic!("type pattern was not a syn::Pat::Ident"),
                }
                variant_decl_fields.push(quote!(#p));
            }
        }

        match &orig_method_sig.output {
            syn::ReturnType::Type(_, return_type) => {
                variant_decl_fields
                    .push(quote!(_responder: futures::channel::oneshot::Sender<#return_type>));
                variant_init_fields.push(quote!(_responder: tx));
                variant_match_arm_fields.push(quote!(_responder));
            }
            _ => panic!("methods that return default/() are not handled yet"),
        }
        variants.push(quote!(#variant_name { #(#variant_decl_fields),* }));
        matchers.push(quote!(
            #enum_name::#variant_name { #(#variant_match_arm_fields),* } => {
                _responder.send(self.#orig_method_name(#(#orig_method_invoke_fields),*)).expect("response sent successfully");
            }
        ));

        let send_call = if args.use_async {
            quote!(self.send_message(#enum_name::#variant_name { #(#variant_init_fields),* }).await;)
        } else {
            quote!(self.send_message(#enum_name::#variant_name { #(#variant_init_fields),* });)
        };

        handle_methods.push(quote!(
            #orig_method_sig {
                let (tx, rx) = futures::channel::oneshot::channel::<usize>();
                #send_call
                futures::executor::block_on(rx).expect("response received successfully")
            }
        ));
        handle_method_sigs.push(quote!(#orig_method_sig;));
    }

    let trait_decl = if args.use_async {
        quote! {
            #[async_trait::async_trait]
            pub trait #trait_name {
                #(async #handle_method_sigs)*
            }

            #[async_trait::async_trait]
            impl #trait_name for #handle_name {
                #(async #handle_methods)*
            }
        }
    } else {
        quote! {
            pub trait #trait_name {
                #(#handle_method_sigs)*
            }

            impl #trait_name for #handle_name {
                #(#handle_methods)*
            }
        }
    };

    // let stream =
    TokenStream::from(quote! {
        #item

        pub type #actor_name = #actor_type<#item_type>;
        pub type #handle_name = #handle_type<#item_type>;

        #trait_decl

        #[allow(non_camel_case_types)]
        pub enum #enum_name {
            #(#variants),*
        }

        impl kaiseki_actor::ActorState for #item_type {
            type Message = #enum_name;

            fn dispatch(&mut self, message: Self::Message) {
                match message {
                    #(#matchers)*
                }
            }
        }
    })

    // let strstr = stream.to_string();
    // eprintln!("{strstr}");

    // let streamstr = unparse(&stream).unwrap();
    // eprintln!("{streamstr}");
    // stream
}

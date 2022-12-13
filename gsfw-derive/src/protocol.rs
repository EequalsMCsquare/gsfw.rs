use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput)]
#[darling(attributes(protocol), forward_attrs(allow, doc, cfg))]
struct DeriveOps {
    registry: String,
    name: Option<String>,
}

pub fn derive_registry_impl(input: DeriveInput) -> TokenStream {
    let ops = DeriveOps::from_derive_input(&input).unwrap();
    let ident = input.ident;
    let name = ops.name.unwrap_or(format!("{}", ident));
    let enumval: syn::Type = match syn::parse_str(&format!("{}::{}", ops.registry, name)) {
        Ok(var) => var,
        Err(err) => panic!("fail to parse variant. {}", err),
    };
    let expand = quote! {
        impl ::gsfw::registry::Protocol for #ident {
            const MSG_ID: i32 = #enumval as i32;
        }
    };
    // println!("{}", expand);
    expand.into()
    // TokenStream::new()
}
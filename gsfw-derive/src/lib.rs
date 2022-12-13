mod dirty;
mod registry;
mod protocol;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Dirty, attributes(dirty))]
pub fn derive_dirty(input: TokenStream) -> TokenStream {
    let derive= parse_macro_input!(input as DeriveInput);
    dirty::try_derive_dirty(derive).unwrap()
}

#[proc_macro_derive(Protocol, attributes(protocol))]
pub fn derive_protocol(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    protocol::derive_registry_impl(input)
}

#[proc_macro_derive(Registry, attributes(registry))]
pub fn derive_registry(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    registry::derive_registry_impl(input)
}

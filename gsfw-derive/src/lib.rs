mod dirty;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Dirty, attributes(dirty))]
pub fn derive_dirty(input: TokenStream) -> TokenStream {
    let derive= parse_macro_input!(input as DeriveInput);
    dirty::try_derive_dirty(derive).unwrap()
}
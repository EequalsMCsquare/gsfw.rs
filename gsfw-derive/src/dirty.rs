use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Ident};

pub fn try_derive_dirty(input: DeriveInput) -> anyhow::Result<TokenStream> {
    let dirty_field = match input.data {
        syn::Data::Struct(ref data) => {
            find_dirty_field(&data.fields)
        }
        _ => panic!("only struct support Dirty")
    };
    let name = input.ident;
    Ok(quote! {
        impl ::gsfw::util::dirty::Dirty for #name {
            #[inline(always)]
            fn make_dirty(&self) {
                self.#dirty_field.make_dirty()
            }

            #[inline(always)]
            fn clear_dirty(&self) {
                self.#dirty_field.clear_dirty()
            }
        }
    }
    .into())
}

fn find_dirty_field(fields: &Fields) -> Ident {
    let dirty_fields: Vec<_> = fields
        .iter()
        .filter(|f| {
            for field_attr in &f.attrs {
                if field_attr.path.get_ident().unwrap().to_string() == "dirty" {
                    return true;
                }
            }
            return false;
        })
        .collect();
    if dirty_fields.len() > 1 {
        panic!("each struct can only have one field has #[dirty]")
    } else if dirty_fields.len() == 0 {
        return syn::parse_str("_dirty").unwrap();
    } else {
        dirty_fields.first().unwrap().ident.to_owned().unwrap()
    }
}

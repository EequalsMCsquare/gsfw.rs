use darling::{FromDeriveInput, FromVariant};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, DeriveInput, LitInt, Token, Variant};

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(registry), forward_attrs(allow, doc, cfg))]
struct DeriveOps {
    prefix: Option<String>, // type prefix, used to specify crate
    rename: Option<String>, // rename generated Registry enum, default is <original name>_
                            // extra_attrs: Option<String>, // attributes for generated Registry
}

#[derive(FromVariant, Default)]
#[darling(attributes(registry))]
#[darling(default)]
struct VariantOps {
    skip: bool, // if set true this protocol will not exist in generated registry
}

pub fn derive_registry_impl(input: DeriveInput) -> TokenStream {
    let ops = DeriveOps::from_derive_input(&input).expect("fail to parse derive attributes. {}");
    let data = ensure_enum(&input);
    let rename = ops.rename.unwrap_or(format!("{}_", input.ident));
    let prefix = match &ops.prefix {
        Some(prefix) => Some(prefix.as_str()),
        None => None,
    };
    let reg = registry_codegen(&data.variants, prefix, input.ident.clone(), rename);
    let expand = quote! {
        #reg
    };
    // println!("{}", expand);
    // TokenStream::new()
    expand.into()
}

fn ensure_enum(input: &DeriveInput) -> &syn::DataEnum {
    if let syn::Data::Enum(data) = &input.data {
        data
    } else {
        panic!("Registry derive can only be used with enum")
    }
}

fn registry_codegen(
    vars: &Punctuated<Variant, Token![,]>,
    prefix: Option<&str>,
    name: syn::Ident,
    rename: String,
) -> TokenStream2 {
    let metas: Vec<_> = vars.iter().filter_map(|v| parse_variant2(v)).collect();
    let count = metas.len();

    let rename = format_ident!("{}", rename);

    let variants = TokenStream2::from_iter(metas.iter().map(|meta| {
        let typ = meta.typ(prefix);
        let ident = meta.ident.clone();
        quote! {#ident(#typ),}
    }));

    let names = metas.iter().map(|m| m.ident.to_string());
    let ids = metas.iter().map(|m| m.id_lit.clone());
    let decode_frame_cases = metas.iter().map(|m| {
        let vname = m.ident.clone();
        let typ = m.typ(prefix);
        quote! {
            #name :: #vname => <#typ as ::prost::Message>::decode(buf)
                .map_err(|err|::gsfw::error::Error::Decode(err.to_string()))
                .map(|pb|Self::#vname(pb))
        }
    });

    let encoded_len_cases = metas.iter().map(|m| {
        let vname = m.ident.clone();
        quote! {
            Self::#vname(pmsg) => ::std::mem::size_of::<i32>() + ::prost::Message::encoded_len(pmsg)
        }
    });

    let encode_to_cases = metas.iter().map(|m| {
        let vname = m.ident.clone();
        let typ = m.typ(prefix);
        quote! {
            Self::#vname(pmsg) => {
                buf.put_i32(<#typ as ::gsfw::registry::Protocol>::MSG_ID);
                <#typ as ::prost::Message>::encode(pmsg, buf)
                    .map_err(|err|::gsfw::error::Error::Encode(err.to_string()))
            }
        }
    });

    let encode_cases = metas.iter().map(|m| {
        let vname = m.ident.clone();
        let typ = m.typ(prefix);
        quote! {
            Self::#vname(pmsg) => {
                let required_bytes = ::std::mem::size_of::<i32>() + ::prost::Message::encoded_len(pmsg);
                let mut buf = ::bytes::BytesMut::with_capacity(required_bytes);
                ::bytes::BufMut::put_i32(&mut buf, <#typ as ::gsfw::registry::Protocol>::MSG_ID);
                <#typ as ::prost::Message>::encode(pmsg, &mut buf).unwrap();
                buf.freeze()
            }
        }
    });

    let encode_to_with_len_cases = metas.iter().map(|m| {
        let vname = m.ident.clone();
        let typ = m.typ(prefix);
        quote! {
            Self::#vname(pmsg) => {
                let encoded_len = ::prost::Message::encoded_len(pmsg) + ::std::mem::size_of::<u32>();
                buf.put_u32(encoded_len as u32);
                buf.put_i32(<#typ as ::gsfw::registry::Protocol>::MSG_ID);
                <#typ as ::prost::Message>::encode(pmsg, buf)
                    .map_err(|err|::gsfw::error::Error::Encode(err.to_string()))
            }
        }
    });

    let encode_with_len_cases = metas.iter().map(|m|{
        let vname = m.ident.clone();
        let typ = m.typ(prefix);
        quote! {
            Self::#vname(pmsg) => {
                let required_bytes = ::std::mem::size_of::<i32>() + ::prost::Message::encoded_len(pmsg);
                let mut buf = ::bytes::BytesMut::with_capacity(required_bytes + ::std::mem::size_of::<u32>());
                ::bytes::BufMut::put_u32(&mut buf, required_bytes as u32);
                ::bytes::BufMut::put_i32(&mut buf, <#typ as ::gsfw::registry::Protocol>::MSG_ID);
                <#typ as ::prost::Message>::encode(pmsg, &mut buf).unwrap();
                buf.freeze()
            }
        }
    });

    let into_expand: TokenStream2 = metas
        .iter()
        .map(|m| {
            let typ = m.typ(prefix);
            let vident = m.ident.clone();
            quote! {
                impl TryInto<#typ> for #rename {
                    type Error = ::gsfw::error::Error;
                    fn try_into(self) -> Result<#typ, Self::Error> {
                        if let Self::#vident(inner) = self {
                            Ok(inner)
                        } else {
                            Err(::gsfw::error::Error::VariantCast(stringify!(#typ)))
                        }
                    }
                }
            }
        })
        .collect();

    let from_expand: TokenStream2 = metas
        .iter()
        .map(|m| {
            let typ = m.typ(prefix);
            let vident = m.ident.clone();
            quote! {
                impl From<#typ> for #rename {
                    fn from(inner: #typ) -> Self {
                        Self::#vident(inner)
                    }
                }
            }
        })
        .collect();
    let enum_expand = quote! (
        #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        pub enum #rename{
            #variants
        }
    );

    let defaults = metas.iter().map(|m| {
        let typ = m.typ(prefix);
        let ident = m.ident.clone();
        quote! {
            #rename::#ident(<#typ>::default())
        }
    });

    let defaults2 = defaults.clone();

    let impl_expand = quote! {
        impl ::gsfw::registry::RegistryExt for #rename {
            const COUNT: usize = #count;
            const NAMES: ::once_cell::sync::Lazy<Vec<&'static str>> = ::once_cell::sync::Lazy::new(||{vec![#(#names),*]});
            const IDS: ::once_cell::sync::Lazy< Vec<i32>> = ::once_cell::sync::Lazy::new(||{vec![#(#ids),*]});
            const ID2NAME_MAP: ::once_cell::sync::Lazy<::std::collections::HashMap<i32, &'static str>> = ::once_cell::sync::Lazy::new(||{
                std::iter::zip(Self::IDS.iter().map(|&id|id), Self::NAMES.iter().map(|&s|s)).collect()
            });
            const NAME2ID_MAP: ::once_cell::sync::Lazy<::std::collections::HashMap<&'static str, i32>> = ::once_cell::sync::Lazy::new(||{
                std::iter::zip(Self::NAMES.iter().map(|&s|s), Self::IDS.iter().map(|&id|id)).collect()
            });
            const NAME_MAP: ::once_cell::sync::Lazy<::std::collections::HashMap<&'static str, Self>> = ::once_cell::sync::Lazy::new(||{
                let defaults_vec = vec![#(#defaults),*];
                std::iter::zip(Self::NAMES.iter().map(|&s|s), defaults_vec.into_iter()).collect()
            });
            const ID_MAP: ::once_cell::sync::Lazy<::std::collections::HashMap<i32, Self>> = ::once_cell::sync::Lazy::new(||{
                let defaults_vec = vec![#(#defaults2),*];
                std::iter::zip(Self::IDS.iter().map(|&s|s), defaults_vec.into_iter()).collect()
            });

            fn decode_frame<B>(mut buf: B) -> Result<Self, ::gsfw::error::Error>
            where
                B: ::bytes::Buf,
                Self: Sized
            {
                let msgid = buf.get_i32();
                let msgid = <#name>::from_i32(msgid)
                    .ok_or(::gsfw::error::Error::Decode(
                        format!("unknown protocol id: {}", msgid))
                    )?;
                match msgid {
                    #(#decode_frame_cases),*,
                    _unexpected => Err(::gsfw::error::Error::Decode(format!("attempt to decode an unregistered message. {:?}", _unexpected))),
                }
            }

            fn encoded_len(&self) -> usize
            where
                Self: Sized
            {
                match self {
                    #(#encoded_len_cases),*,
                }
            }

            fn encode_to<B>(&self, buf: &mut B) -> Result<(), ::gsfw::error::Error>
            where
                B: ::bytes::BufMut,
                Self: Sized
            {
                match self {
                    #(#encode_to_cases),*,
                }
            }

            fn encode(&self) -> ::bytes::Bytes
            where
                Self: Sized
            {
                match self {
                    #(#encode_cases),*,
                }
            }

            fn encode_to_with_len<B>(&self, buf: &mut B) -> Result<(), ::gsfw::error::Error>
            where
                B: ::bytes::BufMut,
                Self: Sized
            {
                match self {
                    #(#encode_to_with_len_cases),*,
                }
            }

            fn encode_with_len(&self) -> ::bytes::Bytes
            where
                Self: Sized
            {
                match self {
                    #(#encode_with_len_cases),*,
                }
            }
        }
    };
    // println!("{}", impl_expand);
    let expand = quote! {
        #enum_expand

        #impl_expand

        #into_expand

        #from_expand
    };
    // println!("{}", expand);
    expand
}

struct VariantMeta {
    ident: syn::Ident,
    id_lit: syn::LitInt,
}

impl VariantMeta {
    pub fn typ(&self, prefix: Option<&str>) -> syn::Type {
        if let Some(prefix) = prefix {
            syn::parse_str(&format!("{}{}", prefix, self.ident)).expect("fail to parse type")
        } else {
            syn::parse_str(&format!("{}", self.ident)).expect("fail to parse type")
        }
    }
}

fn parse_variant2(var: &Variant) -> Option<VariantMeta> {
    let ops = VariantOps::from_variant(var).expect("fail to parse variant attributes.");
    if ops.skip {
        return None;
    }
    Some(VariantMeta {
        ident: var.ident.clone(),
        id_lit: parse_msgid(
            &var.discriminant
                .as_ref()
                .expect("variant discriminant is None")
                .1,
        ),
    })
}

fn parse_msgid(expr: &syn::Expr) -> LitInt {
    if let syn::Expr::Lit(lit) = expr {
        if let syn::Lit::Int(id) = &lit.lit {
            id.clone()
            // id.base10_parse()
            //     .expect("fail to parse Lit to base10 integer")
        } else {
            panic!("invalid format")
        }
    } else {
        panic!("invalid format")
    }
}

fn _parse_variant(var: &Variant, prefix: &str) -> Option<TokenStream2> {
    let ops = VariantOps::from_variant(var).expect("fail to parse variant attributes.");
    if ops.skip {
        return None;
    }
    let ident = var.ident.clone();
    let typ: syn::Type = match syn::parse_str(&format!("{}{}", prefix, ident)) {
        Ok(typ) => typ,
        Err(err) => panic!("fail to parse type. {}", err),
    };
    let expand = quote! {
        #ident(#typ),
    };
    Some(expand)
}

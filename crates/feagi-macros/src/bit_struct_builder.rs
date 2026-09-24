//! Generates a struct wrapping an unsigned integer with one bool  per named bit field.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Attribute, Ident, Token, Type, Visibility};

use crate::common::parse_optional_comma;


pub fn bit_struct_builder(input: TokenStream) -> TokenStream {
    let spec = parse_macro_input!(input as BitStructBuilderSpec);
    match spec.expand() {
        Ok(tokens) => TokenStream::from(tokens),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

struct BitStructBuilderSpec {
    storage_type: Type,
    vis: Visibility,
    struct_name: Ident,
    attrs: Vec<Attribute>,
    fields: Vec<Ident>,
}

impl Parse for BitStructBuilderSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let storage_type: Type = input.parse()?;
        input.parse::<Token![,]>()?;

        let vis: Visibility = input.parse()?;
        let struct_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let mut attrs = Vec::new();
        while input.peek(Token![#]) {
            attrs.extend(input.call(Attribute::parse_outer)?);
        }

        let fields_input;
        syn::braced!(fields_input in input);

        let mut fields = Vec::new();
        while !fields_input.is_empty() {
            fields.push(fields_input.parse()?);
            parse_optional_comma(&fields_input)?;
        }

        Ok(Self {
            storage_type,
            vis,
            struct_name,
            attrs,
            fields,
        })
    }
}

impl BitStructBuilderSpec {
    fn expand(self) -> syn::Result<proc_macro2::TokenStream> {
        let (storage_ident, bit_capacity) = parse_unsigned_storage(&self.storage_type)?;

        if self.fields.len() > bit_capacity {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "bit_struct_builder: {} field(s) exceed {} capacity ({} bit(s))",
                    self.fields.len(),
                    storage_ident,
                    bit_capacity,
                ),
            ));
        }

        let Self {
            storage_type,
            vis,
            struct_name,
            attrs,
            fields,
        } = self;

        let method_impls = fields.iter().enumerate().map(|(index, field)| {
            let setter = format_ident!("set_{}", field);
            let toggler = format_ident!("toggle_{}", field);
            let bit_index = index as u32;

            quote! {
                #[inline]
                #vis fn #field(&self) -> bool {
                    (self.0 & ((1 as #storage_type) << #bit_index)) != 0
                }

                #[inline]
                #vis fn #setter(&mut self, value: bool) {
                    let mask = (1 as #storage_type) << #bit_index;
                    if value {
                        self.0 |= mask;
                    } else {
                        self.0 &= !mask;
                    }
                }

                #[inline]
                #vis fn #toggler(&mut self) {
                    self.0 ^= (1 as #storage_type) << #bit_index;
                }
            }
        });

        Ok(quote! {
            #(#attrs)*
            #vis struct #struct_name(#storage_type);

            impl #struct_name {
                #[inline]
                #vis const fn new(bits: #storage_type) -> Self {
                    Self(bits)
                }

                #[inline]
                #vis const fn bits(&self) -> #storage_type {
                    self.0
                }

                #(#method_impls)*
            }
        })
    }
}

fn parse_unsigned_storage(storage_type: &Type) -> syn::Result<(Ident, usize)> {
    let path = match storage_type {
        Type::Path(type_path) if type_path.qself.is_none() => &type_path.path,
        _ => {
            return Err(syn::Error::new_spanned(
                storage_type,
                "bit_struct_builder: storage type must be u8, u16, u32, or u64",
            ));
        }
    };

    let ident = path.get_ident().ok_or_else(|| {
        syn::Error::new_spanned(
            storage_type,
            "bit_struct_builder: storage type must be u8, u16, u32, or u64",
        )
    })?;

    // feels dumb but couldnt think of another way
    let bit_capacity = match ident.to_string().as_str() {
        "u8" => 8,
        "u16" => 16,
        "u32" => 32,
        "u64" => 64,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "bit_struct_builder: storage type must be u8, u16, u32, or u64",
            ));
        }
    };

    Ok((ident.clone(), bit_capacity))
}

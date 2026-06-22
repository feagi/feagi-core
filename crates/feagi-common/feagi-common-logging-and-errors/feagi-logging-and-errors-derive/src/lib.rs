use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Error, Fields, LitStr, Path, Type, TypePath};

#[proc_macro_derive(FeagiErrorKey, attributes(feagi_error))]
pub fn derive_feagi_error_key(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_error_key(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(FeagiError, attributes(feagi_error))]
pub fn derive_feagi_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_error(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_error_key(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    reject_type_generics(&input)?;
    let feagi_error_crate = feagi_error_crate_path(&input)?;

    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(Error::new_spanned(
                    name,
                    "FeagiErrorKey can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                name,
                "FeagiErrorKey can only be derived for structs",
            ));
        }
    };

    let mut field_idents = Vec::new();
    let mut field_types = Vec::new();
    let mut has_context = false;

    for field in fields {
        let ident = field
            .ident
            .ok_or_else(|| Error::new_spanned(&name, "FeagiErrorKey requires named fields"))?;

        if ident == "context" {
            has_context = true;
            if !is_static_str(&field.ty) {
                return Err(Error::new_spanned(
                    field.ty,
                    "the `context` field must have type `&'static str`",
                ));
            }
        }

        field_idents.push(ident);
        field_types.push(field.ty);
    }

    if !has_context {
        return Err(Error::new_spanned(
            name,
            "FeagiErrorKey requires a `context: &'static str` field",
        ));
    }

    let opaque_debug_values = field_idents.iter().map(|ident| {
        if ident == "context" {
            quote! { .field("context", &self.context) }
        } else {
            let field_name = ident.to_string();
            quote! { .field(#field_name, &"<opaque>") }
        }
    });

    Ok(quote! {
        impl #name {
            pub const fn new(#(#field_idents: #field_types),*) -> Self {
                Self {
                    #(#field_idents),*
                }
            }

            pub const fn context(&self) -> &'static str {
                self.context
            }
        }

        impl ::core::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(::core::stringify!(#name))
                    #(#opaque_debug_values)*
                    .finish()
            }
        }

        impl ::core::fmt::Display for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.context)
            }
        }

        impl ::core::error::Error for #name {}

        impl #feagi_error_crate::FeagiErrorKeyTrait for #name {
            fn context(&self) -> &'static str {
                self.context
            }
        }
    })
}

fn expand_error(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    reject_type_generics(&input)?;
    let feagi_error_crate = feagi_error_crate_path(&input)?;

    let name = input.ident;
    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => {
            return Err(Error::new_spanned(
                name,
                "FeagiError can only be derived for enums",
            ));
        }
    };

    let mut variant_idents = Vec::new();
    let mut variant_field_idents = Vec::new();

    for variant in variants {
        let variant_ident = variant.ident;
        let Fields::Unnamed(fields) = variant.fields else {
            return Err(Error::new_spanned(
                variant_ident,
                "FeagiError variants must be tuple variants with exactly one wrapped error key or error enum",
            ));
        };

        if fields.unnamed.len() != 1 {
            return Err(Error::new_spanned(
                variant_ident,
                "FeagiError variants must be tuple variants with exactly one wrapped error key or error enum",
            ));
        }

        let field_ident = format_ident!("wrapped_{}", variant_ident.to_string().to_lowercase());
        variant_idents.push(variant_ident);
        variant_field_idents.push(field_ident);
    }

    Ok(quote! {
        impl #name {
            pub fn context(&self) -> &'static str {
                match self {
                    #(
                        Self::#variant_idents(#variant_field_idents) => #variant_field_idents.context(),
                    )*
                }
            }
        }

        impl ::core::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(
                        Self::#variant_idents(#variant_field_idents) => {
                            f.debug_tuple(::core::stringify!(#variant_idents))
                                .field(#variant_field_idents)
                                .finish()
                        }
                    )*
                }
            }
        }

        impl ::core::fmt::Display for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(
                        Self::#variant_idents(#variant_field_idents) => ::core::fmt::Display::fmt(#variant_field_idents, f),
                    )*
                }
            }
        }

        impl ::core::error::Error for #name {
            fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                match self {
                    #(
                        Self::#variant_idents(#variant_field_idents) => {
                            ::core::option::Option::Some(#variant_field_idents)
                        }
                    )*
                }
            }
        }

        impl #feagi_error_crate::FeagiErrorTrait for #name {
            fn context(&self) -> &'static str {
                self.context()
            }
        }
    })
}

fn feagi_error_crate_path(input: &DeriveInput) -> syn::Result<Path> {
    let mut crate_path = None;

    for attribute in &input.attrs {
        if !attribute.path().is_ident("feagi_error") {
            continue;
        }

        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("crate") {
                return Err(meta.error("unsupported feagi_error attribute; expected `crate = \"...\"`"));
            }

            if crate_path.is_some() {
                return Err(meta.error("duplicate feagi_error crate path override"));
            }

            let value = meta.value()?;
            let literal: LitStr = value.parse()?;
            crate_path = Some(literal.parse()?);
            Ok(())
        })?;
    }

    Ok(crate_path.unwrap_or_else(|| parse_quote!(::feagi_logging_and_errors)))
}

fn reject_type_generics(input: &DeriveInput) -> syn::Result<()> {
    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            &input.generics,
            "FEAGI error derives do not support generic parameters",
        ));
    }

    Ok(())
}

fn is_static_str(ty: &Type) -> bool {
    let Type::Reference(reference) = ty else {
        return false;
    };

    let Some(lifetime) = &reference.lifetime else {
        return false;
    };

    if lifetime.ident != "static" {
        return false;
    }

    let Type::Path(TypePath { path, .. }) = reference.elem.as_ref() else {
        return false;
    };

    path.is_ident("str")
}

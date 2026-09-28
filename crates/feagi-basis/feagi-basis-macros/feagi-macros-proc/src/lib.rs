
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, Type, TypePath};
mod basis;
mod templates;

//region Independent Builders

use independent_builders::bit_struct_builder;

mod independent_builders;

/// Builds a bit field struct around a uint with one named bit field per flag.
///
/// # Example
///
/// ```ignore
/// use feagi_macros::bit_struct_builder;
///
/// bit_struct_builder! {
///     u8,
///     pub ExampleFlags,
///     #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
///     /// Runtime visibility and enable flags.
///     {
///         enabled,
///         visible,
///     }
/// }
///
/// let mut flags = ExampleFlags::new(0);
/// flags.set_enabled(true);
/// assert!(flags.enabled());
/// flags.toggle_visible();
/// assert!(flags.visible());
/// assert_eq!(flags.bits(), 0b0000_0011);
/// ```
#[proc_macro]
pub fn bit_struct_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bit_struct_builder::bit_struct_builder(input)
}

//endregion

//region Errors

// TODO move to seperate module

#[proc_macro_derive(FeagiFail)]
pub fn derive_feagi_fail(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_error_key(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(FeagiError)]
pub fn derive_feagi_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_error(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_error_key(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    reject_type_generics(&input)?;

    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(Error::new_spanned(
                    name,
                    "FeagiFail can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(name, "FeagiFail can only be derived for structs"));
        }
    };

    let mut field_idents = Vec::new();
    let mut field_types = Vec::new();
    let mut has_context = false;

    for field in fields {
        let ident = field
            .ident
            .ok_or_else(|| Error::new_spanned(&name, "FeagiFail requires named fields"))?;

        if ident == "context" {
            has_context = true;
            if !is_static_str(&field.ty) {
                return Err(Error::new_spanned(field.ty, "the `context` field must have type `&'static str`"));
            }
        }

        field_idents.push(ident);
        field_types.push(field.ty);
    }

    if !has_context {
        return Err(Error::new_spanned(name, "FeagiFail requires a `context: &'static str` field"));
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

        impl FeagiFailTrait for #name {
            fn context(&self) -> &'static str {
                self.context
            }
        }
    })
}

fn expand_error(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    reject_type_generics(&input)?;

    let name = input.ident;
    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => {
            return Err(Error::new_spanned(name, "FeagiError can only be derived for enums"));
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

        impl FeagiErrorTrait for #name {
            fn context(&self) -> &'static str {
                self.context()
            }
        }
    })
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

//endregion

//region Templates and Generators

use crate::basis::GeneratorFromTemplate;
use crate::templates::template_root::TemplateRoot;

//region Request Response

#[cfg(feature = "request_response_base_macros")]
#[proc_macro]
pub fn template_request_category(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use crate::templates::requests_responses::requests_responses_structs::TemplateRequestResponseCategory;
    TemplateRoot::<TemplateRequestResponseCategory>::parse_template_and_generate_generator_input_macro(input)
}

//region Generators

#[cfg(feature = "request_response_base_macros")]
#[proc_macro]
pub fn generate_from_template_request_response_structs_and_enums(generator_input_macro: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use crate::templates::requests_responses::generators::request_response_enum_generator::EnumRequestResponseGenerator;
    EnumRequestResponseGenerator::read_template_and_generate_code(generator_input_macro)
}

#[cfg(feature = "request_response_ahkami_codegen")]
#[proc_macro]
pub fn generate_from_template_ohkami_rest_server(generator_input_macro: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use crate::templates::requests_responses::generators::request_response_ohkami_server_generator::OhkamiServerGenerator;
    OhkamiServerGenerator::read_template_and_generate_code(generator_input_macro)
}

//endregion

//endregion

//endregion











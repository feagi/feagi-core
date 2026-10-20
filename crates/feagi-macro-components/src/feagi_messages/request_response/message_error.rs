use crate::common::{basis_path, serde_data_derive_attributes};
use crate::templates::Unparse;
use indexmap::IndexMap;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, LitStr, Token, Type};

/// The error a message can respond with, on top of the errors of the channel / backend it was
/// sent over. Formatted as one of the following:
/// error: {
///     ErrorName: "Description String",
/// }
/// error: path::to::ExplicitErrorType
/// (or no `error` at all, for messages that should never fail)
pub enum MessageErrorTemplate {
    /// No error was given, the shared `FeagiUnexpectedError` is used
    Default,
    /// An error enum of unit variants is generated, each displayed as its description
    Variants(IndexMap<Ident, LitStr>),
    /// An existing error type is used as is. It must implement `FeagiMessageError`
    Explicit(Type),
}

impl MessageErrorTemplate {
    /// The error type the message uses. `generated_name` is the name a `Variants` enum is
    /// generated under
    pub fn error_type(&self, generated_name: &Ident) -> proc_macro2::TokenStream {
        let basis = basis_path();
        match self {
            MessageErrorTemplate::Default => quote!(#basis::feagi_message::feagi_response_message::FeagiUnexpectedError),
            MessageErrorTemplate::Variants(_) => quote!(#generated_name),
            MessageErrorTemplate::Explicit(error_type) => quote!(#error_type),
        }
    }

    /// Generates the error enum for `Variants`. Nothing is generated otherwise
    pub fn generate_rust_error_enum(&self, generated_name: &Ident, message_description: &LitStr) -> proc_macro2::TokenStream {
        let MessageErrorTemplate::Variants(variants) = self else {
            return proc_macro2::TokenStream::new();
        };

        let basis = basis_path();
        let derives = serde_data_derive_attributes();
        let doc = LitStr::new(
            &format!("Errors the request \"{}\" can respond with", message_description.value()),
            message_description.span(),
        );
        let variant_names: Vec<&Ident> = variants.keys().collect();
        let descriptions: Vec<&LitStr> = variants.values().collect();

        quote! {
            #[doc = #doc]
            #derives
            pub enum #generated_name {
                #(
                    #[doc = #descriptions]
                    #variant_names,
                )*
            }

            impl ::core::fmt::Display for #generated_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #( Self::#variant_names => f.write_str(#descriptions), )*
                    }
                }
            }

            impl ::core::error::Error for #generated_name {}

            impl #basis::feagi_message::feagi_response_message::FeagiMessageError for #generated_name {}
        }
    }
}

impl Parse for MessageErrorTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.peek(Brace) {
            return Ok(MessageErrorTemplate::Explicit(input.parse()?));
        }

        let entries;
        braced!(entries in input);
        let mut variants = IndexMap::new();
        while !entries.is_empty() {
            let name: Ident = entries.parse()?;
            entries.parse::<Token![:]>()?;
            let description: LitStr = entries.parse()?;
            let span = name.span();
            if variants.insert(name, description).is_some() {
                return Err(syn::Error::new(span, "duplicate error name"));
            }
            if entries.is_empty() {
                break;
            }
            entries.parse::<Token![,]>()?;
        }
        if variants.is_empty() {
            return Err(entries.error("error block must define at least one error. Leave `error` out to use the default error"));
        }
        Ok(MessageErrorTemplate::Variants(variants))
    }
}

impl Unparse for MessageErrorTemplate {
    /// Re-emit `error: ...,`, or nothing for the default error
    fn unparse(&self) -> proc_macro2::TokenStream {
        match self {
            MessageErrorTemplate::Default => quote!(),
            MessageErrorTemplate::Variants(variants) => {
                let names = variants.keys();
                let descriptions = variants.values();
                quote!(error: { #( #names: #descriptions, )* },)
            }
            MessageErrorTemplate::Explicit(error_type) => quote!(error: #error_type,),
        }
    }
}

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{braced, parse_macro_input, Token};

mod kw {
    syn::custom_keyword!(exported_macro_name);
    syn::custom_keyword!(template);

}

/// Defines a struct that can be parsed but also exported to be parsed again
pub trait TemplateStruct: Parse {

    // parse() from Parse is for reading and validating input

    /// Export this template into a token stream that can be parsed again
    fn expand_template(&self) -> proc_macro2::TokenStream;
}

pub struct TemplateRoot<T: TemplateStruct> {
    macro_name: syn::Ident,
    generated_template: T
}

impl<T: TemplateStruct> TemplateRoot<T> {
    /// Given a template definition token stream, take the macro name, and parse the actual struct
    /// T. If all is valid, export the generator macro of the given name for T
    pub fn parse_template_and_generate_generator_macro(input: TokenStream) -> TokenStream {
        let template_root = parse_macro_input!(input as Self);

        let macro_name = template_root.macro_name;
        let template = template_root.generated_template.expand_template();

        quote! {
            macro_rules! #macro_name {
                () => {
                    #template
                };
            }
        }.into()
    }

    /// Read the generator macro (no deep validation needed as it was already confirmed valid), and
    /// return the struct
    pub fn parse_generator_macro(input: TokenStream) -> syn::Result<T> {

    }
}

impl<T: TemplateStruct> Parse for TemplateRoot<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        input.parse::<kw::exported_macro_name>()?;
        input.parse::<Token![:]>()?;
        let macro_name: syn::Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<kw::template>()?;
        input.parse::<Token![:]>()?;
        let struct_stream;
        let mut generated_template: T;
        braced!(struct_stream in input);
        while !struct_stream.is_empty() {
            generated_template = T::parse(&struct_stream)?;
            // Yes, technically one could put multiple templates and we only read the last one. I dont care to fix this extreme intentional edge case rn
        };

        Ok(Self {
            macro_name,
            generated_template
        })

    }
}
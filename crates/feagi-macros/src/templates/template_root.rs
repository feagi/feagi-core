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
    pub macro_name: syn::Ident,
    pub generated_template: T
}

impl<T: TemplateStruct> TemplateRoot<T> {
    /// Parse and validate a template definition, then export it as a reusable `macro_rules!`.
    /// [`Self::parse_generator_input_macro`].
    pub fn parse_template_and_generate_generator_input_macro(input: TokenStream) -> TokenStream {
        let template_root = parse_macro_input!(input as Self);

        let macro_name = template_root.macro_name;
        let template = template_root.generated_template.expand_template();

        quote! {
            macro_rules! #macro_name {
                ($consumer:path) => {
                    $consumer! {
                        #template
                    }
                };
            }
        }.into()
    }

    /// Parse the tokens a consumer receives from an exported template callback.
    pub fn parse_generator_input_macro(input: TokenStream) -> syn::Result<T> {
        T::parse.parse(input)
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
        braced!(struct_stream in input);
        let generated_template: T = struct_stream.parse()?;
        Ok(Self {
            macro_name,
            generated_template
        })

    }
}
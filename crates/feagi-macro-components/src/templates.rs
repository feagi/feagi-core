use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{braced, Token};

mod kw {
    syn::custom_keyword!(exported_macro_name);
    syn::custom_keyword!(template);

}

/// A template fragment that can be parsed from tokens and re-emitted as the same DSL.
pub trait Unparse: Parse {
    /// Export this template into a token stream that can be parsed again
    fn unparse(&self) -> proc_macro2::TokenStream;
}

/// Takes in a template data struct, then outputs actual usable source code
pub trait GeneratorFromTemplate<T: Unparse> {
    fn generate_code_from_template(template: T) -> proc_macro2::TokenStream;

    fn read_template_and_generate_code(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match TemplateRoot::<T>::parse_generator_input_macro(input.into()) {
            Ok(template) => Self::generate_code_from_template(template).into(),
            Err(err) => err.to_compile_error().into(),
        }
    }
}


pub struct TemplateRoot<T: Unparse> {
    pub macro_name: syn::Ident,
    pub generated_template: T
}

impl<T: Unparse> TemplateRoot<T> {
    /// Parse and validate a template definition, then export it as a reusable `macro_rules!`.
    /// [`Self::parse_generator_input_macro`].
    pub fn parse_template_and_generate_generator_input_macro(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let template_root = match TemplateRoot::<T>::parse.parse2(input) {
            Ok(template_root) => template_root,
            Err(err) => return err.to_compile_error(),
        };

        let macro_name = template_root.macro_name;
        let template = template_root.generated_template.unparse();

        quote! {
            macro_rules! #macro_name {
                ($consumer:path) => {
                    $consumer! {
                        #template
                    }
                };
            }
        }
    }

    /// Parse the tokens a consumer receives from an exported template callback.
    pub fn parse_generator_input_macro(input: proc_macro2::TokenStream) -> syn::Result<T> {
        T::parse.parse(input.into())
    }
}

impl<T: Unparse> Parse for TemplateRoot<T> {
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
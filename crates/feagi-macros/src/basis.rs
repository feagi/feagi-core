use proc_macro2::{Ident};
use quote::quote;
use syn::bracketed;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};
use crate::templates::template_root::TemplateRoot;

/// If a comma exists, parse it. If not, dont die
pub fn parse_optional_comma(input: ParseStream) -> syn::Result<()> {
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }
    Ok(())
}

/// A template fragment that can be parsed from tokens and re-emitted as the same DSL.
pub trait TemplateStruct: Parse {
    /// Export this template into a token stream that can be parsed again
    fn expand_template(&self) -> proc_macro2::TokenStream;
}

/// Takes in a template data struct, then outputs actual usable source code
pub trait GeneratorFromTemplate<T: TemplateStruct> {
    fn generate_code_from_template(template: T) -> proc_macro2::TokenStream;
    
    fn read_template_and_generate_code(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
        match TemplateRoot::<T>::parse_generator_input_macro(input.into()) {
            Ok(template) => Self::generate_code_from_template(template).into(),
            Err(err) => err.to_compile_error().into(),
        }
    }
}


//region Struct Builder Parameters

/// Represents all members (`StructBuilderParameterField`) to make a struct
pub struct StructBuilderParameters(Vec<StructBuilderParameterField>);

impl StructBuilderParameters {
    pub(crate) fn fields(&self) -> &[StructBuilderParameterField] {
        &self.0
    }

    pub(crate) fn to_vec(self) -> Vec<StructBuilderParameterField> {self.0}
}

impl TemplateStruct for StructBuilderParameters {
    /// Re-emits this parameter list in contract DSL form (`[ ... ]`).
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let fields = self.fields();
        if fields.is_empty() {
            return quote! { [] };
        }

        let entries = fields.iter().map(TemplateStruct::expand_template);
        quote! {
            [
                #(#entries,)*
            ]
        }
    }
}

/*
    Parses the following:
    [
        StructBuilderParameterField, ...
    ]
*/
impl Parse for StructBuilderParameters {
    fn parse(fields_input: ParseStream) -> syn::Result<Self> {
        let field_input;
        bracketed!(field_input in fields_input);
        let mut fields = Vec::new();
        while !field_input.is_empty() {
            fields.push(field_input.parse::<StructBuilderParameterField>()?);
            if field_input.is_empty() {
                break;
            }
            field_input.parse::<Token![,]>()?;
        }
        Ok(Self(fields))
    }
}

/// A field of a `StructBuilderParameters`, used to represent a members name, its type, and optional comment.
pub struct StructBuilderParameterField {
    /// The string name of the parameter.
    /// This becomes the generated structs field member name so should be snake case
    pub parameter_name: LitStr,
    /// The data type of the parameter. Depending on the usecase, there may be various restrictions
    pub parameter_type: Ident,
    /// Optional description for this parameter
    pub description: Option<LitStr>,
}

/*
    Parses the following:
    "parameter_name": ParameterType, "optional comment"
*/
impl Parse for StructBuilderParameterField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parameter_name: LitStr = input.parse()?;
        input.parse::<Token![:]>()?;
        let parameter_type: Ident = input.parse()?;

        let description = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.peek(LitStr) {
                Some(input.parse::<LitStr>()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            parameter_name,
            parameter_type,
            description,
        })
    }
}

impl TemplateStruct for StructBuilderParameterField {
    /// Re-emits a single struct-builder field entry inside `[ ... ]`.
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let parameter_name = &self.parameter_name;
        let parameter_type = &self.parameter_type;
        match &self.description {
            Some(description) => {
                quote! {
                    #parameter_name: #parameter_type, #description
                }
            }
            None => {
                quote! {
                    #parameter_name: #parameter_type
                }
            }
        }
    }
}

//endregion
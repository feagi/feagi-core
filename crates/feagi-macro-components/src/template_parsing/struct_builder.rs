use quote::quote;
use syn::{bracketed, LitStr, Token};
use syn::parse::{Parse, ParseStream};
use crate::common::lit_str_to_ident;
use crate::templates::TemplateStruct;

/// Represents all members (`StructBuilderParameterField`) to make a struct
pub struct StructParameters(Vec<StructParameterField>);

impl StructParameters {
    pub fn fields(&self) -> &[StructParameterField] {
        &self.0
    }

    pub fn to_vec(self) -> Vec<StructParameterField> {self.0}

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add field to the end
    pub fn push(&mut self, field: StructParameterField) {
        self.0.push(field)
    }

    /// Combine this field and another one to make a new combo field
    pub fn make_combination_with(&self, other: &StructParameters) -> StructParameters {
        let mut self_parms = self.0.to_vec();
        self_parms.extend_from_slice(other.0.as_slice());
        StructParameters(self_parms)
    }

    /// Generate a basic struct source code output. Does not include comments or any metas
    ///
    /// pub $struct_name {
    ///     #[doc = $doc1]
    ///     field1: type1
    ///     ...
    /// }
    pub fn generate_pub_struct(&self, struct_name: &syn::Ident) -> proc_macro2::TokenStream {

        let mut fields: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

        for field in &self.0 {
            fields.extend(field.to_tokens());
        }

        quote! {

            pub struct #struct_name {
                #fields
            }

        }
    }

    /// Generate a basic struct source code output. Does not include comments or any metas
    ///
    /// $struct_name {
    ///     #[doc = $doc1]
    ///     field1: type1
    ///     ...
    /// }
    pub fn generate_priv_struct(&self, struct_name: &syn::Ident) -> proc_macro2::TokenStream {

        let mut fields: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

        for field in &self.0 {
            fields.extend(field.to_tokens());
        }

        quote! {

            struct #struct_name {
                #fields
            }

        }
    }


}

impl TemplateStruct for StructParameters {
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
impl Parse for StructParameters {
    fn parse(fields_input: ParseStream) -> syn::Result<Self> {
        let field_input;
        bracketed!(field_input in fields_input);
        let mut fields = Vec::new();
        while !field_input.is_empty() {
            fields.push(field_input.parse::<StructParameterField>()?);
            if field_input.is_empty() {
                break;
            }
            field_input.parse::<Token![,]>()?;
        }
        Ok(Self(fields))
    }
}

/// A field of a `StructBuilderParameters`, used to represent a members name, its type, and optional comment.
#[derive(Clone)]
pub struct StructParameterField {
    /// The string name of the parameter.
    /// This becomes the generated structs field member name so should be snake case
    pub parameter_name: LitStr,
    /// The data type of the parameter. Depending on the usecase, there may be various restrictions
    pub parameter_type: syn::Type,
    /// Optional description for this parameter
    pub description: Option<LitStr>,
}

/*
    Parses the following:
    "parameter_name": ParameterType, "optional comment"
*/
impl Parse for StructParameterField {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let parameter_name: LitStr = input.parse()?;
        input.parse::<Token![:]>()?;
        let parameter_type: syn::Type = input.parse()?;

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

impl TemplateStruct for StructParameterField {
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

impl StructParameterField {
    pub fn to_tokens(&self) -> proc_macro2::TokenStream {

        let no_com = LitStr::new("", proc_macro2::Span::call_site());

        let name = lit_str_to_ident(&self.parameter_name).unwrap(); // TODO error handling
        let par_type = &self.parameter_type;
        let comment = self.description.clone().unwrap_or(no_com);

        quote!{
            #[doc = #comment]
            pub #name : #par_type,
        }
    }
}
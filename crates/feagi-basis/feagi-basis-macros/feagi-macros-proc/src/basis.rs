use quote::quote;
use syn::bracketed;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{LitStr, Token};
use crate::templates::template_root::TemplateRoot;

/// If a comma exists, parse it. If not, dont die
pub fn parse_optional_comma(input: ParseStream) -> syn::Result<()> {
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }
    Ok(())
}

/// Used to process "Ident : $SomeData" into SomeData
pub fn parse_property_colon_member<FieldIdent: Parse, FieldType: Parse>(buffer: &ParseBuffer) -> syn::Result<FieldType> {
    buffer.parse::<FieldIdent>()?;
    buffer.parse::<Token![:]>()?;
    let output = buffer.parse::<FieldType>()?;
    parse_optional_comma(&buffer)?;
    Ok(output)
}

pub fn lit_str_to_ident(lit_str: &LitStr) -> syn::Result<syn::Ident> {
    lit_str.parse::<syn::Ident>()
}

pub fn ident_to_lit_str(ident: &syn::Ident) -> LitStr {
    LitStr::new(&ident.to_string(), ident.span())
}

pub fn prepend_to_lit_str(prefix: &str, given: &LitStr) -> LitStr {
    let combined_value = format!("{}{}", prefix, given.value());
    LitStr::new(&combined_value, given.span())
}

pub fn append_to_lit_str(given: &LitStr, postfix: &str) -> LitStr {
    let combined_value = format!("{}{}", given.value(), postfix);
    LitStr::new(&combined_value, given.span())
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
pub struct StructParameters(Vec<StructParameterField>);

impl StructParameters {
    pub(crate) fn fields(&self) -> &[StructParameterField] {
        &self.0
    }

    pub(crate) fn to_vec(self) -> Vec<StructParameterField> {self.0}

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
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

//endregion
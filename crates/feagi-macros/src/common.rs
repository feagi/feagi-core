use proc_macro2::Ident;
use syn::bracketed;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

/// If a comma exists, parse it. If not, dont die
pub fn parse_optional_comma(input: ParseStream) -> syn::Result<()> {
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }
    Ok(())
}

//region Struct Builder

/// Represents all members (`StructBuilderParameterField`) to make a struct
pub struct StructBuilderParameters(Vec<StructBuilderParameterField>);

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


//endregion
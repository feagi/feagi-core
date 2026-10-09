use quote::quote;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{LitStr, Token};

//region Parsing

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

//endregion

//region Conversion and Adaptions

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

/// Visibility written onto generated Rust items.
#[derive(Clone, Copy)]
pub enum RustVisibility {
    /// `pub`
    Public,
    /// `pub(crate)`
    PubCrate,
    /// No visibility keyword. The item is private to its parent module.
    Private,
}

impl RustVisibility {
    /// Emits `pub`, `pub(crate)`, or an empty stream for [`RustVisibility::Private`].
    pub fn to_token_stream(self) -> proc_macro2::TokenStream {
        match self {
            RustVisibility::Public => quote!(pub),
            RustVisibility::PubCrate => quote!(pub(crate)),
            RustVisibility::Private => quote!(),
        }
    }
}

//endregion

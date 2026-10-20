use quote::quote;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::{token, LitStr, Token, Type, TypeTuple};
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
    parse_optional_comma(buffer)?;
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

pub fn unit_type() -> syn::Type {
    Type::Tuple(TypeTuple {
        attrs: vec![],
        paren_token: token::Paren::default(),
        elems: Punctuated::new(),
    })
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

//region Generated Code Paths

// NOTE: Generated code names everything by absolute path through `feagi_basis`, so it compiles
// wherever a template is defined without the caller importing anything. `feagi_basis` declares
// `extern crate self as feagi_basis` so these paths also resolve inside of it

/// Absolute path to the `feagi_basis` crate
pub fn basis_path() -> proc_macro2::TokenStream {
    quote!(::feagi_basis)
}

/// Clone, Debug, and serde derives, with serde reached through `feagi_basis` so the caller does
/// not need to depend on serde directly
pub fn serde_data_derive_attributes() -> proc_macro2::TokenStream {
    let basis = basis_path();
    quote! {
        #[derive(Clone, Debug, #basis::__private::serde::Serialize, #basis::__private::serde::Deserialize)]
        #[serde(crate = "::feagi_basis::__private::serde")]
    }
}

/// Wraps `alloc_items` so they only exist when `feagi_basis` is built with its `alloc` feature,
/// and `no_alloc_items` so they only exist when it is not
pub fn alloc_gated(alloc_items: proc_macro2::TokenStream, no_alloc_items: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let basis = basis_path();
    quote! {
        #basis::__feagi_if_alloc! {
            { #alloc_items }
            else
            { #no_alloc_items }
        }
    }
}

//endregion

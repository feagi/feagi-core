use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{LitStr, Type};
use syn::parse::{Parse, ParseStream};
use crate::template_parsing::struct_template::StructTemplate;
use crate::templates::Unparse;


mod kw {

    syn::custom_keyword!(path);


}

//region Feagi Message


/// Parses over a FeagiMessage with a payload. Formatted as the following
/// "MessageTitle": {
///     path: `FeagiMessagePath`,
///     description: "Description String",
///     payload: {
///         property_name: Type, "optional_comment",
///     },
///     response: {
///         property_name: Type, "optional_comment",
///     }
/// }
pub struct FeagiMessageWithPayload {
    path: FeagiMessagePath,
    description: LitStr,
    payload: StructTemplate,
    response: StructTemplate
}

impl Parse for FeagiMessageWithPayload {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

impl Unparse for FeagiMessageWithPayload {
    fn unparse(&self) -> TokenStream {
        todo!()
    }
}

//endregion

//region FeagiMessagePath

/// Parses over a single string to get the pathing out, with the formatting:
/// "static_1/static_2/:parameter_1<ParameterType1>/static_3?query_1<QueryType1>&query_2<QueryType2>"
pub struct FeagiMessagePath {
    leaf: Vec<MessagePathElement>
}

impl FeagiMessagePath {
    pub fn full_message_path(&self, category: &LitStr) -> LitStr {
        let path = self.as_lit_str();
        LitStr::new(&format!("{}/{}", category.value(), path.value()), Span::call_site())
    }

    pub fn as_lit_str(&self) -> LitStr {
        let string_out: String = self.leaf
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("/");
       LitStr::new(&string_out, Span::call_site())
    }
}

impl Parse for FeagiMessagePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_str: LitStr = input.parse()?;
        let path_parts: Vec<LitStr> = path_str
            .value()
            .split('/')
            .map(|s| LitStr::new(s, Span::call_site())).collect();


        // TODO parse this string

    }
}

impl Unparse for FeagiMessagePath {
    fn unparse(&self) -> proc_macro2::TokenStream {
        let string_out = self.as_lit_str();
        quote! { #string_out }
    }
}

pub enum MessagePathElement {
    StaticPath(LitStr),
    ParameterOfName(LitStr, Type),
    QueryableOfName(LitStr, Type)
}

impl MessagePathElement {
    pub fn to_string(&self) -> String {
        match self {
            MessagePathElement::StaticPath(s) => {s.value()}
            MessagePathElement::ParameterOfName(s, t) => {format!("{}<{}>", s.value(), quote!(#t).to_string())}
            MessagePathElement::QueryableOfName(s, t) => {format!("{}<{}>", s.value(), quote!(#t).to_string())}
        }
    }
}

//endregion

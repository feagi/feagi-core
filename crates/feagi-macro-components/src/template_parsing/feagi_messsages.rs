use proc_macro2::{Span};
use quote::quote;
use syn::{LitStr, Type};
use syn::parse::{Parse, ParseStream};
use crate::template_parsing::struct_builder::StructParameters;
use crate::templates::TemplateStruct;




//region FeagiMessagePathLeaf

/// Parses over a single string to get the pathing out, with the formatting:
/// "static_1/static_2/:parameter_1<ParameterType1>/static_3?query_1<QueryType1>&query_2<QueryType2>"
pub struct FeagiMessagePathLeaf {
    leaf: Vec<MessagePathElement>
}

impl FeagiMessagePathLeaf {
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

impl Parse for FeagiMessagePathLeaf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_str: LitStr = input.parse()?;
        let path_parts: Vec<LitStr> = path_str
            .value()
            .split('/')
            .map(|s| LitStr::new(s, Span::call_site())).collect();


        // TODO parse this string

    }
}

impl TemplateStruct for FeagiMessagePathLeaf {
    fn expand_template(&self) -> proc_macro2::TokenStream {
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

//region FeagiMessagePayload


pub struct FeagiMessagePayload(StructParameters);


//endregion
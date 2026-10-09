use syn::{LitStr, Type};
use quote::quote;

/// A singular element of a MessagePath or RequestMessagePath
pub enum MessagePathElement {
    StaticPath(LitStr),
    ParameterOfName(LitStr, Type),
    QueryableOfName(LitStr, Type),
}

impl MessagePathElement {
    pub fn to_string(&self) -> String {
        match self {
            MessagePathElement::StaticPath(s) => s.value(),
            MessagePathElement::ParameterOfName(s, t) => {
                format!("{}<{}>", s.value(), quote!(#t).to_string())
            }
            MessagePathElement::QueryableOfName(s, t) => {
                format!("{}<{}>", s.value(), quote!(#t).to_string())
            }
        }
    }

    pub fn as_rust_message_path_element_enum(&self) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();

        match self {
            MessagePathElement::StaticPath(s) => {
                output.extend(quote!{ MessagePathElement::StaticPath(#s), });
                output
            }
            MessagePathElement::ParameterOfName(s, _) => {
                output.extend(quote!{ MessagePathElement::ParameterOfName(#s), });
                output
            }
            MessagePathElement::QueryableOfName(s, _) => {
                output.extend(quote!{ MessagePathElement::QueryableOfName(#s), });
                output
            }
        }

    }
}
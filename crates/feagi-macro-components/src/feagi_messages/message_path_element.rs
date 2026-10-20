use quote::quote;
use syn::{LitStr, Type};

/// A singular element of a MessagePath or RequestMessagePath
pub enum MessagePathElement {
    /// Fixed at compile time
    StaticPath(LitStr),
    /// Fixed for the lifetime of the program, but only known at runtime. Its value comes from
    /// the category configurable of the same name
    ConfigurableOfName(LitStr),
    /// Parsed from the path of each request
    ParameterOfName(LitStr, Type),
    /// Parsed from the query of each request
    QueryableOfName(LitStr, Type),
}

impl MessagePathElement {
    pub fn to_string(&self) -> String {
        match self {
            MessagePathElement::StaticPath(s) => s.value(),
            MessagePathElement::ConfigurableOfName(s) => format!("${}", s.value()),
            MessagePathElement::ParameterOfName(s, t) => {
                format!(":{}<{}>", s.value(), quote!(#t).to_string())
            }
            MessagePathElement::QueryableOfName(s, t) => {
                format!("{}<{}>", s.value(), quote!(#t).to_string())
            }
        }
    }

    /// True for elements that are part of the path itself, rather than its query
    pub fn is_path_segment(&self) -> bool {
        !matches!(self, MessagePathElement::QueryableOfName(..))
    }
}

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
}

impl MessagePathElement {
    /// Copies this element
    pub fn duplicate(&self) -> Self {
        match self {
            MessagePathElement::StaticPath(text) => MessagePathElement::StaticPath(text.clone()),
            MessagePathElement::ConfigurableOfName(name) => MessagePathElement::ConfigurableOfName(name.clone()),
            MessagePathElement::ParameterOfName(name, parameter_type) => MessagePathElement::ParameterOfName(name.clone(), parameter_type.clone()),
        }
    }
}

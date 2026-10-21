//! Const descriptions of messages, for generating documentation (such as OpenAPI) of the
//! messages without needing the types themselves

/// The JSON kind of a field, as far as documentation needs it
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeagiFieldKind {
    Boolean,
    Integer,
    Number,
    String,
    Array,
    /// Any other type, including structs and enums
    Object,
}

/// One field of a message's payload or response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeagiFieldDescription {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub kind: FeagiFieldKind,
    /// False for `Option` fields, which may be left out
    pub required: bool,
}

/// One element of a message path
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeagiMessagePathElementDescription {
    /// Fixed at compile time
    Static(&'static str),
    /// Set at runtime, from the category configurable named `name`
    Configurable { name: &'static str },
    /// Parsed from each request, into the parameter field `name`
    Parameter {
        name: &'static str,
        description: Option<&'static str>,
        kind: FeagiFieldKind,
    },
}

use crate::feagi_message::feagi_message::FeagiMessage;

/// Represents the actual path that a message takes
pub trait FeagiMessagePath: Sized + Clone + Copy + core::fmt::Debug {
    /// The actual path of the message
    const PATH: &'static [MessagePathElement];

    /// The message data struct that is relevant
    type Message: FeagiMessage<Path = Self>;
}

/// Individual path segment
#[derive(Debug, Copy, Clone)]
pub enum MessagePathElement {
    StaticPath(&'static str),
    ParameterOfName(&'static str),
    QueryableOfName(&'static str),
}

impl MessagePathElement {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessagePathElement::StaticPath(s) => s,
            MessagePathElement::ParameterOfName(s) => s,
            MessagePathElement::QueryableOfName(s) => s,
        }
    }
}

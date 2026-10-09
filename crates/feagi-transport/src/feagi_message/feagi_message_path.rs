use crate::feagi_message::feagi_message::{FeagiMessage, FeagiMessageRequestMethod};

/// Represents the actual path that a message takes
pub trait FeagiMessagePath: Sized + Clone + Copy + core::fmt::Debug
{
    /// The actual path of the message
    const PATH: &'static [MessagePathElement];

    /// The message data struct that is relevant
    type Message: FeagiMessage<Path=Self>;
}

/// In the case of a Request Message, we have a request method and expect some form of response
pub trait FeagiRequestMessagePath: FeagiMessagePath {
    /// What type of Request method is being used
    const METHOD: FeagiMessageRequestMethod;
    
    /// Returns true if the given request type allows payloads (AKA is not Read/GET)
    const IS_PAYLOAD_ALLOWED: bool = Self::METHOD.is_payload_allowed();
}

/// Individual path segment
#[derive(Debug, Copy, Clone)]
pub enum MessagePathElement {
    StaticPath(&'static str),
    ParameterOfName(&'static str),
    QueryableOfName(&'static str)
}

impl MessagePathElement {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessagePathElement::StaticPath(s) => {s}
            MessagePathElement::ParameterOfName(s) => {s}
            MessagePathElement::QueryableOfName(s) => {s}
        }
    }
}
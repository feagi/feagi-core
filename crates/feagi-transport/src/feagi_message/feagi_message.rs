use std::fmt::Formatter;
use crate::feagi_message::feagi_message_specifications::{FeagiMessageParameters, FeagiMessagePayload, FeagiMessageQueryables, FeagiMessageResponse};


/// Represents all the data needed for a specific FEAGI Message, as individual fields
pub trait FeagiMessage: Sized + Clone + core::fmt::Debug {
    type Path: FeagiMessagePath; // Reverse matches the Message field inside
    type Parameters: FeagiMessageParameters;
    type Queryables: FeagiMessageQueryables;
    type Payload: FeagiMessagePayload;

    /// Compose from separate specifications of data
    fn from_message_data(parameters: Self::Parameters, queryables: Self::Queryables, payload: Self::Payload) -> Self;

    /// Decompose into the specifications of data needed by the FEAGI Message
    fn to_message_data(self) -> (Self::Parameters, Self::Queryables, Self::Payload);

    // NOTE: Generated struct should have the members across all specifications accessible as pub
    // (this does NOT include response members)
}

/// Includes the response type of a FeagiMessage if a response is expected
pub trait FeagiMessageWithResponse: FeagiMessage {
    type Response: FeagiMessageResponse;
}

/// Represents the actual path that a message takes
pub trait FeagiMessagePath: Sized + Clone + Copy + core::fmt::Debug
{
    /// The actual path of the message
    const PATH: &'static [MessagePathElement];

    type Message: FeagiMessage<Path=Self>;
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
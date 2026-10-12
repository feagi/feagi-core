use crate::feagi_message::feagi_message_path::FeagiMessagePath;
use crate::feagi_message::request_response::feagi_request_message_specifications::{FeagiMessageParameters, FeagiMessagePayload, FeagiMessageQueryables};

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
}

// TODO zenoh level rety policy, priority. Implement as consts in the trait with defaults
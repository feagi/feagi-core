use crate::feagi_message::feagi_message_path::{FeagiMessagePath, FeagiRequestMessagePath};
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
}

/// In the case of a request type message, a response is expected
pub trait FeagiRequestMessage: FeagiMessage<Path: FeagiRequestMessagePath> {
    /// The response struct we expect (could be null type potentially)
    type Response: FeagiMessageResponse;
    
    /// The type of request method being used
    const METHOD: FeagiMessageRequestMethod = Self::Path::METHOD;

    /// Returns true if the given request type allows payloads (AKA is not Read/GET)
    const IS_PAYLOAD_ALLOWED: bool = Self::METHOD.is_payload_allowed();

    // NOTE: Generated struct should NOT include response members
}


/// Defines what type of method (REST equivilent) a FEAGI Request message is using
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FeagiMessageRequestMethod {
    Read = 0,
    Create = 1,
    Edit = 2,
    Delete = 3,
    Patch = 4
}

impl FeagiMessageRequestMethod {
    
    /// Returns true if a payload is allowed
    pub const fn is_payload_allowed(&self) -> bool {
        match self { 
            Self::Read => false,
            _ => true
        }
    }
}
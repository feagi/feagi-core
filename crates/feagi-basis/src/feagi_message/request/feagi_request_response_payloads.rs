/// The struct that gets sent as the main request payload
pub trait FeagiRequestPayload: serde::Serialize + serde::de::DeserializeOwned {
    
}

// Messages without any request payload fields use the unit type
impl FeagiRequestPayload for () {}


/// The struct that gets returned as a response (assuming no error, which is handled separately)
pub trait FeagiResponsePayload: serde::Serialize + serde::de::DeserializeOwned {

}

// Messages without any response fields use the unit type
impl FeagiResponsePayload for () {}


//region Error
/// The error a specific message can respond with, when the request reached its responder but
/// could not be fulfilled. Errors of the channel / backend the request was sent over are
/// separate. Serialized as a plain serde enum, so it reads the same in json for every message
pub trait FeagiMessageError: Sized + core::error::Error + serde::Serialize + serde::de::DeserializeOwned {
    /// Error to return if not implemented
    const NOT_IMPLEMENTED: Self;
}

/// Error of messages that define no error of their own, as they are unlikely to ever fail
#[derive(Clone, Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum FeagiUnexpectedError {
    #[error("Unexpected Error")]
    UnexpectedError,
    #[error("This request is not implemented")]
    NotImplemented,
}

impl FeagiMessageError for FeagiUnexpectedError {
    const NOT_IMPLEMENTED: Self = FeagiUnexpectedError::NotImplemented;
}

//endregion
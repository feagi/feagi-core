/// The error a specific message can respond with, when the request reached its responder but
/// could not be fulfilled. Errors of the channel / backend the request was sent over are
/// separate. Serialized as a plain serde enum, so it reads the same in json for every message
pub trait FeagiMessageError: core::error::Error + serde::Serialize + serde::de::DeserializeOwned {}

/// Error of messages that define no error of their own, as they are unlikely to ever fail
#[derive(Clone, Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum FeagiUnexpectedError {
    #[error("Unexpected Error")]
    UnexpectedError,
}

impl FeagiMessageError for FeagiUnexpectedError {}

#[derive(thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
#[serde(bound(deserialize = "Processing: serde::de::DeserializeOwned"))]
pub enum FeagiResponseError<Processing: core::error::Error + serde::Serialize + serde::de::DeserializeOwned> { // TODO
    #[error("Failed to connect to make request")]
    NoConnection,
    #[error(transparent)]
    ProcessingError(Processing)
}
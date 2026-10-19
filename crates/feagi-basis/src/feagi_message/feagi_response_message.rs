/// The struct that gets returned as a response (assuming no error, which is handled separately)
pub trait FeagiResponsePayload: serde::Serialize + serde::de::DeserializeOwned {

}

#[derive(thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
#[serde(bound(deserialize = "Processing: serde::de::DeserializeOwned"))]
pub enum FeagiResponseError<Processing: core::error::Error + serde::Serialize + serde::de::DeserializeOwned> { // TODO
    #[error("Failed to connect to make request")]
    NoConnection,
    #[error(transparent)]
    ProcessingError(Processing)
}
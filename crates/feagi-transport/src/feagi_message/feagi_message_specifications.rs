
/// A struct that contains all parameters of a MessagePath
pub trait FeagiMessageParameters: Clone + core::fmt::Debug
+ serde::Serialize + serde::de::DeserializeOwned
{
    // All members of this struct need to be 'pub MessageElementSerializable'!
}

/// A struct that contains all queryables of a MessagePath
pub trait FeagiMessageQueryables: Clone + core::fmt::Debug
+ serde::Serialize + serde::de::DeserializeOwned
{
    // All members of this struct need to be 'pub MessageElementSerializable'!
}

/// A struct that contains all payloads that a MessagePath takes in. Note that "GET" style messages
/// do NOT use this
pub trait FeagiMessagePayload: Clone + core::fmt::Debug
+ serde::Serialize + serde::de::DeserializeOwned
{
    // All members of this struct need to be 'pub MessageElementSerializable'!
}

/// A struct that contains the response of a FeagiMessage (if relevant)
pub trait FeagiMessageResponse: Clone + core::fmt::Debug
+ serde::Serialize + serde::de::DeserializeOwned
{
    // All members of this struct need to be 'pub MessageElementSerializable'!
}

/// Used in place of when Parameters / Queryables / Payload is empty (or must be empty)
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct NullMessageSpecification;

impl FeagiMessageParameters for NullMessageSpecification {}

impl FeagiMessageQueryables for NullMessageSpecification {}

impl FeagiMessagePayload for NullMessageSpecification {}

impl FeagiMessageResponse for NullMessageSpecification {}







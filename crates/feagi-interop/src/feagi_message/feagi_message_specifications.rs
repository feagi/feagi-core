
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

// Empty generated specifications use the unit type.
impl FeagiMessageParameters for () {}

impl FeagiMessageQueryables for () {}

impl FeagiMessagePayload for () {}

impl FeagiMessageResponse for () {}







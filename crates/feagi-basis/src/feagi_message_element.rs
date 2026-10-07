// NOTE: This is here to allow easier implementation across the workspace

/// Allows a struct to be serialized / deserialized from a MessagePath parameter, query, or payload
pub trait MessageElementSerializable:
core::str::FromStr + Clone + core::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned
{
    // TODO conversion functions? Have them return errors if it fails to parse
}
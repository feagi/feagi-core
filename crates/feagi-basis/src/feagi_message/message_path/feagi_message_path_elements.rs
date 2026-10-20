//! These elements make up a Feagi message Path

use core::str::FromStr;

//region Top Level Traits

/// Makes up a part of the Message Path
pub trait MessagePathElement {}

/// Supports being deserialized from the string of a path directly
pub trait MessagePathElementDeserializable: FromStr + serde::de::DeserializeOwned {}

/*
/// Supports being deserialized from the string of a path directly with an exact known size
pub trait SizedMessagePathElementDeserializable: MessagePathElementDeserializable {
    /// The number of bytes / ascii characters
    const LENGTH: usize;
}
 */

/// Supports being serialized to the string of a path directly
pub trait MessagePathElementSerializable: serde::Serialize {}

/*
/// Supports being serialized to the string of a path directly with an exact number of bytes / ascii characters
pub trait SizedMessagePathElementSerializable: MessagePathElementSerializable {
    const NUM_CHAR: usize;
}

 */

//endregion


//region Message Path Elements

/// Defined a constant part of a path. Simply a &'static str slice
#[derive(Copy, Clone, serde::Serialize)]
pub struct MessagePathElementConst(pub &'static str);

impl MessagePathElementSerializable for MessagePathElementConst {}











//endregion
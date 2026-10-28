use base64::Engine;
use crate::node::feagi_node_state::FeagiNodeState;
use crate::node::feagi_role::FeagiRoles;

/// The version of a given node
pub type FeagiNodeVersion = u16;
pub const NUMBER_OF_BYTES_NODE_FLAGS: usize = 6;

/// What node version the node with this code compiled will report itself as
pub const CURRENT_FEAGI_NODE_VERSION: FeagiNodeVersion = 0;

/// If nodes below (exclusive) this version are detected in the network, this node will refuse to start
pub const MINIMUM_SUPPORTED_INTERFACE_VERSION: FeagiNodeVersion = 0;

/// Length of the padded URL-safe base64 text of [FeagiNodeFlags] (6 bytes -> 8 characters)
#[allow(clippy::manual_div_ceil)]
pub const NUMBER_OF_BASE_64_BYTES_NODE_FLAGS: usize = (NUMBER_OF_BYTES_NODE_FLAGS + 2) / 3 * 4;

/// Denotes the version, state, and roles this given node supports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeagiNodeFlags([u8; NUMBER_OF_BYTES_NODE_FLAGS]);

impl FeagiNodeFlags {

    // Bytes 0, 1 specify version
    // Bytes 2, 3 specify roles
    // Bytes 4 specify general device state
    // Byte 5 Reserved

    /// Creates a new struct with the current version
    pub fn new() -> Self {
        let mut out = [0u8; 6];
        let bytes: [u8; 2] = CURRENT_FEAGI_NODE_VERSION.to_be_bytes();
        out[0] = bytes[0];
        out[1] = bytes[1];
        Self(out)
    }

    pub fn get_version(&self) -> FeagiNodeVersion {
        let slice = [self.0[0], self.0[1]];
        u16::from_be_bytes(slice)
    }

    // cannot "set" version, thats defined at compile time

    pub fn get_roles(&self) -> FeagiRoles {
        FeagiRoles::from_be_bytes([self.0[2], self.0[3]])
    }

    pub fn set_roles(&mut self, roles: FeagiRoles) {
        let bytes: [u8; 2] = roles.to_be_bytes();
        self.0[2] = bytes[0];
        self.0[3] = bytes[1];
    }

    pub fn get_state(&self) -> FeagiNodeState {
        FeagiNodeState::new(self.0[4])
    }

    pub fn set_state(&mut self, state: FeagiNodeState) {
        self.0[4] = state.bits()
    }
}

impl Default for FeagiNodeFlags {
    fn default() -> Self {
        Self::new()
    }
}

//region Base64

impl FeagiNodeFlags {
    /// Encode the raw bytes as padded URL-safe base64
    pub fn to_base64(&self) -> heapless::String<NUMBER_OF_BASE_64_BYTES_NODE_FLAGS> {
        let mut arr = [0u8; NUMBER_OF_BASE_64_BYTES_NODE_FLAGS];
        // NUMBER_OF_BASE_64_BYTES_NODE_FLAGS is the padded length, so this write fills `arr`
        _ = base64::prelude::BASE64_URL_SAFE
            .encode_slice(self.0, &mut arr)
            .expect("node flags encode into NUMBER_OF_BASE_64_BYTES_NODE_FLAGS characters");
        let vec = heapless::Vec::from_array(arr);
        // SAFETY: every byte was written by BASE64_URL_SAFE, whose alphabet is ASCII
        unsafe { heapless::String::from_utf8_unchecked(vec) }
    }

    /// Parse padded URL-safe base64 text. Must be the canonical encoding of exactly
    /// NUMBER_OF_BYTES_NODE_FLAGS bytes
    pub fn try_from_base64(text: &str) -> Result<Self, FeagiNodeFlagsBase64Error> {
        if text.len() != NUMBER_OF_BASE_64_BYTES_NODE_FLAGS {
            return Err(FeagiNodeFlagsBase64Error::WrongLength {
                actual: text.len(),
                expected: NUMBER_OF_BASE_64_BYTES_NODE_FLAGS,
            });
        }

        let mut decoded = [0u8; NUMBER_OF_BYTES_NODE_FLAGS];
        let decoded_len = base64::prelude::BASE64_URL_SAFE
            .decode_slice(text.as_bytes(), &mut decoded)
            .map_err(|_| FeagiNodeFlagsBase64Error::InvalidEncoding)?;
        // Padding can make text of the right length decode to fewer bytes, only an exact decode is canonical
        if decoded_len != NUMBER_OF_BYTES_NODE_FLAGS {
            return Err(FeagiNodeFlagsBase64Error::InvalidEncoding);
        }
        Ok(Self(decoded))
    }
}

impl core::fmt::Display for FeagiNodeFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.to_base64().as_str())
    }
}

impl core::str::FromStr for FeagiNodeFlags {
    type Err = FeagiNodeFlagsBase64Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::try_from_base64(text)
    }
}

impl TryFrom<&str> for FeagiNodeFlags {
    type Error = FeagiNodeFlagsBase64Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Self::try_from_base64(text)
    }
}

impl feagi_basis::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement for FeagiNodeFlags {
    const KIND: feagi_basis::feagi_message::feagi_message_description::FeagiFieldKind =
        feagi_basis::feagi_message::feagi_message_description::FeagiFieldKind::String;

    fn write_path_segment<W: core::fmt::Write>(&self, out: &mut W) -> core::fmt::Result {
        out.write_str(self.to_base64().as_str())
    }
}

impl serde::Serialize for FeagiNodeFlags {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_base64().as_str())
    }
}

impl<'de> serde::Deserialize<'de> for FeagiNodeFlags {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FeagiNodeFlagsVisitor;

        impl<'de> serde::de::Visitor<'de> for FeagiNodeFlagsVisitor {
            type Value = FeagiNodeFlags;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("padded URL-safe base64 Feagi node flags")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                FeagiNodeFlags::try_from_base64(value).map_err(E::custom)
            }

            // Owned input cannot be borrowed. Copying it needs an allocator.
            #[cfg(feature = "alloc")]
            fn visit_string<E: serde::de::Error>(self, value: alloc::string::String) -> Result<Self::Value, E> {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_str(FeagiNodeFlagsVisitor)
    }
}

/// Failure to parse [FeagiNodeFlags] from base64 text
#[derive(Debug, thiserror::Error)]
pub enum FeagiNodeFlagsBase64Error {
    /// The text was not the padded URL-safe length of the node flags
    #[error("Feagi node flags base64 text length {actual} did not match padded length {expected}")]
    WrongLength { actual: usize, expected: usize },

    /// The text was not a canonical padded URL-safe encoding of the node flags
    #[error("Feagi node flags base64 text was not a canonical padded URL-safe encoding")]
    InvalidEncoding,
}

//endregion


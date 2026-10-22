use crate::FeagiNetworkError;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use feagi_serialization::{AgentIdentifier, FeagiByteContainer};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Agent identifier: 48-byte AgentDescriptor (instance_id(4) + manufacturer(20) + agent_name(20) + version(4)).
/// Single format - no 8-byte legacy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentID {
    bytes: [u8; AgentID::NUMBER_BYTES],
}

impl Serialize for AgentID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_base64())
    }
}

impl<'de> Deserialize<'de> for AgentID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from_base64(&s).map_err(serde::de::Error::custom)
    }
}

impl AgentID {
    pub const NUMBER_BYTES: usize = FeagiByteContainer::AGENT_ID_BYTE_COUNT;

    pub fn new(bytes: [u8; AgentID::NUMBER_BYTES]) -> Self {
        Self { bytes }
    }

    pub const fn new_blank() -> Self {
        Self {
            bytes: [0; AgentID::NUMBER_BYTES],
        }
    }

    pub fn new_random() -> Self {
        let mut bytes = [0u8; AgentID::NUMBER_BYTES];
        getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
        Self { bytes }
    }

    /// Attempts to create an AgentID from a base64-encoded string.
    ///
    /// # Arguments
    ///
    /// * `base64_str` - A base64-encoded string representing the agent ID bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The string is not valid base64
    /// - The decoded bytes length doesn't match `NUMBER_BYTES`
    pub fn try_from_base64(base64_str: &str) -> Result<Self, FeagiNetworkError> {
        let decoded = BASE64_STANDARD
            .decode(base64_str)
            .map_err(|e| FeagiNetworkError::GeneralFailure(format!("Invalid base64: {}", e)))?;

        if decoded.len() != Self::NUMBER_BYTES {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "Invalid AgentID length: expected {} bytes, got {}",
                Self::NUMBER_BYTES,
                decoded.len()
            )));
        }

        let mut bytes = [0u8; Self::NUMBER_BYTES];
        bytes.copy_from_slice(&decoded);
        Ok(Self { bytes })
    }

    pub fn is_blank(&self) -> bool {
        self.bytes == [0; AgentID::NUMBER_BYTES]
    }

    pub fn bytes(&self) -> &[u8; AgentID::NUMBER_BYTES] {
        &self.bytes
    }

    /// Encodes the agent ID bytes as a base64 string.
    pub fn to_base64(&self) -> String {
        BASE64_STANDARD.encode(self.bytes)
    }
}

impl AgentIdentifier for AgentID {
    fn get_identifier_bytes(&self) -> &[u8; FeagiByteContainer::AGENT_ID_BYTE_COUNT] {
        &self.bytes
    }
}

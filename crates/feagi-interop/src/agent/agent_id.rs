use core::fmt::{Display, Formatter};
use feagi_basis::base_64_id;
use feagi_basis::byte_id::{Base64EncodedByteID, ByteID};
use thiserror::Error;

pub const NUMBER_OF_BYTES: usize = 8;

/// Unique session tracker of an agent registered to FEAGI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AgentID([u8; NUMBER_OF_BYTES]);

impl Display for AgentID {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Base64EncodedAgentID::fmt_id(&self, f)
    }
}

impl ByteID<NUMBER_OF_BYTES> for AgentID {
    type DeserializationError = AgentIDError;

    fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES] {
        &self.0
    }

    fn try_from_id_bytes(bytes: [u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError> {
        // No specific pattern here
        Ok(Self(bytes))
    }
}

#[derive(Debug, Error)]
pub enum AgentIDError {
    // TODO
}

base_64_id!(AgentID, 8);

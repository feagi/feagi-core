use core::fmt::{Display, Formatter};
use feagi_basis::base_64_id;
use feagi_basis::identifiers::byte_id::{Base64EncodedByteID, ByteID};
use thiserror::Error;

pub const NUMBER_OF_BYTES: usize = 8;

/// Unique session tracker of an embodiment registered to FEAGI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EmbodimentID([u8; NUMBER_OF_BYTES]);

impl Display for EmbodimentID {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Base64EncodedEmbodimentID::fmt_id(&self, f)
    }
}

impl ByteID<NUMBER_OF_BYTES> for EmbodimentID {
    type DeserializationError = EmbodimentIDError;

    fn get_id_bytes(&self) -> &[u8; NUMBER_OF_BYTES] {
        &self.0
    }

    fn try_from_id_bytes(bytes: [u8; NUMBER_OF_BYTES]) -> Result<Self, Self::DeserializationError> {
        // No specific pattern here
        Ok(Self(bytes))
    }
}

#[derive(Debug, Error)]
pub enum EmbodimentIDError {
    #[error("No decoding error possible as there are no rules")]
    None
}

base_64_id!(EmbodimentID, 8);

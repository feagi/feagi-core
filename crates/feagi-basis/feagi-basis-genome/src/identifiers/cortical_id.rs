use std::fmt::{Display, Formatter};
use std::str::FromStr;
#[cfg(feature = "base64")]
use base64::engine::general_purpose;
#[cfg(feature = "base64")]
use base64::Engine;
use crate::identifiers::feagi_identifier_error::{FeagiFailCorticalID, FeagiGenomeIdenfitierError};
use crate::identifiers::identifiers::{BrainGraphIdentifier, GenomeIdentifier, ImposterIdentifier};

macro_rules! match_bytes_by_cortical_type {
    ($cortical_id_bytes: expr,
        custom => $custom:block,
        memory => $memory:block,        core => $core:block,

        brain_input => $brain_input:block,
        brain_output => $brain_output:block,
        invalid => $invalid:block,
    ) => {
        match $cortical_id_bytes[0] {
            b'c' => $custom,
            b'm' => $memory,
            b'_' => $core,
            b'i' => $brain_input,
            b'o' => $brain_output,
            _ => $invalid,
        }
    };
}

// TODO move directly to a wrapped u64 to help avoid  endian shenanigans

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CorticalID {
    pub(crate) bytes: [u8; CorticalID::CORTICAL_ID_LENGTH],
}

impl CorticalID {
    pub const CORTICAL_ID_LENGTH: usize = 8; // 8 bytes -> 64 bit
    pub const CORTICAL_ID_LENGTH_BASE_64: usize = 4 * (Self::CORTICAL_ID_LENGTH + 3); // enforces rounding up

    pub const NUMBER_OF_BYTES: usize = Self::CORTICAL_ID_LENGTH;

    //region Constructors

    pub fn try_from_bytes(bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<Self, FeagiGenomeIdenfitierError> {
        match_bytes_by_cortical_type!(bytes,
            custom => {
                Ok(CorticalID {bytes: *bytes})
            },
            memory => {
                Ok(CorticalID {bytes: *bytes})
            },
            core => {
                Ok(CorticalID {bytes: *bytes})
            },
            brain_input => {
                // TODO more checks
                Ok(CorticalID {bytes: *bytes})
            },
            brain_output => {
                // TODO more checks
                Ok(CorticalID {bytes: *bytes})
            },
            invalid => {
                Err
                (
                    FeagiFailCorticalID::new(
                        "Cortical ID bytes do not match a valid Cortical type prefix"
                    ).into()
                )
            },
        )
    }

    pub fn try_from_u64(u: u64) -> Result<Self, FeagiGenomeIdenfitierError> {
        let bytes = u.to_be_bytes();
        Self::try_from_bytes(&bytes)
    }

    #[cfg(feature = "base64")]
    pub fn try_from_base_64(str: &str) -> Result<Self, FeagiGenomeIdenfitierError> {
        let decoded = general_purpose::STANDARD
            .decode(str)
            .map_err(|_| FeagiFailCorticalID::new("failed to decode cortical area ID from base64").into())?;

        if decoded.len() != Self::CORTICAL_ID_LENGTH {
            return Err(FeagiFailCorticalID::new("Base 64 is wrong length for cortical ID").into());
        }

        let mut bytes = [0u8; Self::CORTICAL_ID_LENGTH];
        bytes.copy_from_slice(&decoded);
        Self::try_from_bytes(&bytes)
    }

    //endregion

    //region export

    pub fn write_id_to_bytes(&self, bytes: &mut [u8; Self::NUMBER_OF_BYTES]) {
        bytes.copy_from_slice(&self.bytes)
    }

    pub fn as_bytes(&self) -> &[u8; CorticalID::CORTICAL_ID_LENGTH] {
        &self.bytes
    }

    pub fn as_u64(&self) -> u64 {
        u64::from_be_bytes(self.bytes)
    }

    #[cfg(feature = "base64")]
    pub fn as_base_64_str(&self) -> String {
        general_purpose::STANDARD.encode(self.bytes)
    }

    /// Extract subtype from cortical_area ID (e.g., "isvi0___" → "svi")
    /// Returns None for CORE areas or if bytes are invalid UTF-8
    pub fn extract_subtype(&self) -> Option<String> {
        // For IPU/OPU areas, bytes 1-3 contain the subtype
        if self.bytes[0] == b'i' || self.bytes[0] == b'o' {
            // Extract bytes 1-3, trim trailing underscores/nulls
            let subtype_bytes = &self.bytes[1..4];
            String::from_utf8(subtype_bytes.to_vec())
                .ok()
                .map(|s| s.trim_end_matches('_').trim_end_matches('\0').to_lowercase())
                .filter(|s| !s.is_empty())
        } else {
            None
        }
    }

    /// Extract unit ID from cortical_area ID (typically byte 4)
    /// Returns None for CORE/CUSTOM/MEMORY areas
    pub fn extract_unit_id(&self) -> Option<u8> {
        if self.bytes[0] == b'i' || self.bytes[0] == b'o' {
            // Byte 4 typically contains unit ID (0-9 as ASCII)
            let byte = self.bytes[4];
            if byte.is_ascii_digit() {
                Some(byte - b'0')
            } else if byte == b'_' || byte == 0 {
                Some(0)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Extract group ID from cortical_area ID (similar to unit ID, but may be in different byte)
    /// For now, returns the same as unit_id
    pub fn extract_group_id(&self) -> Option<u8> {
        self.extract_unit_id()
    }

    //endregion

}

impl BrainGraphIdentifier for CorticalID {}

impl Display for CorticalID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl FromStr for CorticalID {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl GenomeIdentifier for CorticalID {}

/// Used to identify incoming cortical areas to be added and their relationship to each other, but
/// the relationship to the destination to be determined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ImposterCorticalID(u64);

impl ImposterCorticalID {
    pub fn new(id: u64) -> Self {
        ImposterCorticalID(id)
    }
}

impl BrainGraphIdentifier for ImposterCorticalID {}

impl Display for ImposterCorticalID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl FromStr for ImposterCorticalID {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl ImposterIdentifier for ImposterCorticalID { type Impostering = CorticalID; }


pub enum MaybeCorticalID {
    Genome(CorticalID),
    Imposter(ImposterCorticalID),
}



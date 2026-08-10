use crate::cortical_area::io_cortical_area_configuration_flag::IOCorticalAreaConfigurationFlag;
use crate::cortical_area::{CoreCorticalType, CorticalAreaType, CustomCorticalType, MemoryCorticalType};
use crate::feagi_genome_context_error::{FeagiCorticalIDErrKey, FeagiGenomeContextError};
use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// TODO remove base64!

macro_rules! match_bytes_by_cortical_type {
    ($cortical_id_bytes: expr,
        custom => $custom:block,
        memory => $memory:block,
        core => $core:block,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorticalID {
    pub(crate) bytes: [u8; CorticalID::CORTICAL_ID_LENGTH],
}

impl CorticalID {
    pub const CORTICAL_ID_LENGTH: usize = 8; // 8 bytes -> 64 bit
    pub const CORTICAL_ID_LENGTH_BASE_64: usize = 4 * (Self::CORTICAL_ID_LENGTH + 3); // enforces rounding up

    pub const NUMBER_OF_BYTES: usize = Self::CORTICAL_ID_LENGTH;

    //region Constructors

    pub fn try_from_bytes(bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<Self, FeagiGenomeContextError> {
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
                    FeagiCorticalIDErrKey::new(
                        "cortical_area ID bytes do not match a valid cortical_area type prefix"
                    ).into()
                )
            },
        )
    }

    pub fn try_from_u64(u: u64) -> Result<Self, FeagiGenomeContextError> {
        let bytes = u.to_be_bytes();
        Self::try_from_bytes(&bytes)
    }

    pub fn try_from_base_64(str: &str) -> Result<Self, FeagiGenomeContextError> {
        let decoded = general_purpose::STANDARD
            .decode(str)
            .map_err(|_| FeagiCorticalIDErrKey::new("failed to decode cortical area ID from base64").into())?;

        if decoded.len() != Self::CORTICAL_ID_LENGTH {
            return Err(FeagiCorticalIDErrKey::new("Base 64 is wrong length for cortical ID").into());
        }

        let mut bytes = [0u8; Self::CORTICAL_ID_LENGTH];
        bytes.copy_from_slice(&decoded);
        Self::try_from_bytes(&bytes)
    }

    /// Parse a legacy 6-char or 8-char ASCII cortical area ID, as written in v2 genome documents.
    ///
    /// Shipped genomes address areas by ASCII name (`c__lef`, `cRSMot`, `omot00`) rather than by
    /// the 8-byte encoding, so every reader of a v2 document needs this to resolve them. Shorter
    /// names are right-padded with `_` to the full 8 bytes, which is how the same names are
    /// spelled in 8-char form.
    ///
    /// An uppercase type prefix (`C`/`M`/`I`/`O`) is lowercased, and any first byte that names no
    /// type at all is read as a custom area, because that is the only type a v2 document can spell
    /// without encoding type metadata into the ID.
    pub fn try_from_legacy_ascii(id_str: &str) -> Result<Self, FeagiGenomeContextError> {
        let mut bytes = [b'_'; Self::CORTICAL_ID_LENGTH];
        let len = id_str.len().min(Self::CORTICAL_ID_LENGTH);
        bytes[..len].copy_from_slice(&id_str.as_bytes()[..len]);
        bytes[0] = match bytes[0] {
            b'C' => b'c',
            b'M' => b'm',
            b'I' => b'i',
            b'O' => b'o',
            b'c' | b'm' | b'_' | b'i' | b'o' => bytes[0],
            _ => b'c',
        };
        Self::try_from_bytes(&bytes)
    }
    //endregion

    //region export

    pub fn write_id_to_bytes(&self, bytes: &mut [u8; Self::NUMBER_OF_BYTES]) {
        bytes.copy_from_slice(&self.bytes)
    }

    /// Extract IO data type configuration from cortical_area ID bytes
    ///
    /// Extracts the data type configuration flag from bytes 4-5 (u16, little-endian)
    /// and converts it to an IOCorticalAreaDataFlag.
    ///
    /// This is used for both BrainInput and BrainOutput cortical_area areas.
    #[inline]
    pub fn extract_io_data_flag(&self) -> Result<IOCorticalAreaConfigurationFlag, FeagiGenomeContextError> {
        let data_type_config = u16::from_le_bytes([self.bytes[4], self.bytes[5]]);
        IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(data_type_config)
    }

    pub fn as_cortical_type(&self) -> Result<CorticalAreaType, FeagiGenomeContextError> {
        match_bytes_by_cortical_type!(self.bytes,
            custom => {
                // NOTE: Only 1 custom type currently
                Ok(CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire))
            },
            memory => {
                // NOTE: Only 1 memory type currently
                Ok(CorticalAreaType::Memory(MemoryCorticalType::Memory))
            },
            core => {
                Ok(CorticalAreaType::Core(CoreCorticalType::try_from_cortical_id_bytes_type_unchecked(&self.bytes)?))
            },
            brain_input => {
                Ok(CorticalAreaType::BrainInput(self.extract_io_data_flag()?))
            },
            brain_output => {
                Ok(CorticalAreaType::BrainOutput(self.extract_io_data_flag()?))
            },
            invalid => {
                Err(
                FeagiCorticalIDErrKey::new("cortical_area ID does not encode a valid cortical_area area type").into()
                )
            },
        )
    }

    pub fn as_bytes(&self) -> &[u8; CorticalID::CORTICAL_ID_LENGTH] {
        &self.bytes
    }

    pub fn as_u64(&self) -> u64 {
        u64::from_be_bytes(self.bytes)
    }

    pub fn as_base_64(&self) -> String {
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

    //region internal

    //endregion
}

impl core::fmt::Display for CorticalID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use base64 encoding for display instead of UTF-8 to avoid control characters
        write!(f, "{}", self.as_base_64())
    }
}

// Implement Serialize for CorticalID - uses base64 format for JSON compatibility
impl Serialize for CorticalID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as base64 string for JSON compatibility
        serializer.serialize_str(&self.as_base_64())
    }
}

// Implement Deserialize for CorticalID - accepts base64 format
impl<'de> Deserialize<'de> for CorticalID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        CorticalID::try_from_base_64(&s).map_err(|e| serde::de::Error::custom(format!("Invalid CorticalID: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_ascii_pads_six_char_names_to_full_width() {
        // Custom areas in the shipped v2 genomes are spelled this way; a six-char name and its
        // eight-char padded spelling name the same area and must resolve to the same ID.
        let padded = CorticalID::try_from_legacy_ascii("c__lef").expect("six-char custom name resolves");
        assert_eq!(padded.as_bytes(), b"c__lef__");
        assert_eq!(
            padded,
            CorticalID::try_from_legacy_ascii("c__lef__").expect("eight-char custom name resolves")
        );
    }

    #[test]
    fn legacy_ascii_lowercases_uppercase_type_prefix() {
        let id = CorticalID::try_from_legacy_ascii("C03bbb").expect("uppercase custom prefix resolves");
        assert_eq!(id.as_bytes()[0], b'c');
        assert_eq!(&id.as_bytes()[1..6], b"03bbb");
    }

    #[test]
    fn legacy_ascii_reads_unknown_prefix_as_custom() {
        for name in ["visioA", "visioB", "0_45de"] {
            let id = CorticalID::try_from_legacy_ascii(name).expect("unknown prefix resolves as custom");
            assert_eq!(id.as_bytes()[0], b'c', "'{name}' should be read as a custom area");
        }
    }

    #[test]
    fn legacy_ascii_preserves_recognised_type_prefixes() {
        for (name, expected_prefix) in [("omot00", b'o'), ("iic000", b'i'), ("m_epis", b'm'), ("_power", b'_')] {
            let id = CorticalID::try_from_legacy_ascii(name).expect("known prefix resolves");
            assert_eq!(id.as_bytes()[0], expected_prefix, "'{name}' should keep its type prefix");
        }
    }
}

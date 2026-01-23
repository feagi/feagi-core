use crate::genomic::cortical_area::cortical_area_type::{
    CoreCorticalType, CorticalAreaType, CustomCorticalType, MemoryCorticalType,
};
use crate::genomic::cortical_area::io_cortical_area_configuration_flag::IOCorticalAreaConfigurationFlag;
use crate::FeagiDataError;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;

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

    pub fn try_from_bytes(
        bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH],
    ) -> Result<Self, FeagiDataError> {
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
                Err(FeagiDataError::DeserializationError("Unable to deserialize cortical ID bytes as any possible type!".into()))
            },
        )
    }

    pub fn try_from_u64(u: u64) -> Result<Self, FeagiDataError> {
        let bytes = u.to_be_bytes();
        Self::try_from_bytes(&bytes)
    }

    pub fn try_from_base_64(str: &str) -> Result<Self, FeagiDataError> {
        let decoded = general_purpose::STANDARD.decode(str).map_err(|e| {
            FeagiDataError::DeserializationError(format!(
                "Failed to decode base64 string to cortical ID: {}",
                e
            ))
        })?;

        if decoded.len() != Self::CORTICAL_ID_LENGTH {
            return Err(FeagiDataError::DeserializationError(format!(
                "Invalid base64 cortical ID length: expected {} bytes, got {}",
                Self::CORTICAL_ID_LENGTH,
                decoded.len()
            )));
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

    /// Extract IO data type configuration from cortical ID bytes
    ///
    /// Extracts the data type configuration flag from bytes 4-5 (u16, little-endian)
    /// and converts it to an IOCorticalAreaDataFlag.
    ///
    /// This is used for both BrainInput and BrainOutput cortical areas.
    #[inline]
    pub fn extract_io_data_flag(&self) -> Result<IOCorticalAreaConfigurationFlag, FeagiDataError> {
        let data_type_config = u16::from_le_bytes([self.bytes[4], self.bytes[5]]);
        IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(data_type_config)
    }

    pub fn as_cortical_type(&self) -> Result<CorticalAreaType, FeagiDataError> {
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
                Err(FeagiDataError::InternalError("Attempted to convert an invalid cortical ID instantiated object to cortical type!".into()))
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

    /// Extract subtype from cortical ID (e.g., "isvi0___" → "svi")
    /// Returns None for CORE areas or if bytes are invalid UTF-8
    pub fn extract_subtype(&self) -> Option<String> {
        // For IPU/OPU areas, bytes 1-3 contain the subtype
        if self.bytes[0] == b'i' || self.bytes[0] == b'o' {
            // Extract bytes 1-3, trim trailing underscores/nulls
            let subtype_bytes = &self.bytes[1..4];
            String::from_utf8(subtype_bytes.to_vec())
                .ok()
                .map(|s| {
                    s.trim_end_matches('_')
                        .trim_end_matches('\0')
                        .to_lowercase()
                })
                .filter(|s| !s.is_empty())
        } else {
            None
        }
    }

    /// Extract unit ID from cortical ID (typically byte 4)
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

    /// Extract group ID from cortical ID (similar to unit ID, but may be in different byte)
    /// For now, returns the same as unit_id
    pub fn extract_group_id(&self) -> Option<u8> {
        self.extract_unit_id()
    }

    //endregion

    //region internal

    //endregion
}

impl Display for CorticalID {
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
        CorticalID::try_from_base_64(&s)
            .map_err(|e| serde::de::Error::custom(format!("Invalid CorticalID: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genomic::cortical_area::cortical_area_type::CoreCorticalType;

    #[test]
    fn test_u64_round_trip() {
        // Create a cortical ID from a core type
        let original_id = CoreCorticalType::Power.to_cortical_id();

        // Convert to u64
        let as_u64 = original_id.as_u64();

        // Convert back from u64
        let restored_id = CorticalID::try_from_u64(as_u64).unwrap();

        // Verify they're equal
        assert_eq!(original_id, restored_id);
        assert_eq!(original_id.as_bytes(), restored_id.as_bytes());
    }

    #[test]
    fn test_base64_round_trip() {
        // Create a cortical ID from a core type
        let original_id = CoreCorticalType::Death.to_cortical_id();

        // Convert to base64
        let as_base64 = original_id.as_base_64();

        // Convert back from base64
        let restored_id = CorticalID::try_from_base_64(&as_base64).unwrap();

        // Verify they're equal
        assert_eq!(original_id, restored_id);
        assert_eq!(original_id.as_bytes(), restored_id.as_bytes());
    }

    #[test]
    fn test_base64_length() {
        let id = CoreCorticalType::Power.to_cortical_id();
        let base64_str = id.as_base_64();

        // Base64 of 8 bytes should be 12 characters (with potential padding)
        // 8 bytes = 64 bits, base64 uses 6 bits per character
        // 64 / 6 = 10.67, rounded up to 11, but base64 padding rounds to multiple of 4 = 12
        assert!(base64_str.len() >= 11 && base64_str.len() <= 12);
    }

    #[test]
    fn test_invalid_base64() {
        // Test with invalid base64 string
        let result = CorticalID::try_from_base_64("not valid base64!");
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_wrong_length() {
        // Test with valid base64 but wrong length (only 4 bytes encoded)
        let short_base64 = general_purpose::STANDARD.encode([1u8, 2, 3, 4]);
        let result = CorticalID::try_from_base_64(&short_base64);
        assert!(result.is_err());
    }

    #[test]
    fn test_u64_with_various_core_types() {
        let core_types = [CoreCorticalType::Power, CoreCorticalType::Death];

        for core_type in &core_types {
            let id = core_type.to_cortical_id();
            let as_u64 = id.as_u64();
            let restored = CorticalID::try_from_u64(as_u64).unwrap();
            assert_eq!(id, restored, "Failed round-trip for {:?}", core_type);
        }
    }
}

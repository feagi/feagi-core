use std::fmt::{Display};
use base64::{Engine as _, engine::general_purpose};
use crate::FeagiDataError;
use crate::genomic::cortical_area::cortical_type::{CoreCorticalType, CorticalAreaType, CustomCorticalType, MemoryCorticalType};

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
    pub const CORTICAL_ID_LENGTH_BASE_64: usize = 4 * (Self::CORTICAL_ID_LENGTH + 3 - 1 / 3); // enforces rounding up

    pub const NUMBER_OF_BYTES: usize = Self::CORTICAL_ID_LENGTH;

    //region Constructors

    pub fn try_from_bytes(bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<Self, FeagiDataError> {
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
        let decoded = general_purpose::STANDARD.decode(str)
            .map_err(|e| FeagiDataError::DeserializationError(
                format!("Failed to decode base64 string to cortical ID: {}", e)
            ))?;
        
        if decoded.len() != Self::CORTICAL_ID_LENGTH {
            return Err(FeagiDataError::DeserializationError(
                format!("Invalid base64 cortical ID length: expected {} bytes, got {}", 
                    Self::CORTICAL_ID_LENGTH, decoded.len())
            ));
        }
        
        let mut bytes = [0u8; Self::CORTICAL_ID_LENGTH];
        bytes.copy_from_slice(&decoded);
        Self::try_from_bytes(&bytes)
    }
    //endregion

    //region export

    pub fn write_id_to_bytes(&self, bytes: &mut[u8; Self::NUMBER_OF_BYTES]) {
        bytes.copy_from_slice(&self.bytes)
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
                todo!()
            },
            brain_output => {
                todo!()
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
        general_purpose::STANDARD.encode(&self.bytes)
    }

    //endregion

    //region internal

    //endregion

}

impl Display for CorticalID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genomic::cortical_area::cortical_type::CoreCorticalType;

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
        let short_base64 = general_purpose::STANDARD.encode(&[1u8, 2, 3, 4]);
        let result = CorticalID::try_from_base_64(&short_base64);
        assert!(result.is_err());
    }

    #[test]
    fn test_u64_with_various_core_types() {
        let core_types = [
            CoreCorticalType::Power,
            CoreCorticalType::Death,
        ];

        for core_type in &core_types {
            let id = core_type.to_cortical_id();
            let as_u64 = id.as_u64();
            let restored = CorticalID::try_from_u64(as_u64).unwrap();
            assert_eq!(id, restored, "Failed round-trip for {:?}", core_type);
        }
    }
}


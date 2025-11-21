use std::fmt::{Display};
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
            b'_' => $core
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
        todo!()
    }

    pub fn try_from_base_64(str: &str) -> Result<Self, FeagiDataError> {
        todo!()
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
        todo!()
    }

    pub fn as_base_64(self) -> String {
        todo!()
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



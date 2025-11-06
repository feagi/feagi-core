use std::fmt::{Display};
use crate::FeagiDataError;
use crate::genomic::cortical_area::cortical_type::CorticalType;

macro_rules! match_bytes_by_cortical_type {
    ($cortical_id_bytes: expr,
        custom => $custom:block,
        memory => $memory:block,
        core => $core:block,
        brain_input => $brain_input:block,
        brain_output => $brain_output:block,
        invalid => $invalid:block,
    ) => {
        match *$cortical_id_bytes[0] {
            b"c" => $custom,
            b"m" => $memory,
            b"_" => $core
            b"i" => $brain_input,
            b"o" => $brain_output,
            _ => $invalid,
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorticalID {
    pub(crate) bytes: [u8; CorticalID::CORTICAL_ID_LENGTH],
}

impl CorticalID {

    pub const CORTICAL_ID_LENGTH: usize = 8; // 8 bytes -> 64 bit
    pub const CORTICAL_ID_LENGTH_BASE_64: usize = 4 * (Self::CORTICAL_ID_LENGTH + 3 - 1 / 3); // enforces rounding up

    pub const NUMBER_OF_BYTES: usize = Self::CORTICAL_ID_LENGTH;

    //region Constructors

    pub fn try_from_bytes(bytes: [u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<Self, FeagiDataError> {
        todo!()
    }

    pub fn try_from_u64(u: u64) -> Result<Self, FeagiDataError> {
        todo!()
    }

    pub fn try_from_base_64(str: &str) -> Result<Self, FeagiDataError> {
        todo!()
    }

    //endregion

    //region export

    pub fn as_cortical_type(&self) -> Result<CorticalType, FeagiDataError> {
        match_bytes_by_cortical_type!(self.bytes,
            custom => {},
            memory => {},
            core => {
                
            },
            brain_input => {},
            brain_output => {},
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



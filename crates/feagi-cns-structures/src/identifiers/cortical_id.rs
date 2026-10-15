use core::fmt::{Display, Formatter};
use feagi_basis::base_64_id;
use feagi_basis::identifiers::byte_id::{Base64EncodedByteID, ByteID};
use crate::identifiers::genome_identifier_error::GenomeIdentifierError;

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

/// A unique identifier for a cortical area in a genome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CorticalID([u8; Self::CORTICAL_ID_LENGTH]);

impl CorticalID {
    pub const CORTICAL_ID_LENGTH: usize = 8; // 8 bytes -> 64 bit
    
    pub const fn from_bytes_const(bytes: [u8; 8]) -> Self {
        match_bytes_by_cortical_type!(bytes,
            custom => {
                CorticalID(bytes)
            },
            memory => {
                CorticalID(bytes)
            },
            core => {
                CorticalID(bytes)
            },
            brain_input => {
                // TODO more checks
                CorticalID(bytes)
            },
            brain_output => {
                // TODO more checks
                CorticalID(bytes)
            },
            invalid => {
                panic!("Invalid Cortical ID!")
            },
        )
    }
}


impl ByteID<8> for CorticalID {
    type DeserializationError = GenomeIdentifierError;

    fn get_id_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    fn try_from_id_bytes(bytes: [u8; 8]) -> Result<Self, Self::DeserializationError> {

        match_bytes_by_cortical_type!(bytes,
            custom => {
                Ok(CorticalID(bytes))
            },
            memory => {
                Ok(CorticalID(bytes))
            },
            core => {
                Ok(CorticalID(bytes))
            },
            brain_input => {
                // TODO more checks
                Ok(CorticalID(bytes))
            },
            brain_output => {
                // TODO more checks
                Ok(CorticalID(bytes))
            },
            invalid => {
                Err
                (
                    GenomeIdentifierError::CannotParseFromBytes("CorticalID")
                )
            },
        )
    }
}

base_64_id!(CorticalID, 8);

impl Display for CorticalID {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
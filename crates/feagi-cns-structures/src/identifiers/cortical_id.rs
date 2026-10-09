use std::fmt::{Display, Formatter};
use base64::prelude::*;
use crate::identifiers::generic_id::GenericID;
use crate::identifiers::genome_identifier_error::GenomeIdentifierError;

pub const CORTICAL_ID_BASE_64_BYTE_COUNT: usize = 12; // Assumes padding


/// A unique identifier for a cortical area in a genome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CorticalID(u64);

/*
impl GenericID<u64> for CorticalID {
    const MAX_URL_PARAMETER_LENGTH: usize = CORTICAL_ID_BASE_64_BYTE_COUNT;

    fn from_backing(backing: &u64) -> Self {
        Self(*backing)
    }

    #[cfg(feature = "alloc")]
    fn try_new_from_url_parameter(parameter_section: &str) -> Result<Self, GenomeIdentifierError> {
        // TODO feature / hardware cfg to more optimal engines (probably make a reusable macro)
        let engine = base64::engine::general_purpose::URL_SAFE;
        let bytes = engine.decode(parameter_section).map_err(
            |_| return Err(GenomeIdentifierError::CannotParseURLParameter("Cortical"))
        )?;
        
        if bytes.len() != 8 {
            return Err(GenomeIdentifierError::CannotParseURLParameter("Cortical"))
        }

        // TODO do we really need all these copies? Cant we do a direct cast?
        
        let mut out_bytes = [0u8; 8];
        out_bytes.copy_from_slice(&bytes);
        Ok(Self(u64::from_be_bytes(out_bytes)))
    }

    fn get_backing(&self) -> &u64 {
        &self.0
    }
}


 */
impl Display for CorticalID {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
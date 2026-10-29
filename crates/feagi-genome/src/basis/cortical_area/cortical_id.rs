use core::hash::Hash;
use feagi_basis::feagi_id::feagi_id;
use crate::identifiers::faux_identifer::{FauxIdentifier, HasFauxIdentifier};
use crate::identifiers::genome_identifier_error::GenomeIdentifierError;

#[macro_export]
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

/// Cortical ID of some sort (faux or real)
pub trait CorticalIDIdentifier: Copy + Eq + core::hash::Hash + core::fmt::Debug {}

/// Whether `bytes` name a known area kind in their first byte.
///
/// Custom, memory, core, brain-input, and brain-output prefixes pass. Const so
/// [`CorticalID::from_id_bytes_const`] can check at compile time
const fn is_valid_cortical_id_bytes(bytes: &[u8; 8]) -> bool {
    match_bytes_by_cortical_type!(bytes,
        custom => { true },
        memory => { true },
        core => { true },
        brain_input => {
            // TODO more checks
            true
        },
        brain_output => {
            // TODO more checks
            true
        },
        invalid => { false },
    )
}

/// Accept a cortical ID whose first byte names a known area kind.
///
/// Any other prefix returns [`GenomeIdentifierError::CannotParseFromBytes`].
fn try_cortical_id_from_bytes(bytes: [u8; 8]) -> Result<CorticalID, GenomeIdentifierError> {
    if is_valid_cortical_id_bytes(&bytes) {
        Ok(CorticalID(bytes))
    } else {
        Err(GenomeIdentifierError::CannotParseFromBytes("CorticalID"))
    }
}

feagi_id! {
    /// A unique identifier for a cortical area in a genome
    pub CorticalID: 8,
    try_from = (GenomeIdentifierError, try_cortical_id_from_bytes),
    base64 = Base64EncodedCorticalID,
}

impl CorticalID {
    /// Wraps `bytes` as a cortical ID, running the same check as `try_from_id_bytes`.
    ///
    /// # Panics
    ///
    /// If `bytes` are not a valid cortical ID. Call it in a const block (`const { ... }`) to
    /// turn that into a compile error
    pub const fn from_id_bytes_const(bytes: [u8; 8]) -> Self {
        assert!(is_valid_cortical_id_bytes(&bytes), "bytes are not a valid CorticalID");
        Self(bytes)
    }
}


feagi_id! {
    /// A faux identifier unique to a processing context for a cortical area
    pub FauxCorticalID: 8,
    base64 = Base64EncodedFauxCorticalID,
}

impl FauxIdentifier<8> for FauxCorticalID {
    type RealID = CorticalID;
}

impl FauxIdentifier<12> for Base64EncodedFauxCorticalID {
    type RealID = Base64EncodedCorticalID;
}

impl HasFauxIdentifier<8> for CorticalID { type FauxID = FauxCorticalID; }

impl HasFauxIdentifier<12> for Base64EncodedCorticalID { type FauxID = Base64EncodedFauxCorticalID; }



impl CorticalIDIdentifier for CorticalID {}

impl CorticalIDIdentifier for FauxCorticalID {}


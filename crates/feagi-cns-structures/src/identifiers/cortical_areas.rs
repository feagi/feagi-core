use crate::identifiers::genome_identifier_error::GenomeIdentifierError;
use feagi_basis::impl_byte_array_id;

//region Cortical ID

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

/// Accept a cortical ID whose first byte names a known area kind.
///
/// Custom, memory, core, brain-input, and brain-output prefixes pass through.
/// Any other prefix returns [`GenomeIdentifierError::CannotParseFromBytes`].
fn try_cortical_id_from_bytes(bytes: [u8; 8]) -> Result<CorticalID, GenomeIdentifierError> {
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
            Err(GenomeIdentifierError::CannotParseFromBytes("CorticalID"))
        },
    )
}

impl_byte_array_id!(
    /// A unique identifier for a cortical area in a genome
    pub CorticalID,
    8,
    GenomeIdentifierError,
    try_cortical_id_from_bytes
);

//endregion

impl_byte_array_id!(
    /// Used as placeholders in amalgamations
    pub FauxCorticalID,
    8
);

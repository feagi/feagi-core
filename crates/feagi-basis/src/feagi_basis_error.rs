use crate::generic_collections::feagi_index_organizer_error::IndexOrganizerError;
use feagi_basis_collections::FeagiBasisCollectionsError;
use feagi_basis_quantization::FeagiBasisQuantizationError;
use feagi_basis_threading::FeagiBasisThreadingError;
use thiserror::Error;

/// The root error for the Basis crate.
#[derive(Error, Debug)]
pub enum FeagiBasisError {
    /// A generic error type. Anything using this should be updated with more specific errors.
    #[error("A generic error type. Anything using this should be updated with more specific errors")]
    DataEtc,
    #[error(transparent)]
    IndexOrganizerError(#[from] IndexOrganizerError),
    #[error(transparent)]
    Quantization(#[from] FeagiBasisQuantizationError),
    #[error(transparent)]
    Collections(#[from] FeagiBasisCollectionsError),
    #[error(transparent)]
    ThreadMessaging(#[from] FeagiBasisThreadingError),
}

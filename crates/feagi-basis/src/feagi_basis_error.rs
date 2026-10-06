use crate::collections::generic_collections::feagi_index_organizer_error::GenericCollectionError;
use crate::collections::CollectionsError;
use crate::quantization::QuantizationError;
use crate::threading::ThreadingError;
use thiserror::Error;

/// The root error for the Basis crate.
#[derive(Error, Debug)]
pub enum FeagiBasisError {
    /// A generic error type. Anything using this should be updated with more specific errors.
    #[error("A generic error type. Anything using this should be updated with more specific errors")]
    DataEtc,
    #[error(transparent)]
    IndexOrganizerError(#[from] GenericCollectionError),
    #[error(transparent)]
    Quantization(#[from] QuantizationError),
    #[error(transparent)]
    Collections(#[from] CollectionsError),
    #[error(transparent)]
    ThreadMessaging(#[from] ThreadingError),
}

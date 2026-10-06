use thiserror::Error;
use crate::collections::bit_packed_bool::BitBatchParDataError;
use crate::collections::par_data::ParDataError;
use crate::collections::spatial_helpers::SpatialHelperError;
use crate::collections::generic_collections::feagi_index_organizer_error::GenericCollectionError;

#[derive(Error, Debug)]
pub enum CollectionsError {
    #[error(transparent)]
    ParDataError(#[from] ParDataError),
    #[error(transparent)]
    BitBatchParDataError(#[from] BitBatchParDataError),
    #[error(transparent)]
    SpatialIndexingError(#[from] SpatialHelperError),
    #[error(transparent)]
    GenericCollectionError(#[from] GenericCollectionError)
}

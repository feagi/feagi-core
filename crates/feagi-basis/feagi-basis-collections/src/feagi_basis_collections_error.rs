use crate::bit_packed_bool::BitBatchParDataError;
use crate::generic_data::ParDataError;
use crate::spatial_indexing_structs::SpatialIndexingError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeagiBasisCollectionsError {
    #[error(transparent)]
    ParDataError(#[from] ParDataError),
    #[error(transparent)]
    BitBatchParDataError(#[from] BitBatchParDataError),
    #[error(transparent)]
    SpatialIndexingError(#[from] SpatialIndexingError),
}

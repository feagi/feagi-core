use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexOrganizerError {
    #[error("The index manager was given an invalid range")]
    IndexManagerError,
    #[error("Reached maximum index")]
    IndexManagerLimit,
    #[error("Index not found")]
    IndexManagerIndex { index: usize },
    #[error("Failed to merge an index range")]
    RangeVectorFailedMerge,
}

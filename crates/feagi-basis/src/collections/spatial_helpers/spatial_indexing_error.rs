use thiserror::Error;
use crate::collections::spatial_context::spatial_context_error::SpatialContextError;

#[derive(Error, Debug)]
pub enum SpatialHelperError {
    #[error("Axis order is invalid for the given dimension count")]
    InvalidAxisOrder,
    #[error(transparent)]
    InvalidContext(#[from] SpatialContextError)
}

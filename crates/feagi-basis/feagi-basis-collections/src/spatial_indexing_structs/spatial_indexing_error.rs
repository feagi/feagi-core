use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpatialIndexingError {
    #[error("Dimensions was set to 0 in some axis")]
    InvalidDimensions,
    #[error("Axis order is invalid for the given dimension count")]
    InvalidAxisOrder,
}

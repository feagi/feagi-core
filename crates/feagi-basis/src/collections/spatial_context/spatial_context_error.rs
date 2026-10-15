use crate::quantization::quantizable::DataValueQuantizationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpatialContextError {
    #[error("Attempted to index using a coordinate, but it was not in the given dimensions")]
    InvalidSpatialIndex,
    #[error("Attempted to create a dimensions value with a zero sized axis")]
    DimensionsCannotBeZero,
    #[error("Attempted to convert spatial data to another quantization but a value would not fit in the target quantization")]
    SpatialQuantizationOutOfRange,
    #[error(transparent)]
    InvalidSpatialQuantization(#[from] DataValueQuantizationError),
}

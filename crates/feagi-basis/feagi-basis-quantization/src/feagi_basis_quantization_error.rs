use thiserror::Error;
use crate::values::quantizable::DataValueQuantizationError;
use crate::values::spatial::data_values_spatial_error::DataValuesSpatialError;

#[derive(Error, Debug)]
pub enum FeagiBasisQuantizationError {
    #[error(transparent)]
    DataValueQuantizationError(#[from] DataValueQuantizationError),
    #[error(transparent)]
    DataValuesSpatialError(#[from] DataValuesSpatialError),
}
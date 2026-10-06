use thiserror::Error;
use crate::quantizable::DataValueQuantizationError;

#[derive(Error, Debug)]
pub enum FeagiBasisQuantizationError {
    #[error(transparent)]
    DataValueQuantizationError(#[from] DataValueQuantizationError),
}
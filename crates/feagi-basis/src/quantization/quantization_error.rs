use crate::quantization::quantizable::DataValueQuantizationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantizationError {
    #[error(transparent)]
    DataValueQuantizationError(#[from] DataValueQuantizationError),
}
use thiserror::Error;
use crate::quantization::quantizable::DataValueQuantizationError;

#[derive(Error, Debug)]
pub enum QuantizationError {
    #[error(transparent)]
    DataValueQuantizationError(#[from] DataValueQuantizationError),
}
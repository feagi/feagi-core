use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataValueQuantizationError {
    #[error("Given data is not in acceptable range")]
    QuantizationOutOfRange,
    #[error("Invalid input quantization")]
    InvalidQuantization,
    #[error("Quantization type not supported by target hardware backend")]
    HardwareNoLikeQuant,
    #[error("Attempted to store a percentage value that was not in range")]
    PercentageOutOfRange,
}
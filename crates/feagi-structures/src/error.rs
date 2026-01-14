use std::error::Error;
use std::fmt::{Display, Formatter};

/// Common error type for FEAGI data operations.
///
/// Provides structured error handling for serialization, deserialization,
/// validation, and internal errors across the FEAGI data processing pipeline.
///
/// # Examples
/// ```
/// use feagi_structures::FeagiDataError;
///
/// fn validate_count(count: u32) -> Result<(), FeagiDataError> {
///     if count == 0 {
///         return Err(FeagiDataError::BadParameters("Count must be > 0".into()));
///     }
///     Ok(())
/// }
///
/// assert!(validate_count(0).is_err());
/// assert!(validate_count(5).is_ok());
/// ```
#[derive(Debug)]
pub enum FeagiDataError {
    /// Failed to deserialize bytes into data structures
    DeserializationError(String),
    /// Failed to serialize data structures into bytes
    SerializationError(String),
    /// Invalid parameters provided to a function
    BadParameters(String),
    /// Error related to neuron operations
    NeuronError(String),
    /// Internal error indicating a bug (please report)
    InternalError(String),
    /// resource is locked while system is running
    ResourceLockedWhileRunning(String),
    /// failed to process something in a const function
    ConstError(&'static str),
    /// Feature not yet implemented
    NotImplemented,
}

impl Display for FeagiDataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiDataError::DeserializationError(msg) => {
                write!(f, "Failed to Deserialize Bytes: {}", msg)
            }
            FeagiDataError::SerializationError(msg) => {
                write!(f, "Failed to Serialize Bytes: {}", msg)
            }
            FeagiDataError::BadParameters(msg) => write!(f, "Bad Parameters: {}", msg),
            FeagiDataError::NeuronError(msg) => write!(f, "NeuronError: {}", msg),
            FeagiDataError::InternalError(msg) => write!(
                f,
                "Internal Error, please raise an issue on Github: {}",
                msg
            ),
            FeagiDataError::ResourceLockedWhileRunning(msg) => write!(f, "Resource Locked While Running: {}", msg),
            FeagiDataError::ConstError(msg) => write!(f, "ConstError: {}", msg),
            FeagiDataError::NotImplemented => write!(
                f,
                "This function is not yet implemented! Please raise an issue on Github!"
            ),
        }
    }
}
impl Error for FeagiDataError {}

//  TODO From<> from other error types

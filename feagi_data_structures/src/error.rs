use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum FeagiDataError {
    DeserializationError(String),
    SerializationError(String),
    BadParameters(String),
    InternalError(String),
    NotImplemented,
}

impl Display for FeagiDataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self { 
            FeagiDataError::DeserializationError(msg) => write!(f, "Failed to Deserialize Bytes: {}", msg),
            FeagiDataError::SerializationError(msg) => write!(f, "Failed to Serialize Bytes: {}", msg),
            FeagiDataError::BadParameters(msg) => write!(f, "Bad Parameters: {}", msg),
            FeagiDataError::InternalError(msg) => write!(f, "Internal Error, please raise an issue on Github: {}", msg),
            FeagiDataError::NotImplemented => write!(f, "This function is not yet implemented! Please raise an issue on Github!")
        }
    }
}
impl Error for FeagiDataError {}

//  TODO From<> from other error types

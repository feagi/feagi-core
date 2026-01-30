//! Error types for the FEAGI agent.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur in FEAGI agent operations.
#[derive(Debug, Clone)]
pub enum FeagiAgentError {
    /// Failed to connect
    ConnectionFailed(String),
    /// Authentication failed (invalid credentials, expired token, etc.)
    AuthenticationFailed(String),
    /// General failure (deserialization, parsing, validation, etc.)
    GeneralFailure(String),
    /// Server failed to send data
    ServerFailedToSendData(String),
    /// Server failed to send data
    ServerFailedToGetData(String),
}

impl Display for FeagiAgentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiAgentError::ConnectionFailed(msg) => {
                write!(f, "FeagiAgentError: Connection failed: {}", msg)
            }
            FeagiAgentError::AuthenticationFailed(msg) => {
                write!(f, "FeagiAgentError: Authentication failed: {}", msg)
            }
            FeagiAgentError::GeneralFailure(msg) => {
                write!(f, "FeagiAgentError: {}", msg)
            }
            FeagiAgentError::ServerFailedToSendData(msg) => {
                write!(f, "FeagiAgentError: Server failed to send data: {}", msg)
            }
            FeagiAgentError::ServerFailedToGetData(msg) => {
                write!(f, "FeagiAgentError: Server failed to get data: {}", msg)
            }
        }
    }
}

impl Error for FeagiAgentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

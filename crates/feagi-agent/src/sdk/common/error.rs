//! Error types for the FEAGI agent.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur in FEAGI agent operations.
#[derive(Debug, Clone)]
pub enum FeagiAgentError {
    /// Authentication failed (invalid credentials, expired token, etc.)
    AuthenticationFailed(String),
    /// General failure (deserialization, parsing, validation, etc.)
    GeneralFailure(String),
}

impl Display for FeagiAgentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiAgentError::AuthenticationFailed(msg) => {
                write!(f, "FeagiAgentError: Authentication failed: {}", msg)
            }
            FeagiAgentError::GeneralFailure(msg) => {
                write!(f, "FeagiAgentError: {}", msg)
            }
        }
    }
}

impl Error for FeagiAgentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

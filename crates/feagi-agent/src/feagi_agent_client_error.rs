//! Error types for the FEAGI agent.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur in FEAGI agent operations.
#[derive(Debug, Clone)]
pub enum FeagiAgentClientError {
    /// Failed to connect
    ConnectionFailed(String),
    /// Authentication failed (invalid credentials, expired token, etc.)
    AuthenticationFailed(String),
    /// We cannot understand what the server sent
    UnableToDecodeReceivedData(String),
    /// Client failed to send data to the server
    UnableToSendData(String),
    /// Other SDK/controller error
    Other(String),
}

impl Display for FeagiAgentClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiAgentClientError::ConnectionFailed(msg) => {
                write!(f, "FeagiAgentClientError: Connection failed: {}", msg)
            }
            FeagiAgentClientError::AuthenticationFailed(msg) => {
                write!(f, "FeagiAgentClientError: Authentication failed: {}", msg)
            }
            FeagiAgentClientError::UnableToDecodeReceivedData(msg) => {
                write!(f, "FeagiAgentClientError: Unable to decode data from server: {}", msg)
            }
            FeagiAgentClientError::UnableToSendData(msg) => {
                write!(f, "FeagiAgentClientError: Unable to send data to server: {}", msg)
            }
            FeagiAgentClientError::Other(msg) => {
                write!(f, "FeagiAgentClientError: {}", msg)
            }
        }
    }
}

impl Error for FeagiAgentClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

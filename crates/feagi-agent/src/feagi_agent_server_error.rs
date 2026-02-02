use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur in FEAGI agent server operations.
#[derive(Debug, Clone)]
pub enum FeagiAgentServerError {
    /// Failed to connect
    ConnectionFailed(String),
    /// Authentication failed (invalid credentials, expired token, etc.)
    AuthenticationFailed(String),
    /// We cannot understand what the client sent
    UnableToDecodeReceivedData(String),
    /// Server failed to send data to the client
    UnableToSendData(String),
}

impl Display for FeagiAgentServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiAgentServerError::ConnectionFailed(msg) => {
                write!(f, "FeagiAgentServerError: Connection failed: {}", msg)
            }
            FeagiAgentServerError::AuthenticationFailed(msg) => {
                write!(f, "FeagiAgentServerError: Authentication failed: {}", msg)
            }
            FeagiAgentServerError::UnableToDecodeReceivedData(msg) => {
                write!(f, "FeagiAgentServerError: Unable to interpret received data: {}", msg)
            }
            FeagiAgentServerError::UnableToSendData(msg) => {
                write!(f, "FeagiAgentServerError: Unable to send data: {}", msg)
            }
        }
    }
}

impl Error for FeagiAgentServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

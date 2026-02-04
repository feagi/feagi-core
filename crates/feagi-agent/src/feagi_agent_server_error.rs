use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur in FEAGI agent server operations.
#[derive(Debug, Clone)]
pub enum FeagiAgentServerError {
    /// Unable to start
    InitFail(String),
    /// Failed to connect
    ConnectionFailed(String),
    /// Authentication failed (invalid credentials, expired token, etc.)
    AuthenticationFailed(String),
    /// We cannot understand what the client sent
    UnableToDecodeReceivedData(String),
    /// Server failed to send data to the client
    UnableToSendData(String),
    /// Failed to persist connectome or genome data
    PersistenceFailed(String),
}

impl Display for FeagiAgentServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiAgentServerError::InitFail(msg) => {
                write!(f, "FeagiAgentServerError: Init failed: {}", msg)
            }
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
            FeagiAgentServerError::PersistenceFailed(msg) => {
                write!(f, "FeagiAgentServerError: Unable to persist data: {}", msg)
            }
        }
    }
}

impl Error for FeagiAgentServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

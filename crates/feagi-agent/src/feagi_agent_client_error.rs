//! Error types for the FEAGI agent.

use feagi_io::FeagiNetworkError;
use feagi_structures::FeagiDataError;
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

impl From<FeagiDataError> for FeagiAgentClientError {
    fn from(err: FeagiDataError) -> Self {
        match err {
            FeagiDataError::DeserializationError(msg) => {
                FeagiAgentClientError::UnableToDecodeReceivedData(msg)
            }
            FeagiDataError::SerializationError(msg) => {
                FeagiAgentClientError::UnableToSendData(msg)
            }
            FeagiDataError::BadParameters(msg) => {
                FeagiAgentClientError::Other(format!("Bad parameters: {}", msg))
            }
            FeagiDataError::NeuronError(msg) => {
                FeagiAgentClientError::Other(format!("Neuron error: {}", msg))
            }
            FeagiDataError::InternalError(msg) => {
                FeagiAgentClientError::Other(format!("Internal error: {}", msg))
            }
            FeagiDataError::ResourceLockedWhileRunning(msg) => {
                FeagiAgentClientError::Other(format!("Resource locked: {}", msg))
            }
            FeagiDataError::ConstError(msg) => {
                FeagiAgentClientError::Other(format!("Const error: {}", msg))
            }
            FeagiDataError::NotImplemented => {
                FeagiAgentClientError::Other("Not implemented".to_string())
            }
        }
    }
}

impl From<FeagiNetworkError> for FeagiAgentClientError {
    fn from(err: FeagiNetworkError) -> Self {
        match err {
            FeagiNetworkError::CannotBind(msg) => {
                FeagiAgentClientError::ConnectionFailed(format!("Cannot bind: {}", msg))
            }
            FeagiNetworkError::CannotUnbind(msg) => {
                FeagiAgentClientError::ConnectionFailed(format!("Cannot unbind: {}", msg))
            }
            FeagiNetworkError::CannotConnect(msg) => {
                FeagiAgentClientError::ConnectionFailed(format!("Cannot connect: {}", msg))
            }
            FeagiNetworkError::CannotDisconnect(msg) => {
                FeagiAgentClientError::ConnectionFailed(format!("Cannot disconnect: {}", msg))
            }
            FeagiNetworkError::SendFailed(msg) => {
                FeagiAgentClientError::UnableToSendData(msg)
            }
            FeagiNetworkError::ReceiveFailed(msg) => {
                FeagiAgentClientError::UnableToDecodeReceivedData(format!("Receive failed: {}", msg))
            }
            FeagiNetworkError::InvalidSocketProperties(msg) => {
                FeagiAgentClientError::ConnectionFailed(format!("Invalid socket properties: {}", msg))
            }
            FeagiNetworkError::SocketCreationFailed(msg) => {
                FeagiAgentClientError::ConnectionFailed(format!("Socket creation failed: {}", msg))
            }
            FeagiNetworkError::GeneralFailure(msg) => {
                FeagiAgentClientError::Other(format!("General failure: {}", msg))
            }
        }
    }
}

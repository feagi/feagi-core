use feagi_io::FeagiNetworkError;
use feagi_structures::FeagiDataError;
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

impl From<FeagiDataError> for FeagiAgentServerError {
    fn from(err: FeagiDataError) -> Self {
        match err {
            FeagiDataError::DeserializationError(msg) => {
                FeagiAgentServerError::UnableToDecodeReceivedData(msg)
            }
            FeagiDataError::SerializationError(msg) => {
                FeagiAgentServerError::UnableToSendData(msg)
            }
            FeagiDataError::BadParameters(msg) => {
                FeagiAgentServerError::UnableToDecodeReceivedData(format!("Bad parameters: {}", msg))
            }
            FeagiDataError::NeuronError(msg) => {
                FeagiAgentServerError::UnableToDecodeReceivedData(format!("Neuron error: {}", msg))
            }
            FeagiDataError::InternalError(msg) => {
                FeagiAgentServerError::UnableToDecodeReceivedData(format!("Internal error: {}", msg))
            }
            FeagiDataError::ResourceLockedWhileRunning(msg) => {
                FeagiAgentServerError::UnableToDecodeReceivedData(format!("Resource locked: {}", msg))
            }
            FeagiDataError::ConstError(msg) => {
                FeagiAgentServerError::UnableToDecodeReceivedData(format!("Const error: {}", msg))
            }
            FeagiDataError::NotImplemented => {
                FeagiAgentServerError::UnableToDecodeReceivedData("Not implemented".to_string())
            }
        }
    }
}

impl From<FeagiNetworkError> for FeagiAgentServerError {
    fn from(err: FeagiNetworkError) -> Self {
        match err {
            FeagiNetworkError::CannotBind(msg) => {
                FeagiAgentServerError::InitFail(format!("Cannot bind: {}", msg))
            }
            FeagiNetworkError::CannotUnbind(msg) => {
                FeagiAgentServerError::ConnectionFailed(format!("Cannot unbind: {}", msg))
            }
            FeagiNetworkError::CannotConnect(msg) => {
                FeagiAgentServerError::ConnectionFailed(format!("Cannot connect: {}", msg))
            }
            FeagiNetworkError::CannotDisconnect(msg) => {
                FeagiAgentServerError::ConnectionFailed(format!("Cannot disconnect: {}", msg))
            }
            FeagiNetworkError::SendFailed(msg) => {
                FeagiAgentServerError::UnableToSendData(msg)
            }
            FeagiNetworkError::ReceiveFailed(msg) => {
                FeagiAgentServerError::UnableToDecodeReceivedData(format!("Receive failed: {}", msg))
            }
            FeagiNetworkError::InvalidSocketProperties(msg) => {
                FeagiAgentServerError::InitFail(format!("Invalid socket properties: {}", msg))
            }
            FeagiNetworkError::SocketCreationFailed(msg) => {
                FeagiAgentServerError::InitFail(format!("Socket creation failed: {}", msg))
            }
            FeagiNetworkError::GeneralFailure(msg) => {
                FeagiAgentServerError::ConnectionFailed(format!("General failure: {}", msg))
            }
        }
    }
}


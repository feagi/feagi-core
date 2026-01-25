use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum FeagiNetworkError {
    /// Failed to bind server socket to address
    CannotBind(String),
    /// Failed to unbind server socket from address
    CannotUnbind(String),
    /// Failed to connect client socket to server
    CannotConnect(String),
    /// Failed to disconnect client socket from server
    CannotDisconnect(String),
    /// Failed to send data
    SendFailed(String),
    /// Failed to receive data
    ReceiveFailed(String),
    /// Invalid URL format
    InvalidUrl(String),
    /// Socket creation failed
    SocketCreationFailed(String),
    /// General failure (e.g., configuration error, invalid state)
    GeneralFailure(String),
}

impl Display for FeagiNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiNetworkError::CannotBind(msg) => {
                write!(f, "FeagiNetworkError: Unable to bind: {}", msg)
            }
            FeagiNetworkError::CannotUnbind(msg) => {
                write!(f, "FeagiNetworkError: Unable to unbind: {}", msg)
            }
            FeagiNetworkError::CannotConnect(msg) => {
                write!(f, "FeagiNetworkError: Unable to connect: {}", msg)
            }
            FeagiNetworkError::CannotDisconnect(msg) => {
                write!(f, "FeagiNetworkError: Unable to disconnect: {}", msg)
            }
            FeagiNetworkError::SendFailed(msg) => {
                write!(f, "FeagiNetworkError: Send failed: {}", msg)
            }
            FeagiNetworkError::ReceiveFailed(msg) => {
                write!(f, "FeagiNetworkError: Receive failed: {}", msg)
            }
            FeagiNetworkError::InvalidUrl(msg) => {
                write!(f, "FeagiNetworkError: Invalid URL: {}", msg)
            }
            FeagiNetworkError::SocketCreationFailed(msg) => {
                write!(f, "FeagiNetworkError: Socket creation failed: {}", msg)
            }
            FeagiNetworkError::GeneralFailure(msg) => {
                write!(f, "FeagiNetworkError: {}", msg)
            }
        }
    }
}

impl Error for FeagiNetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

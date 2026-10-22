//! Unified error types for the FEAGI agent (client and server).

use feagi_io::FeagiNetworkError;
use feagi_structures::FeagiDataError;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur in FEAGI agent operations (both client and server).
#[derive(Debug, Clone)]
pub enum FeagiAgentError {
    /// Unable to initialize/start (typically server-side)
    InitFail(String),
    /// Failed to connect
    ConnectionFailed(String),
    /// Authentication failed (invalid credentials, expired token, etc.)
    AuthenticationFailed(String),
    /// Cannot understand what the remote endpoint sent
    UnableToDecodeReceivedData(String),
    /// Failed to send data to the remote endpoint
    UnableToSendData(String),
    /// Something went wrong with the server network socket and it should be restarted
    SocketFailure(String),
    /// Other/uncategorized error
    Other(String),
}

impl Display for FeagiAgentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiAgentError::InitFail(msg) => {
                write!(f, "FeagiAgentError: Init failed: {}", msg)
            }
            FeagiAgentError::ConnectionFailed(msg) => {
                write!(f, "FeagiAgentError: Connection failed: {}", msg)
            }
            FeagiAgentError::AuthenticationFailed(msg) => {
                write!(f, "FeagiAgentError: Authentication failed: {}", msg)
            }
            FeagiAgentError::UnableToDecodeReceivedData(msg) => {
                write!(
                    f,
                    "FeagiAgentError: Unable to decode received data: {}",
                    msg
                )
            }
            FeagiAgentError::UnableToSendData(msg) => {
                write!(f, "FeagiAgentError: Unable to send data: {}", msg)
            }
            FeagiAgentError::SocketFailure(msg) => {
                write!(f, "FeagiAgentError: Socket failure: {}", msg)
            }
            FeagiAgentError::Other(msg) => {
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

impl From<FeagiDataError> for FeagiAgentError {
    fn from(err: FeagiDataError) -> Self {
        match err {
            FeagiDataError::DeserializationError(msg) => {
                FeagiAgentError::UnableToDecodeReceivedData(msg)
            }
            FeagiDataError::SerializationError(msg) => FeagiAgentError::UnableToSendData(msg),
            FeagiDataError::BadParameters(msg) => {
                FeagiAgentError::Other(format!("Bad parameters: {}", msg))
            }
            FeagiDataError::NeuronError(msg) => {
                FeagiAgentError::Other(format!("Neuron error: {}", msg))
            }
            FeagiDataError::InternalError(msg) => {
                FeagiAgentError::Other(format!("Internal error: {}", msg))
            }
            FeagiDataError::ResourceLockedWhileRunning(msg) => {
                FeagiAgentError::Other(format!("Resource locked: {}", msg))
            }
            FeagiDataError::ConstError(msg) => {
                FeagiAgentError::Other(format!("Const error: {}", msg))
            }
            FeagiDataError::NotImplemented => FeagiAgentError::Other("Not implemented".to_string()),
        }
    }
}

/// Returns true when `err` indicates a non-blocking transport send would block (e.g. ZMQ `EAGAIN`
/// surfaced as `FeagiNetworkError::SendFailed` with `"Socket would block"`).
///
/// Used by callers and telemetry to avoid treating transient backpressure like a session failure.
pub fn is_transient_zmq_send_would_block(err: &FeagiAgentError) -> bool {
    match err {
        FeagiAgentError::UnableToSendData(msg) | FeagiAgentError::SocketFailure(msg) => {
            msg.contains("Socket would block")
        }
        _ => err.to_string().contains("Socket would block"),
    }
}

/// String-based check for error messages already formatted (e.g. `anyhow` chains).
pub fn is_transient_zmq_send_message(message: &str) -> bool {
    message.contains("Socket would block")
}

impl From<FeagiNetworkError> for FeagiAgentError {
    fn from(err: FeagiNetworkError) -> Self {
        match err {
            FeagiNetworkError::CannotBind(msg) => {
                FeagiAgentError::InitFail(format!("Cannot bind: {}", msg))
            }
            FeagiNetworkError::CannotUnbind(msg) => {
                FeagiAgentError::SocketFailure(format!("Cannot unbind: {}", msg))
            }
            FeagiNetworkError::CannotConnect(msg) => {
                FeagiAgentError::ConnectionFailed(format!("Cannot connect: {}", msg))
            }
            FeagiNetworkError::CannotDisconnect(msg) => {
                FeagiAgentError::SocketFailure(format!("Cannot disconnect: {}", msg))
            }
            FeagiNetworkError::SendFailed(msg) => FeagiAgentError::UnableToSendData(msg),
            FeagiNetworkError::ReceiveFailed(msg) => {
                FeagiAgentError::UnableToDecodeReceivedData(format!("Receive failed: {}", msg))
            }
            FeagiNetworkError::InvalidSocketProperties(msg) => {
                FeagiAgentError::InitFail(format!("Invalid socket properties: {}", msg))
            }
            FeagiNetworkError::SocketCreationFailed(msg) => {
                FeagiAgentError::SocketFailure(format!("Socket creation failed: {}", msg))
            }
            FeagiNetworkError::GeneralFailure(msg) => {
                FeagiAgentError::Other(format!("General failure: {}", msg))
            }
        }
    }
}

#[cfg(test)]
mod transient_send_tests {
    use super::*;

    #[test]
    fn detects_would_block_in_unable_to_send() {
        let e = FeagiAgentError::UnableToSendData("Socket would block".to_string());
        assert!(is_transient_zmq_send_would_block(&e));
    }

    #[test]
    fn ignores_other_unable_to_send() {
        let e =
            FeagiAgentError::UnableToSendData("Cannot send to inactive sensory socket".to_string());
        assert!(!is_transient_zmq_send_would_block(&e));
    }

    #[test]
    fn message_helper_matches() {
        assert!(is_transient_zmq_send_message(
            "FeagiAgentError: Unable to send data: Socket would block"
        ));
        assert!(!is_transient_zmq_send_message("connection reset"));
    }
}

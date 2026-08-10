//! Unified error types for the FEAGI agent (client and server).

use feagi_io::FeagiNetworkError;
use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

macro_rules! define_feagi_agent_error_key {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(FeagiErrorKey)]
        pub struct $name {
            context: &'static str,
            pub message: String,
        }
    };
}

define_feagi_agent_error_key!(
    /// Unable to initialize/start (typically server-side)
    FeagiAgentInitFailErrKey
);
define_feagi_agent_error_key!(
    /// Failed to connect
    FeagiAgentConnectionFailedErrKey
);
define_feagi_agent_error_key!(
    /// Authentication failed (invalid credentials, expired token, etc.)
    FeagiAgentAuthenticationFailedErrKey
);
define_feagi_agent_error_key!(
    /// Cannot understand what the remote endpoint sent
    FeagiAgentUnableToDecodeReceivedDataErrKey
);
define_feagi_agent_error_key!(
    /// Failed to send data to the remote endpoint
    FeagiAgentUnableToSendDataErrKey
);
define_feagi_agent_error_key!(
    /// Something went wrong with the server network socket and it should be restarted
    FeagiAgentSocketFailureErrKey
);
define_feagi_agent_error_key!(
    /// Other/uncategorized error
    FeagiAgentOtherErrKey
);

generate_feagi_error! {
    /// Errors that can occur in FEAGI agent operations (both client and server).
    FeagiAgentError,
    keys: {
        InitFail: FeagiAgentInitFailErrKey,
        ConnectionFailed: FeagiAgentConnectionFailedErrKey,
        AuthenticationFailed: FeagiAgentAuthenticationFailedErrKey,
        UnableToDecodeReceivedData: FeagiAgentUnableToDecodeReceivedDataErrKey,
        UnableToSendData: FeagiAgentUnableToSendDataErrKey,
        SocketFailure: FeagiAgentSocketFailureErrKey,
        Other: FeagiAgentOtherErrKey,
    },
    sub_errors: {

    },
}

impl FeagiAgentError {
    pub fn init_fail(message: impl Into<String>) -> Self {
        FeagiAgentInitFailErrKey::new("FeagiAgentError: Init failed", message.into()).into()
    }

    pub fn connection_failed(message: impl Into<String>) -> Self {
        FeagiAgentConnectionFailedErrKey::new("FeagiAgentError: Connection failed", message.into()).into()
    }

    pub fn authentication_failed(message: impl Into<String>) -> Self {
        FeagiAgentAuthenticationFailedErrKey::new("FeagiAgentError: Authentication failed", message.into()).into()
    }

    pub fn unable_to_decode_received_data(message: impl Into<String>) -> Self {
        FeagiAgentUnableToDecodeReceivedDataErrKey::new("FeagiAgentError: Unable to decode received data", message.into()).into()
    }

    pub fn unable_to_send_data(message: impl Into<String>) -> Self {
        FeagiAgentUnableToSendDataErrKey::new("FeagiAgentError: Unable to send data", message.into()).into()
    }

    pub fn socket_failure(message: impl Into<String>) -> Self {
        FeagiAgentSocketFailureErrKey::new("FeagiAgentError: Socket failure", message.into()).into()
    }

    pub fn other(message: impl Into<String>) -> Self {
        FeagiAgentOtherErrKey::new("FeagiAgentError", message.into()).into()
    }

    pub fn message(&self) -> &str {
        match self {
            FeagiAgentError::InitFail(key) => &key.message,
            FeagiAgentError::ConnectionFailed(key) => &key.message,
            FeagiAgentError::AuthenticationFailed(key) => &key.message,
            FeagiAgentError::UnableToDecodeReceivedData(key) => &key.message,
            FeagiAgentError::UnableToSendData(key) => &key.message,
            FeagiAgentError::SocketFailure(key) => &key.message,
            FeagiAgentError::Other(key) => &key.message,
        }
    }
}

/// Returns true when `err` indicates a non-blocking transport send would block (e.g. ZMQ `EAGAIN`
/// surfaced as `FeagiNetworkError::SendFailed` with `"Socket would block"`).
///
/// Used by callers and telemetry to avoid treating transient backpressure like a session failure.
pub fn is_transient_zmq_send_would_block(err: &FeagiAgentError) -> bool {
    match err {
        FeagiAgentError::UnableToSendData(key) => key.message.contains("Socket would block"),
        FeagiAgentError::SocketFailure(key) => key.message.contains("Socket would block"),
        _ => err.message().contains("Socket would block"),
    }
}

/// String-based check for error messages already formatted (e.g. `anyhow` chains).
pub fn is_transient_zmq_send_message(message: &str) -> bool {
    message.contains("Socket would block")
}

impl From<FeagiNetworkError> for FeagiAgentError {
    fn from(err: FeagiNetworkError) -> Self {
        match err {
            FeagiNetworkError::CannotBind(msg) => FeagiAgentError::init_fail(format!("Cannot bind: {}", msg)),
            FeagiNetworkError::CannotUnbind(msg) => FeagiAgentError::socket_failure(format!("Cannot unbind: {}", msg)),
            FeagiNetworkError::CannotConnect(msg) => FeagiAgentError::connection_failed(format!("Cannot connect: {}", msg)),
            FeagiNetworkError::CannotDisconnect(msg) => FeagiAgentError::socket_failure(format!("Cannot disconnect: {}", msg)),
            FeagiNetworkError::SendFailed(msg) => FeagiAgentError::unable_to_send_data(msg),
            FeagiNetworkError::ReceiveFailed(msg) => FeagiAgentError::unable_to_decode_received_data(format!("Receive failed: {}", msg)),
            FeagiNetworkError::InvalidSocketProperties(msg) => FeagiAgentError::init_fail(format!("Invalid socket properties: {}", msg)),
            FeagiNetworkError::SocketCreationFailed(msg) => FeagiAgentError::socket_failure(format!("Socket creation failed: {}", msg)),
            FeagiNetworkError::GeneralFailure(msg) => FeagiAgentError::other(format!("General failure: {}", msg)),
        }
    }
}

impl From<()> for FeagiAgentError {
    fn from(_: ()) -> Self {
        FeagiAgentError::other("operation failed")
    }
}

#[cfg(test)]
mod transient_send_tests {
    use super::*;

    #[test]
    fn detects_would_block_in_unable_to_send() {
        let e = FeagiAgentError::unable_to_send_data("Socket would block");
        assert!(is_transient_zmq_send_would_block(&e));
    }

    #[test]
    fn ignores_other_unable_to_send() {
        let e = FeagiAgentError::unable_to_send_data("Cannot send to inactive sensory socket");
        assert!(!is_transient_zmq_send_would_block(&e));
    }

    #[test]
    fn message_helper_matches() {
        assert!(is_transient_zmq_send_message("FeagiAgentError: Unable to send data: Socket would block"));
        assert!(!is_transient_zmq_send_message("connection reset"));
    }
}

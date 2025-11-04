//! Common error types for all transports

use std::fmt;

/// Result type alias for transport operations
pub type TransportResult<T> = Result<T, TransportError>;

/// Transport-agnostic error type
#[derive(Debug)]
pub enum TransportError {
    /// Failed to initialize transport
    InitializationFailed(String),
    
    /// Failed to bind server socket
    BindFailed(String),
    
    /// Failed to connect client socket
    ConnectFailed(String),
    
    /// Failed to send message
    SendFailed(String),
    
    /// Failed to receive message
    ReceiveFailed(String),
    
    /// Timeout occurred
    Timeout,
    
    /// No data available (non-blocking receive)
    NoData,
    
    /// Connection closed
    ConnectionClosed,
    
    /// Transport is not running
    NotRunning,
    
    /// Transport is already running
    AlreadyRunning,
    
    /// Invalid configuration
    InvalidConfig(String),
    
    /// Message too large
    MessageTooLarge { size: usize, max_size: usize },
    
    /// Invalid message format
    InvalidMessage(String),
    
    /// Transport-specific error
    #[cfg(feature = "zmq-server")]
    Zmq(zmq::Error),
    
    /// I/O error
    Io(std::io::Error),
    
    /// Serialization error
    Serialization(String),
    
    /// Other error
    Other(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            Self::BindFailed(msg) => write!(f, "Bind failed: {}", msg),
            Self::ConnectFailed(msg) => write!(f, "Connect failed: {}", msg),
            Self::SendFailed(msg) => write!(f, "Send failed: {}", msg),
            Self::ReceiveFailed(msg) => write!(f, "Receive failed: {}", msg),
            Self::Timeout => write!(f, "Operation timed out"),
            Self::NoData => write!(f, "No data available"),
            Self::ConnectionClosed => write!(f, "Connection closed"),
            Self::NotRunning => write!(f, "Transport is not running"),
            Self::AlreadyRunning => write!(f, "Transport is already running"),
            Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::MessageTooLarge { size, max_size } => {
                write!(f, "Message too large: {} bytes (max: {})", size, max_size)
            }
            Self::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            #[cfg(feature = "zmq-server")]
            Self::Zmq(e) => write!(f, "ZMQ error: {}", e),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "zmq-server")]
            Self::Zmq(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(feature = "zmq-server")]
impl From<zmq::Error> for TransportError {
    fn from(err: zmq::Error) -> Self {
        match err {
            zmq::Error::EAGAIN => Self::Timeout,
            _ => Self::Zmq(err),
        }
    }
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<String> for TransportError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

impl From<&str> for TransportError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}





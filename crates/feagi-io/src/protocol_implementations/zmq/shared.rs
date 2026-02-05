//! Shared utilities for ZMQ implementations.

use serde::{Deserialize, Serialize};
use crate::FeagiNetworkError;

/// URL endpoint struct for ZMQ endpoints with validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZmqUrl {
    url: String,
}

impl ZmqUrl {
    /// Creates a new ZmqUrl after validating the format.
    ///
    /// # Arguments
    ///
    /// * `url` - The ZMQ URL (e.g., "tcp://127.0.0.1:5555", "tcp://*:5555", "ipc:///tmp/feed").
    ///
    /// # Errors
    ///
    /// Returns an error if the URL format is invalid.
    pub fn new(url: &str) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(url)?;
        Ok(ZmqUrl { url: url.to_string() })
    }

    /// Returns the URL as a string slice.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.url
    }
}

impl std::fmt::Display for ZmqUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

/// Validates a ZMQ URL format.
///
/// Valid transports: tcp, ipc, inproc, pgm, epgm
fn validate_zmq_url(url: &str) -> Result<(), FeagiNetworkError> {
    // Check for valid ZMQ transport prefixes
    const VALID_PREFIXES: [&str; 5] = ["tcp://", "ipc://", "inproc://", "pgm://", "epgm://"];

    if !VALID_PREFIXES.iter().any(|prefix| url.starts_with(prefix)) {
        return Err(FeagiNetworkError::InvalidSocketProperties(format!(
            "Invalid ZMQ URL '{}': must start with one of {:?}",
            url, VALID_PREFIXES
        )));
    }

    // Basic format validation for tcp URLs
    if url.starts_with("tcp://") {
        let addr_part = &url[6..]; // Skip "tcp://"
        if addr_part.is_empty() {
            return Err(FeagiNetworkError::InvalidSocketProperties(
                "Invalid ZMQ URL: empty address after tcp://".to_string(),
            ));
        }
        // Should contain host:port or *:port for binding
        if !addr_part.contains(':') {
            return Err(FeagiNetworkError::InvalidSocketProperties(format!(
                "Invalid ZMQ TCP URL '{}': missing port (expected host:port)",
                url
            )));
        }
    }

    Ok(())
}

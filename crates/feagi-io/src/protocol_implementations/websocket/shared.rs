//! Shared utilities for WebSocket implementations.

use crate::FeagiNetworkError;
use serde::{Deserialize, Serialize};

/// URL endpoint struct for WebSocket endpoints with validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebSocketUrl {
    url: String,
}

impl WebSocketUrl {
    /// Creates a new WebSocketUrl after validating the format.
    ///
    /// The URL will be normalized to include the `ws://` or `wss://` scheme if not present.
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL (e.g., "ws://localhost:8080", "wss://example.com/path", "localhost:8080").
    ///
    /// # Errors
    ///
    /// Returns an error if the URL format is invalid.
    pub fn new(url: &str) -> Result<Self, FeagiNetworkError> {
        let normalized = normalize_ws_url(url);
        validate_ws_url(&normalized)?;
        Ok(WebSocketUrl { url: normalized })
    }

    /// Returns the URL as a string slice.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.url
    }

    /// Extracts host:port from the WebSocket URL for TCP connection.
    ///
    /// Strips the `ws://` or `wss://` scheme and any path component,
    /// returning just the `host:port` portion suitable for TCP connection.
    /// If no port is specified, defaults to port 80 for ws:// or 443 for wss://.
    pub fn host_port(&self) -> String {
        let is_secure = self.url.starts_with("wss://");

        // Remove ws:// or wss:// prefix
        let without_scheme = self
            .url
            .strip_prefix("ws://")
            .or_else(|| self.url.strip_prefix("wss://"))
            .unwrap_or(&self.url);

        // Remove any path component
        let host_port = without_scheme.split('/').next().unwrap_or(without_scheme);

        // Add default port if not specified
        if host_port.contains(':') {
            host_port.to_string()
        } else {
            // Default WebSocket ports: 80 for ws://, 443 for wss://
            let default_port = if is_secure { 443 } else { 80 };
            format!("{}:{}", host_port, default_port)
        }
    }

    /// Returns whether this is a secure WebSocket URL (wss://).
    #[allow(dead_code)]
    pub fn is_secure(&self) -> bool {
        self.url.starts_with("wss://")
    }
}

impl std::fmt::Display for WebSocketUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

/// Normalizes a host string to a valid WebSocket URL.
///
/// If the host already starts with `ws://` or `wss://`, it's returned as-is.
/// Otherwise, `ws://` is prepended to the host.
///
/// # Examples
/// ```ignore
/// assert_eq!(normalize_ws_url("localhost:8080"), "ws://localhost:8080");
/// assert_eq!(normalize_ws_url("ws://localhost:8080"), "ws://localhost:8080");
/// assert_eq!(normalize_ws_url("wss://secure.example.com"), "wss://secure.example.com");
/// ```
fn normalize_ws_url(url: &str) -> String {
    if url.starts_with("ws://") || url.starts_with("wss://") {
        url.to_string()
    } else {
        format!("ws://{}", url)
    }
}

/// Validates a WebSocket URL format.
///
/// Valid schemes: ws://, wss://
fn validate_ws_url(url: &str) -> Result<(), FeagiNetworkError> {
    // Check for valid WebSocket scheme prefixes
    const VALID_PREFIXES: [&str; 2] = ["ws://", "wss://"];

    if !VALID_PREFIXES.iter().any(|prefix| url.starts_with(prefix)) {
        return Err(FeagiNetworkError::InvalidSocketProperties(format!(
            "Invalid WebSocket URL '{}': must start with one of {:?}",
            url, VALID_PREFIXES
        )));
    }

    // Extract the part after the scheme
    let addr_part = url
        .strip_prefix("wss://")
        .or_else(|| url.strip_prefix("ws://"))
        .ok_or_else(|| {
            FeagiNetworkError::InvalidSocketProperties(format!(
                "Invalid WebSocket URL '{}': expected {:?}",
                url, VALID_PREFIXES
            ))
        })?;

    if addr_part.is_empty() {
        return Err(FeagiNetworkError::InvalidSocketProperties(format!(
            "Invalid WebSocket URL '{}': empty address after scheme",
            url
        )));
    }

    // Extract host:port (before any path)
    let host_port = addr_part.split('/').next().unwrap_or(addr_part);

    if host_port.is_empty() {
        return Err(FeagiNetworkError::InvalidSocketProperties(format!(
            "Invalid WebSocket URL '{}': empty host",
            url
        )));
    }

    Ok(())
}

/// Validates a bind address format (host:port).
///
/// # Arguments
///
/// * `bind_address` - The address to bind to (e.g., "127.0.0.1:8080", "0.0.0.0:8080").
///
/// # Errors
///
/// Returns an error if the address format is invalid.
#[allow(dead_code)]
pub fn validate_bind_address(bind_address: &str) -> Result<(), FeagiNetworkError> {
    if bind_address.is_empty() {
        return Err(FeagiNetworkError::InvalidSocketProperties(
            "Invalid bind address: empty string".to_string(),
        ));
    }

    // Check for scheme prefixes that shouldn't be in bind addresses
    if bind_address.starts_with("ws://")
        || bind_address.starts_with("wss://")
        || bind_address.starts_with("http://")
        || bind_address.starts_with("https://")
    {
        return Err(FeagiNetworkError::InvalidSocketProperties(format!(
            "Invalid bind address '{}': should be host:port without scheme",
            bind_address
        )));
    }

    // Should contain host:port
    if !bind_address.contains(':') {
        return Err(FeagiNetworkError::InvalidSocketProperties(format!(
            "Invalid bind address '{}': missing port (expected host:port)",
            bind_address
        )));
    }

    Ok(())
}

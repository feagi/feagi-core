//! Shared utility functions for WebSocket implementations.

use serde::{Deserialize, Serialize};
use crate::FeagiNetworkError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebSocketUrl {
    url: String,
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
pub fn normalize_ws_url(host: &str) -> String {
    if host.starts_with("ws://") || host.starts_with("wss://") {
        host.to_string()
    } else {
        format!("ws://{}", host)
    }
}

/// Extracts host:port from a WebSocket URL for TCP connection.
///
/// Strips the `ws://` or `wss://` scheme and any path component,
/// returning just the `host:port` portion suitable for TCP connection.
/// If no port is specified, defaults to port 80.
///
/// # Examples
/// ```ignore
/// assert_eq!(extract_host_port("ws://localhost:8080/path"), Ok("localhost:8080".to_string()));
/// assert_eq!(extract_host_port("wss://example.com"), Ok("example.com:80".to_string()));
/// ```
pub fn extract_host_port(url: &str) -> Result<String, FeagiNetworkError> {
    // Remove ws:// or wss:// prefix
    let without_scheme = url
        .strip_prefix("ws://")
        .or_else(|| url.strip_prefix("wss://"))
        .unwrap_or(url);

    // Remove any path component
    let host_port = without_scheme.split('/').next().unwrap_or(without_scheme);

    // Add default port if not specified
    if host_port.contains(':') {
        Ok(host_port.to_string())
    } else {
        // Default WebSocket port
        Ok(format!("{}:80", host_port))
    }
}

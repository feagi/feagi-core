//! Common configuration types for transports

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Generic transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Address to bind (server) or connect (client)
    pub address: String,
    
    /// Timeout for blocking operations (None = infinite)
    pub timeout: Option<Duration>,
    
    /// High water mark for send buffer (0 = unlimited)
    pub send_hwm: usize,
    
    /// High water mark for receive buffer (0 = unlimited)
    pub recv_hwm: usize,
    
    /// Linger time on close (None = immediate)
    pub linger: Option<Duration>,
    
    /// Maximum message size (None = unlimited)
    pub max_message_size: Option<usize>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            address: "tcp://127.0.0.1:5555".to_string(),
            timeout: Some(Duration::from_secs(1)),
            send_hwm: 1000,
            recv_hwm: 1000,
            linger: Some(Duration::from_millis(1000)),
            max_message_size: Some(10 * 1024 * 1024), // 10 MB default
        }
    }
}

impl TransportConfig {
    /// Create a new config with the given address
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            ..Default::default()
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Set no timeout (blocking)
    pub fn with_no_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }
    
    /// Set send high water mark
    pub fn with_send_hwm(mut self, hwm: usize) -> Self {
        self.send_hwm = hwm;
        self
    }
    
    /// Set receive high water mark
    pub fn with_recv_hwm(mut self, hwm: usize) -> Self {
        self.recv_hwm = hwm;
        self
    }
    
    /// Set linger time
    pub fn with_linger(mut self, linger: Duration) -> Self {
        self.linger = Some(linger);
        self
    }
    
    /// Set no linger (immediate close)
    pub fn with_no_linger(mut self) -> Self {
        self.linger = None;
        self
    }
    
    /// Set maximum message size
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = Some(size);
        self
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.address.is_empty() {
            return Err("Address cannot be empty".to_string());
        }
        
        if let Some(max_size) = self.max_message_size {
            if max_size == 0 {
                return Err("Maximum message size must be greater than 0".to_string());
            }
        }
        
        Ok(())
    }
}

/// Server-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Base transport config
    #[serde(flatten)]
    pub base: TransportConfig,
    
    /// Maximum number of concurrent connections (0 = unlimited)
    pub max_connections: usize,
    
    /// Enable connection tracking
    pub track_connections: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            base: TransportConfig::default(),
            max_connections: 0,
            track_connections: true,
        }
    }
}

impl ServerConfig {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            base: TransportConfig::new(address),
            ..Default::default()
        }
    }
}

/// Client-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Base transport config
    #[serde(flatten)]
    pub base: TransportConfig,
    
    /// Reconnect automatically on disconnect
    pub auto_reconnect: bool,
    
    /// Reconnect delay
    pub reconnect_delay: Duration,
    
    /// Maximum reconnect attempts (0 = unlimited)
    pub max_reconnect_attempts: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base: TransportConfig::default(),
            auto_reconnect: true,
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_attempts: 0,
        }
    }
}

impl ClientConfig {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            base: TransportConfig::new(address),
            ..Default::default()
        }
    }
}


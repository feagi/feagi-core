//! Common message types for transports

use serde::{Deserialize, Serialize};

/// Generic message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message payload
    pub data: Vec<u8>,
    
    /// Optional metadata
    pub metadata: Option<MessageMetadata>,
}

impl Message {
    /// Create a new message
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            metadata: None,
        }
    }
    
    /// Create a message with metadata
    pub fn with_metadata(data: Vec<u8>, metadata: MessageMetadata) -> Self {
        Self {
            data,
            metadata: Some(metadata),
        }
    }
    
    /// Get message size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl From<Vec<u8>> for Message {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&[u8]> for Message {
    fn from(data: &[u8]) -> Self {
        Self::new(data.to_vec())
    }
}

/// Message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Message ID
    pub id: Option<String>,
    
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp: Option<u64>,
    
    /// Sender identity
    pub sender: Option<String>,
    
    /// Message type/topic
    pub topic: Option<String>,
    
    /// Custom key-value pairs
    pub custom: std::collections::HashMap<String, String>,
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            id: None,
            timestamp: None,
            sender: None,
            topic: None,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// Reply handle for request-reply patterns
pub trait ReplyHandle: Send {
    /// Send reply
    fn send(&self, data: &[u8]) -> Result<(), crate::common::error::TransportError>;
    
    /// Send error reply
    fn send_error(&self, error: &str) -> Result<(), crate::common::error::TransportError> {
        let error_msg = serde_json::json!({
            "error": error
        });
        let data = serde_json::to_vec(&error_msg)?;
        self.send(&data)
    }
}

/// Multipart message (for ZMQ and similar transports)
#[derive(Debug, Clone)]
pub struct MultipartMessage {
    /// Message parts
    pub parts: Vec<Vec<u8>>,
}

impl MultipartMessage {
    /// Create a new multipart message
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }
    
    /// Add a part
    pub fn add_part(mut self, part: Vec<u8>) -> Self {
        self.parts.push(part);
        self
    }
    
    /// Get number of parts
    pub fn len(&self) -> usize {
        self.parts.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
    
    /// Get total size in bytes
    pub fn total_size(&self) -> usize {
        self.parts.iter().map(|p| p.len()).sum()
    }
}

impl Default for MultipartMessage {
    fn default() -> Self {
        Self::new()
    }
}




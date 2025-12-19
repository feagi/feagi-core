// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Transport trait definitions
//!
//! These traits define the common interface for all transport implementations.
//! Transport implementations can be for different protocols (ZMQ, UDP, SHM) and
//! different roles (server, client).

use crate::common::{MultipartMessage, ReplyHandle, TransportResult};

/// Base transport trait - implemented by all transports
pub trait Transport: Send + Sync {
    /// Start the transport
    fn start(&mut self) -> TransportResult<()>;
    
    /// Stop the transport
    fn stop(&mut self) -> TransportResult<()>;
    
    /// Check if transport is running
    fn is_running(&self) -> bool;
    
    /// Get transport name/type
    fn transport_type(&self) -> &str;
}

/// Request-Reply pattern (Client side)
///
/// Used for synchronous RPC-style communication where the client sends a request
/// and waits for a reply.
pub trait RequestReplyClient: Transport {
    /// Send a request and wait for reply
    fn request(&self, data: &[u8]) -> TransportResult<Vec<u8>>;
    
    /// Send a request with timeout
    fn request_timeout(
        &self,
        data: &[u8],
        timeout_ms: u64,
    ) -> TransportResult<Vec<u8>>;
    
    /// Send request without waiting for reply (fire and forget)
    fn send(&self, data: &[u8]) -> TransportResult<()>;
}

/// Request-Reply pattern (Server side)
///
/// Used for handling incoming requests and sending replies.
pub trait RequestReplyServer: Transport {
    /// Receive a request and get a reply handle
    fn receive(&self) -> TransportResult<(Vec<u8>, Box<dyn ReplyHandle>)>;
    
    /// Receive with timeout
    fn receive_timeout(
        &self,
        timeout_ms: u64,
    ) -> TransportResult<(Vec<u8>, Box<dyn ReplyHandle>)>;
    
    /// Poll for incoming messages (non-blocking)
    fn poll(&self, timeout_ms: u64) -> TransportResult<bool>;
}

/// Publish-Subscribe pattern (Publisher side)
///
/// Used for one-to-many broadcast communication.
pub trait Publisher: Transport {
    /// Publish a message to all subscribers
    fn publish(&self, topic: &[u8], data: &[u8]) -> TransportResult<()>;
    
    /// Publish a message (single frame with topic prefix)
    fn publish_simple(&self, data: &[u8]) -> TransportResult<()>;
}

/// Publish-Subscribe pattern (Subscriber side)
///
/// Used for receiving broadcast messages from publishers.
pub trait Subscriber: Transport {
    /// Subscribe to a topic
    fn subscribe(&mut self, topic: &[u8]) -> TransportResult<()>;
    
    /// Unsubscribe from a topic
    fn unsubscribe(&mut self, topic: &[u8]) -> TransportResult<()>;
    
    /// Receive a published message
    fn receive(&self) -> TransportResult<(Vec<u8>, Vec<u8>)>; // (topic, data)
    
    /// Receive with timeout
    fn receive_timeout(&self, timeout_ms: u64) -> TransportResult<(Vec<u8>, Vec<u8>)>;
}

/// Push-Pull pattern (Push side)
///
/// Used for distributing work to multiple workers (load balancing).
pub trait Push: Transport {
    /// Push a message to the pull queue
    fn push(&self, data: &[u8]) -> TransportResult<()>;
    
    /// Push with timeout
    fn push_timeout(&self, data: &[u8], timeout_ms: u64) -> TransportResult<()>;
}

/// Push-Pull pattern (Pull side)
///
/// Used for receiving distributed work.
pub trait Pull: Transport {
    /// Pull a message from the queue
    fn pull(&self) -> TransportResult<Vec<u8>>;
    
    /// Pull with timeout
    fn pull_timeout(&self, timeout_ms: u64) -> TransportResult<Vec<u8>>;
}

/// Multipart message support
///
/// For transports that support sending/receiving multiple frames in a single message.
pub trait MultipartTransport: Transport {
    /// Send a multipart message
    fn send_multipart(&self, msg: &MultipartMessage) -> TransportResult<()>;
    
    /// Receive a multipart message
    fn receive_multipart(&self) -> TransportResult<MultipartMessage>;
}

/// Connection tracking (for server-side transports)
///
/// Allows tracking of connected clients and their states.
pub trait ConnectionTracker {
    /// Get number of active connections
    fn connection_count(&self) -> usize;
    
    /// Get list of connected client IDs
    fn connected_clients(&self) -> Vec<String>;
    
    /// Check if a specific client is connected
    fn is_client_connected(&self, client_id: &str) -> bool;
}

/// Statistics tracking
///
/// For monitoring transport performance and health.
pub trait TransportStats {
    /// Get total messages sent
    fn messages_sent(&self) -> u64;
    
    /// Get total messages received
    fn messages_received(&self) -> u64;
    
    /// Get total bytes sent
    fn bytes_sent(&self) -> u64;
    
    /// Get total bytes received
    fn bytes_received(&self) -> u64;
    
    /// Get error count
    fn error_count(&self) -> u64;
}


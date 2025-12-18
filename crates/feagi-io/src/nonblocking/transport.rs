// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Non-blocking transport trait for async/await transports
//!
//! PLACEHOLDER: This will be implemented in Phase 3 when adding UDP support.

use crate::core::{Result, SharedFBC};

/// Trait for transports that use async/await with tokio
///
/// # Design
/// - Operations are async and return futures
/// - Typically used with tokio runtime
/// - Good for: UDP (tokio::net::UdpSocket), WebSocket, HTTP
///
/// # Implementation
/// This trait will be fully implemented when we add UDP transport support.
#[async_trait::async_trait]
pub trait NonBlockingTransport: Send + Sync {
    /// Get the transport backend name (e.g., "udp", "websocket")
    fn backend_name(&self) -> &str;

    /// Start the transport (bind sockets, spawn tasks, etc.)
    async fn start(&mut self) -> Result<()>;

    /// Stop the transport (shutdown tasks, close connections, etc.)
    async fn stop(&mut self) -> Result<()>;

    /// Publish visualization data to all subscribers
    async fn publish_visualization(&self, fbc: SharedFBC) -> Result<()>;

    /// Publish motor commands to a specific agent
    async fn publish_motor(&self, agent_id: &str, fbc: SharedFBC) -> Result<()>;
}

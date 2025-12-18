// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket PUSH pattern (client-side send)
//!
//! Sends messages to a WebSocket pull server.
//! Used for sending sensory data to FEAGI.

use crate::common::{ClientConfig, TransportError, TransportResult};
use crate::traits::{Push, Transport};
use futures_util::SinkExt;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::info;

/// WebSocket PUSH socket implementation (sender)
pub struct WsPush {
    config: ClientConfig,
    running: Arc<RwLock<bool>>,
    ws_stream: Arc<RwLock<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
}

impl WsPush {
    /// Create a new WebSocket PUSH socket
    pub fn new(config: ClientConfig) -> TransportResult<Self> {
        config.base.validate()?;

        Ok(Self {
            config,
            running: Arc::new(RwLock::new(false)),
            ws_stream: Arc::new(RwLock::new(None)),
        })
    }

    /// Create with address
    pub async fn with_address(address: impl Into<String>) -> TransportResult<Self> {
        let config = ClientConfig::new(address);
        Self::new(config)
    }

    /// Start the WebSocket client
    pub async fn start_async(&mut self) -> TransportResult<()> {
        if *self.running.read() {
            return Err(TransportError::AlreadyRunning);
        }

        let url = if self.config.base.address.starts_with("ws://")
            || self.config.base.address.starts_with("wss://")
        {
            self.config.base.address.clone()
        } else {
            format!("ws://{}", self.config.base.address)
        };

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| TransportError::ConnectFailed(e.to_string()))?;

        info!("🦀 [WS-PUSH] Connected to {}", url);

        *self.ws_stream.write() = Some(ws_stream);
        *self.running.write() = true;

        Ok(())
    }

    /// Push a message asynchronously
    pub async fn push_async(&self, data: &[u8]) -> TransportResult<()> {
        let mut ws_guard = self.ws_stream.write();
        let ws = ws_guard.as_mut().ok_or(TransportError::NotRunning)?;

        ws.send(Message::Binary(data.to_vec()))
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        Ok(())
    }
}

impl Transport for WsPush {
    fn start(&mut self) -> TransportResult<()> {
        Err(TransportError::Other(
            "Use start_async() for WebSocket transports".to_string(),
        ))
    }

    fn stop(&mut self) -> TransportResult<()> {
        *self.running.write() = false;
        *self.ws_stream.write() = None;
        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.read()
    }

    fn transport_type(&self) -> &str {
        "websocket-push"
    }
}

impl Push for WsPush {
    fn push(&self, data: &[u8]) -> TransportResult<()> {
        // Note: This is a blocking call for a sync trait
        // For async usage, prefer push_async()
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| TransportError::Other("No tokio runtime".to_string()))?;

        handle.block_on(self.push_async(data))
    }

    fn push_timeout(&self, data: &[u8], timeout_ms: u64) -> TransportResult<()> {
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| TransportError::Other("No tokio runtime".to_string()))?;

        let timeout = std::time::Duration::from_millis(timeout_ms);

        handle.block_on(async {
            tokio::time::timeout(timeout, self.push_async(data))
                .await
                .map_err(|_| TransportError::Timeout)?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wspush_creation() {
        let config = ClientConfig::new("ws://127.0.0.1:30027");
        let push_socket = WsPush::new(config);
        assert!(push_socket.is_ok());
    }
}

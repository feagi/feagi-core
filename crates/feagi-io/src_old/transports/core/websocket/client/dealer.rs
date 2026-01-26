// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket DEALER pattern (client-side request-reply)
//!
//! Sends requests and receives replies from a WebSocket router.
//! Used for control plane communication in FEAGI agents.

use crate::transports::core::common::{ClientConfig, TransportError, TransportResult};
use crate::transports::core::traits::{RequestReplyClient, Transport};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::info;

/// WebSocket DEALER socket implementation (request-reply client)
pub struct WsDealer {
    config: ClientConfig,
    running: Arc<RwLock<bool>>,
    ws_stream: Arc<RwLock<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
}

impl WsDealer {
    /// Create a new WebSocket DEALER socket
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

        info!("🦀 [WS-DEALER] Connected to {}", url);

        *self.ws_stream.write() = Some(ws_stream);
        *self.running.write() = true;

        Ok(())
    }

    /// Send request and receive reply asynchronously
    pub async fn request_async(&self, data: &[u8]) -> TransportResult<Vec<u8>> {
        let mut ws_guard = self.ws_stream.write();
        let ws = ws_guard.as_mut().ok_or(TransportError::NotRunning)?;

        // Send request
        ws.send(Message::Binary(data.to_vec()))
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        // Wait for reply
        match ws.next().await {
            Some(Ok(Message::Binary(reply))) => Ok(reply),
            Some(Ok(Message::Text(text))) => Ok(text.into_bytes()),
            Some(Ok(Message::Close(_))) => Err(TransportError::ConnectionClosed),
            Some(Err(e)) => Err(TransportError::ReceiveFailed(e.to_string())),
            None => Err(TransportError::ConnectionClosed),
            _ => Err(TransportError::Other("Unexpected message type".to_string())),
        }
    }
}

impl Transport for WsDealer {
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
        "websocket-dealer"
    }
}

impl RequestReplyClient for WsDealer {
    fn request(&self, data: &[u8]) -> TransportResult<Vec<u8>> {
        // Note: This is a blocking call for a sync trait
        // For async usage, prefer request_async()
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| TransportError::Other("No tokio runtime".to_string()))?;

        handle.block_on(self.request_async(data))
    }

    fn request_timeout(&self, data: &[u8], timeout_ms: u64) -> TransportResult<Vec<u8>> {
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| TransportError::Other("No tokio runtime".to_string()))?;

        let timeout = std::time::Duration::from_millis(timeout_ms);

        handle.block_on(async {
            tokio::time::timeout(timeout, self.request_async(data))
                .await
                .map_err(|_| TransportError::Timeout)?
        })
    }

    fn send(&self, data: &[u8]) -> TransportResult<()> {
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| TransportError::Other("No tokio runtime".to_string()))?;

        handle.block_on(async {
            let mut ws_guard = self.ws_stream.write();
            let ws = ws_guard.as_mut().ok_or(TransportError::NotRunning)?;

            ws.send(Message::Binary(data.to_vec()))
                .await
                .map_err(|e| TransportError::SendFailed(e.to_string()))?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wsdealer_creation() {
        let config = ClientConfig::new("ws://127.0.0.1:30028");
        let dealer_socket = WsDealer::new(config);
        assert!(dealer_socket.is_ok());
    }
}

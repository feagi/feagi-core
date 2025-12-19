// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket PULL pattern (server-side receive)
//!
//! Receives messages from multiple WebSocket clients.
//! Used for sensory data input in FEAGI.

use crate::transports::core::common::{ServerConfig, TransportError, TransportResult};
use crate::transports::core::traits::{Pull, Transport};
use futures_util::StreamExt;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// WebSocket PULL socket implementation (receiver)
pub struct WsPull {
    config: ServerConfig,
    running: Arc<RwLock<bool>>,
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    message_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<Vec<u8>>>>>,
    message_tx: Arc<RwLock<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
}

impl WsPull {
    /// Create a new WebSocket PULL socket
    pub fn new(config: ServerConfig) -> TransportResult<Self> {
        config.base.validate()?;

        Ok(Self {
            config,
            running: Arc::new(RwLock::new(false)),
            server_handle: Arc::new(RwLock::new(None)),
            message_rx: Arc::new(RwLock::new(None)),
            message_tx: Arc::new(RwLock::new(None)),
        })
    }

    /// Create with address
    pub async fn with_address(address: impl Into<String>) -> TransportResult<Self> {
        let config = ServerConfig::new(address);
        Self::new(config)
    }

    /// Start the WebSocket server
    pub async fn start_async(&mut self) -> TransportResult<()> {
        if *self.running.read() {
            return Err(TransportError::AlreadyRunning);
        }

        let addr = self.config.base.address.clone();
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| TransportError::BindFailed(e.to_string()))?;

        info!("🦀 [WS-PULL] Listening on {}", addr);

        let (message_tx, message_rx) = mpsc::unbounded_channel();
        *self.message_tx.write() = Some(message_tx.clone());
        *self.message_rx.write() = Some(message_rx);
        *self.running.write() = true;

        let running = self.running.clone();

        let handle = tokio::spawn(async move {
            while *running.read() {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        debug!("[WS-PULL] New connection from {}", peer_addr);

                        let message_tx_clone = message_tx.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, peer_addr, message_tx_clone).await
                            {
                                warn!("[WS-PULL] Client {} error: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("[WS-PULL] Accept error: {}", e);
                    }
                }
            }
        });

        *self.server_handle.write() = Some(handle);

        Ok(())
    }
}

impl Transport for WsPull {
    fn start(&mut self) -> TransportResult<()> {
        Err(TransportError::Other(
            "Use start_async() for WebSocket transports".to_string(),
        ))
    }

    fn stop(&mut self) -> TransportResult<()> {
        *self.running.write() = false;
        *self.message_tx.write() = None;
        *self.message_rx.write() = None;

        if let Some(handle) = self.server_handle.write().take() {
            handle.abort();
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.read()
    }

    fn transport_type(&self) -> &str {
        "websocket-pull"
    }
}

impl Pull for WsPull {
    fn pull(&self) -> TransportResult<Vec<u8>> {
        let mut rx_guard = self.message_rx.write();
        let rx = rx_guard.as_mut().ok_or(TransportError::NotRunning)?;

        match rx.try_recv() {
            Ok(data) => Ok(data),
            Err(mpsc::error::TryRecvError::Empty) => Err(TransportError::NoData),
            Err(e) => Err(TransportError::ReceiveFailed(e.to_string())),
        }
    }

    fn pull_timeout(&self, timeout_ms: u64) -> TransportResult<Vec<u8>> {
        let mut rx_guard = self.message_rx.write();
        let rx = rx_guard.as_mut().ok_or(TransportError::NotRunning)?;

        // Use a different approach since tokio doesn't have blocking_recv_timeout
        let start = std::time::Instant::now();
        loop {
            match rx.try_recv() {
                Ok(data) => return Ok(data),
                Err(mpsc::error::TryRecvError::Empty) => {
                    if start.elapsed().as_millis() as u64 >= timeout_ms {
                        return Err(TransportError::Timeout);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(e) => return Err(TransportError::ReceiveFailed(e.to_string())),
            }
        }
    }
}

/// Handle a single WebSocket client connection
async fn handle_client(
    stream: TcpStream,
    peer_addr: SocketAddr,
    message_tx: mpsc::UnboundedSender<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (_write, mut read) = ws_stream.split();

    info!("[WS-PULL] Client {} connected", peer_addr);

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                if message_tx.send(data).is_err() {
                    break;
                }
            }
            Ok(Message::Text(text)) => {
                if message_tx.send(text.into_bytes()).is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                warn!("[WS-PULL] Message error from {}: {}", peer_addr, e);
                break;
            }
            _ => {}
        }
    }

    info!("[WS-PULL] Client {} disconnected", peer_addr);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wspull_creation() {
        let config = ServerConfig::new("127.0.0.1:30022");
        let pull_socket = WsPull::new(config);
        assert!(pull_socket.is_ok());
    }

    #[tokio::test]
    async fn test_wspull_start_stop() {
        let mut pull_socket = WsPull::with_address("127.0.0.1:30023").await.unwrap();
        assert!(!pull_socket.is_running());

        pull_socket.start_async().await.unwrap();
        assert!(pull_socket.is_running());

        pull_socket.stop().unwrap();
        assert!(!pull_socket.is_running());
    }
}

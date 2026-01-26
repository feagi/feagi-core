// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket ROUTER pattern (server-side request-reply)
//!
//! Handles request-reply communication with routing to specific clients.
//! Used for per-agent control channels in FEAGI.

use crate::transports::core::common::{ReplyHandle, ServerConfig, TransportError, TransportResult};
use crate::transports::core::traits::{RequestReplyServer, Transport};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use std::cell::RefCell;

/// Reply handle for WebSocket router
struct WsReplyHandle {
    reply_tx: RefCell<Option<oneshot::Sender<Vec<u8>>>>,
}

impl ReplyHandle for WsReplyHandle {
    fn send(&self, data: &[u8]) -> TransportResult<()> {
        // Take ownership of the sender
        if let Some(tx) = self.reply_tx.borrow_mut().take() {
            tx.send(data.to_vec())
                .map_err(|_| TransportError::SendFailed("Reply channel closed".to_string()))?;
            Ok(())
        } else {
            Err(TransportError::SendFailed("Reply already sent".to_string()))
        }
    }
}

/// WebSocket ROUTER socket implementation
pub struct WsRouter {
    config: ServerConfig,
    running: Arc<RwLock<bool>>,
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    request_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<(Vec<u8>, oneshot::Sender<Vec<u8>>)>>>>,
    request_tx: Arc<RwLock<Option<mpsc::UnboundedSender<(Vec<u8>, oneshot::Sender<Vec<u8>>)>>>>,
}

impl WsRouter {
    /// Create a new WebSocket ROUTER socket
    pub fn new(config: ServerConfig) -> TransportResult<Self> {
        config.base.validate()?;

        Ok(Self {
            config,
            running: Arc::new(RwLock::new(false)),
            server_handle: Arc::new(RwLock::new(None)),
            request_rx: Arc::new(RwLock::new(None)),
            request_tx: Arc::new(RwLock::new(None)),
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

        info!("🦀 [WS-ROUTER] Listening on {}", addr);

        let (request_tx, request_rx) = mpsc::unbounded_channel();
        *self.request_tx.write() = Some(request_tx.clone());
        *self.request_rx.write() = Some(request_rx);
        *self.running.write() = true;

        let running = self.running.clone();

        let handle = tokio::spawn(async move {
            while *running.read() {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        debug!("[WS-ROUTER] New connection from {}", peer_addr);

                        let request_tx_clone = request_tx.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, peer_addr, request_tx_clone).await
                            {
                                warn!("[WS-ROUTER] Client {} error: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("[WS-ROUTER] Accept error: {}", e);
                    }
                }
            }
        });

        *self.server_handle.write() = Some(handle);

        Ok(())
    }
}

impl Transport for WsRouter {
    fn start(&mut self) -> TransportResult<()> {
        Err(TransportError::Other(
            "Use start_async() for WebSocket transports".to_string(),
        ))
    }

    fn stop(&mut self) -> TransportResult<()> {
        *self.running.write() = false;
        *self.request_tx.write() = None;
        *self.request_rx.write() = None;

        if let Some(handle) = self.server_handle.write().take() {
            handle.abort();
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.read()
    }

    fn transport_type(&self) -> &str {
        "websocket-router"
    }
}

impl RequestReplyServer for WsRouter {
    fn receive(&self) -> TransportResult<(Vec<u8>, Box<dyn ReplyHandle>)> {
        let mut rx_guard = self.request_rx.write();
        let rx = rx_guard.as_mut().ok_or(TransportError::NotRunning)?;

        match rx.try_recv() {
            Ok((data, reply_tx)) => {
                let handle = Box::new(WsReplyHandle {
                    reply_tx: RefCell::new(Some(reply_tx)),
                });
                Ok((data, handle))
            }
            Err(mpsc::error::TryRecvError::Empty) => Err(TransportError::NoData),
            Err(e) => Err(TransportError::ReceiveFailed(e.to_string())),
        }
    }

    fn receive_timeout(&self, timeout_ms: u64) -> TransportResult<(Vec<u8>, Box<dyn ReplyHandle>)> {
        let mut rx_guard = self.request_rx.write();
        let rx = rx_guard.as_mut().ok_or(TransportError::NotRunning)?;

        // Use a different approach since tokio doesn't have blocking_recv_timeout
        let start = std::time::Instant::now();
        loop {
            match rx.try_recv() {
                Ok((data, reply_tx)) => {
                    let handle = Box::new(WsReplyHandle {
                        reply_tx: RefCell::new(Some(reply_tx)),
                    });
                    return Ok((data, handle));
                }
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

    fn poll(&self, timeout_ms: u64) -> TransportResult<bool> {
        self.receive_timeout(timeout_ms).map(|_| true)
    }
}

/// Handle a single WebSocket client connection
async fn handle_client(
    stream: TcpStream,
    peer_addr: SocketAddr,
    request_tx: mpsc::UnboundedSender<(Vec<u8>, oneshot::Sender<Vec<u8>>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    info!("[WS-ROUTER] Client {} connected", peer_addr);

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                let (reply_tx, reply_rx) = oneshot::channel();

                if request_tx.send((data, reply_tx)).is_err() {
                    break;
                }

                // Wait for reply
                match reply_rx.await {
                    Ok(response) => {
                        if write.send(Message::Binary(response)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            Ok(Message::Text(text)) => {
                let (reply_tx, reply_rx) = oneshot::channel();

                if request_tx.send((text.into_bytes(), reply_tx)).is_err() {
                    break;
                }

                // Wait for reply
                match reply_rx.await {
                    Ok(response) => {
                        if write.send(Message::Binary(response)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                warn!("[WS-ROUTER] Message error from {}: {}", peer_addr, e);
                break;
            }
            _ => {}
        }
    }

    info!("[WS-ROUTER] Client {} disconnected", peer_addr);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wsrouter_creation() {
        let config = ServerConfig::new("127.0.0.1:30024");
        let router_socket = WsRouter::new(config);
        assert!(router_socket.is_ok());
    }

    #[tokio::test]
    async fn test_wsrouter_start_stop() {
        let mut router_socket = WsRouter::with_address("127.0.0.1:30025").await.unwrap();
        assert!(!router_socket.is_running());

        router_socket.start_async().await.unwrap();
        assert!(router_socket.is_running());

        router_socket.stop().unwrap();
        assert!(!router_socket.is_running());
    }
}

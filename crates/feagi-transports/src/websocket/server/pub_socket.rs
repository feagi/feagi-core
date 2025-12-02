// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket PUB pattern (server-side publish-subscribe)
//!
//! Broadcasts messages to all connected WebSocket clients.
//! Used for visualization data and motor commands in FEAGI.

use crate::common::{ServerConfig, TransportError, TransportResult};
use crate::traits::{Publisher, Transport};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// WebSocket PUB socket implementation (publisher)
pub struct WsPub {
    config: ServerConfig,
    running: Arc<RwLock<bool>>,
    clients: Arc<RwLock<HashMap<SocketAddr, broadcast::Sender<Vec<u8>>>>>,
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    broadcast_tx: Arc<RwLock<Option<broadcast::Sender<Vec<u8>>>>>,
}

impl WsPub {
    /// Create a new WebSocket PUB socket
    pub fn new(config: ServerConfig) -> TransportResult<Self> {
        config.base.validate()?;
        
        Ok(Self {
            config,
            running: Arc::new(RwLock::new(false)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            server_handle: Arc::new(RwLock::new(None)),
            broadcast_tx: Arc::new(RwLock::new(None)),
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
        
        info!("🦀 [WS-PUB] Listening on {}", addr);
        
        let (broadcast_tx, _) = broadcast::channel(1000);
        *self.broadcast_tx.write() = Some(broadcast_tx.clone());
        *self.running.write() = true;
        
        let clients = self.clients.clone();
        let running = self.running.clone();
        
        let handle = tokio::spawn(async move {
            while *running.read() {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        debug!("[WS-PUB] New connection from {}", peer_addr);
                        
                        let broadcast_rx = broadcast_tx.subscribe();
                        let clients_clone = clients.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, peer_addr, broadcast_rx, clients_clone).await {
                                warn!("[WS-PUB] Client {} error: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("[WS-PUB] Accept error: {}", e);
                    }
                }
            }
        });
        
        *self.server_handle.write() = Some(handle);
        
        Ok(())
    }
}

impl Transport for WsPub {
    fn start(&mut self) -> TransportResult<()> {
        Err(TransportError::Other(
            "Use start_async() for WebSocket transports".to_string(),
        ))
    }
    
    fn stop(&mut self) -> TransportResult<()> {
        *self.running.write() = false;
        *self.broadcast_tx.write() = None;
        self.clients.write().clear();
        
        if let Some(handle) = self.server_handle.write().take() {
            handle.abort();
        }
        
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        *self.running.read()
    }
    
    fn transport_type(&self) -> &str {
        "websocket-pub"
    }
}

impl Publisher for WsPub {
    fn publish(&self, topic: &[u8], data: &[u8]) -> TransportResult<()> {
        let broadcast_tx = self.broadcast_tx.read();
        let tx = broadcast_tx
            .as_ref()
            .ok_or(TransportError::NotRunning)?;
        
        // Combine topic and data
        let mut message = Vec::with_capacity(topic.len() + data.len() + 1);
        message.extend_from_slice(topic);
        message.push(b'|');
        message.extend_from_slice(data);
        
        // Ignore SendError if no receivers (no clients connected yet)
        // This is normal - messages are dropped if no one is listening
        let _ = tx.send(message);
        
        Ok(())
    }
    
    fn publish_simple(&self, data: &[u8]) -> TransportResult<()> {
        let broadcast_tx = self.broadcast_tx.read();
        let tx = broadcast_tx
            .as_ref()
            .ok_or(TransportError::NotRunning)?;
        
        // Ignore SendError if no receivers (no clients connected yet)
        let _ = tx.send(data.to_vec());
        
        Ok(())
    }
}

/// Handle a single WebSocket client connection
async fn handle_client(
    stream: TcpStream,
    peer_addr: SocketAddr,
    mut broadcast_rx: broadcast::Receiver<Vec<u8>>,
    clients: Arc<RwLock<HashMap<SocketAddr, broadcast::Sender<Vec<u8>>>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, _read) = ws_stream.split();
    
    info!("[WS-PUB] Client {} connected", peer_addr);
    
    while let Ok(data) = broadcast_rx.recv().await {
        if write.send(Message::Binary(data)).await.is_err() {
            break;
        }
    }
    
    clients.write().remove(&peer_addr);
    info!("[WS-PUB] Client {} disconnected", peer_addr);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_wspub_creation() {
        let config = ServerConfig::new("127.0.0.1:30020");
        let pub_socket = WsPub::new(config);
        assert!(pub_socket.is_ok());
    }
    
    #[tokio::test]
    async fn test_wspub_start_stop() {
        let mut pub_socket = WsPub::with_address("127.0.0.1:30021").await.unwrap();
        assert!(!pub_socket.is_running());
        
        pub_socket.start_async().await.unwrap();
        assert!(pub_socket.is_running());
        
        pub_socket.stop().unwrap();
        assert!(!pub_socket.is_running());
    }
}


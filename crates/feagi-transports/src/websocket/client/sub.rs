//! WebSocket SUB pattern (client-side subscribe)
//!
//! Subscribes to messages from a WebSocket publisher.
//! Used for receiving motor commands and visualization data in FEAGI agents.

use crate::common::{ClientConfig, TransportError, TransportResult};
use crate::traits::{Subscriber, Transport};
use futures_util::StreamExt;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

/// WebSocket SUB socket implementation (subscriber)
pub struct WsSub {
    config: ClientConfig,
    running: Arc<RwLock<bool>>,
    subscriptions: Arc<RwLock<HashSet<Vec<u8>>>>,
    client_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    message_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<(Vec<u8>, Vec<u8>)>>>>,
    message_tx: Arc<RwLock<Option<mpsc::UnboundedSender<(Vec<u8>, Vec<u8>)>>>>,
}

impl WsSub {
    /// Create a new WebSocket SUB socket
    pub fn new(config: ClientConfig) -> TransportResult<Self> {
        config.base.validate()?;
        
        Ok(Self {
            config,
            running: Arc::new(RwLock::new(false)),
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            client_handle: Arc::new(RwLock::new(None)),
            message_rx: Arc::new(RwLock::new(None)),
            message_tx: Arc::new(RwLock::new(None)),
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
        
        let url = if self.config.base.address.starts_with("ws://") || self.config.base.address.starts_with("wss://") {
            self.config.base.address.clone()
        } else {
            format!("ws://{}", self.config.base.address)
        };
        
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| TransportError::ConnectFailed(e.to_string()))?;
        
        info!("🦀 [WS-SUB] Connected to {}", url);
        
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        *self.message_tx.write() = Some(message_tx.clone());
        *self.message_rx.write() = Some(message_rx);
        *self.running.write() = true;
        
        let subscriptions = self.subscriptions.clone();
        let running = self.running.clone();
        
        let handle = tokio::spawn(async move {
            if let Err(e) = handle_messages(ws_stream, message_tx, subscriptions, running).await {
                error!("[WS-SUB] Connection error: {}", e);
            }
        });
        
        *self.client_handle.write() = Some(handle);
        
        Ok(())
    }
}

impl Transport for WsSub {
    fn start(&mut self) -> TransportResult<()> {
        Err(TransportError::Other(
            "Use start_async() for WebSocket transports".to_string(),
        ))
    }
    
    fn stop(&mut self) -> TransportResult<()> {
        *self.running.write() = false;
        *self.message_tx.write() = None;
        *self.message_rx.write() = None;
        
        if let Some(handle) = self.client_handle.write().take() {
            handle.abort();
        }
        
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        *self.running.read()
    }
    
    fn transport_type(&self) -> &str {
        "websocket-sub"
    }
}

impl Subscriber for WsSub {
    fn subscribe(&mut self, topic: &[u8]) -> TransportResult<()> {
        self.subscriptions.write().insert(topic.to_vec());
        debug!("[WS-SUB] Subscribed to topic: {:?}", topic);
        Ok(())
    }
    
    fn unsubscribe(&mut self, topic: &[u8]) -> TransportResult<()> {
        self.subscriptions.write().remove(topic);
        debug!("[WS-SUB] Unsubscribed from topic: {:?}", topic);
        Ok(())
    }
    
    fn receive(&self) -> TransportResult<(Vec<u8>, Vec<u8>)> {
        let mut rx_guard = self.message_rx.write();
        let rx = rx_guard
            .as_mut()
            .ok_or(TransportError::NotRunning)?;
        
        match rx.try_recv() {
            Ok((topic, data)) => Ok((topic, data)),
            Err(mpsc::error::TryRecvError::Empty) => {
                Err(TransportError::NoData)
            }
            Err(e) => Err(TransportError::ReceiveFailed(e.to_string())),
        }
    }
    
    fn receive_timeout(&self, timeout_ms: u64) -> TransportResult<(Vec<u8>, Vec<u8>)> {
        let mut rx_guard = self.message_rx.write();
        let rx = rx_guard
            .as_mut()
            .ok_or(TransportError::NotRunning)?;
        
        // Use a different approach since tokio doesn't have blocking_recv_timeout
        let start = std::time::Instant::now();
        loop {
            match rx.try_recv() {
                Ok((topic, data)) => return Ok((topic, data)),
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

/// Handle incoming WebSocket messages
async fn handle_messages(
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    message_tx: mpsc::UnboundedSender<(Vec<u8>, Vec<u8>)>,
    subscriptions: Arc<RwLock<HashSet<Vec<u8>>>>,
    running: Arc<RwLock<bool>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_write, mut read) = ws_stream.split();
    
    while *running.read() {
        match read.next().await {
            Some(Ok(Message::Binary(data))) => {
                // Split topic and data by '|' delimiter
                if let Some(delimiter_pos) = data.iter().position(|&b| b == b'|') {
                    let topic = data[..delimiter_pos].to_vec();
                    let payload = data[delimiter_pos + 1..].to_vec();
                    
                    // Check if subscribed to this topic
                    let subscriptions_guard = subscriptions.read();
                    if subscriptions_guard.is_empty() || subscriptions_guard.contains(&topic) {
                        if message_tx.send((topic, payload)).is_err() {
                            break;
                        }
                    }
                } else {
                    // No topic, send as is with empty topic
                    if message_tx.send((vec![], data)).is_err() {
                        break;
                    }
                }
            }
            Some(Ok(Message::Close(_))) => break,
            Some(Err(e)) => {
                warn!("[WS-SUB] Receive error: {}", e);
                break;
            }
            None => break,
            _ => {}
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_wssub_creation() {
        let config = ClientConfig::new("ws://127.0.0.1:30026");
        let sub_socket = WsSub::new(config);
        assert!(sub_socket.is_ok());
    }
}


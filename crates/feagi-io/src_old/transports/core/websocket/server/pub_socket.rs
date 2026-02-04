// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket PUB pattern (server-side publish-subscribe)
//!
//! Broadcasts messages to all connected WebSocket clients.
//! Used for visualization data and motor commands in FEAGI.

use crate::transports::core::common::{ServerConfig, TransportError, TransportResult};
use crate::transports::core::traits::{Publisher, Transport};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
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

        // REAL-TIME SEMANTICS:
        // Visualization (and motor) streams must not buffer historical frames/commands.
        // Keep the channel capacity minimal so slow clients "lag" and skip to newest,
        // rather than drifting farther behind real-time.
        //
        // Note: Tokio broadcast is a ring buffer; if a receiver falls behind, it
        // gets `Lagged(skipped)` and only sees the most recent messages.
        let (broadcast_tx, _) = broadcast::channel(1);
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
                            if let Err(e) =
                                handle_client(stream, peer_addr, broadcast_rx, clients_clone).await
                            {
                                // Log detailed error information for debugging
                                warn!("[WS-PUB] Client {} connection error: {}", peer_addr, e);
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
        let tx = broadcast_tx.as_ref().ok_or(TransportError::NotRunning)?;

        // Combine topic and data
        let mut message = Vec::with_capacity(topic.len() + data.len() + 1);
        message.extend_from_slice(topic);
        message.push(b'|');
        message.extend_from_slice(data);

        // Ignore SendError if no receivers (no clients connected yet)
        // This is normal - messages are dropped if no one is listening
        let _ = tx.send(message);

        // Diagnostics: publish rate and payload size (rate-limited)
        ws_pub_record_publish_stats("ws_viz_topic", topic.len() as u64 + data.len() as u64 + 1);

        Ok(())
    }

    fn publish_simple(&self, data: &[u8]) -> TransportResult<()> {
        let broadcast_tx = self.broadcast_tx.read();
        let tx = broadcast_tx.as_ref().ok_or(TransportError::NotRunning)?;

        // Ignore SendError if no receivers (no clients connected yet)
        let _ = tx.send(data.to_vec());

        // Diagnostics: publish rate and payload size (rate-limited)
        ws_pub_record_publish_stats("ws_viz_simple", data.len() as u64);

        Ok(())
    }
}

/// Record server-side publish stats in a low-overhead, rate-limited way.
///
/// This is logging-only instrumentation to detect whether WebSocket PUB is
/// producing messages faster than clients can consume them.
fn ws_pub_record_publish_stats(stream: &'static str, bytes: u64) {
    static PUBLISHED_TOTAL: AtomicU64 = AtomicU64::new(0);
    static BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);
    static LAST_LOG_MS: AtomicU64 = AtomicU64::new(0);
    static LAST_PUBLISHED_TOTAL: AtomicU64 = AtomicU64::new(0);
    static LAST_BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);

    let published_now = PUBLISHED_TOTAL.fetch_add(1, Ordering::Relaxed) + 1;
    let bytes_now = BYTES_TOTAL.fetch_add(bytes, Ordering::Relaxed) + bytes;

    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let last_ms = LAST_LOG_MS.load(Ordering::Relaxed);
    // Log every 5 seconds max, across all WsPub instances.
    if now_ms.saturating_sub(last_ms) < 5_000 {
        return;
    }

    if LAST_LOG_MS
        .compare_exchange(last_ms, now_ms, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return;
    }

    let prev_published = LAST_PUBLISHED_TOTAL.swap(published_now, Ordering::Relaxed);
    let prev_bytes = LAST_BYTES_TOTAL.swap(bytes_now, Ordering::Relaxed);

    let delta_published = published_now.saturating_sub(prev_published);
    let delta_bytes = bytes_now.saturating_sub(prev_bytes);
    let delta_ms = now_ms.saturating_sub(last_ms).max(1);

    let hz = (delta_published as f64) * 1000.0 / (delta_ms as f64);
    let kbps = (delta_bytes as f64) / (delta_ms as f64); // kB/s-ish (bytes/ms)

    info!(
        "[WS-PUB][{}] publish_rate_hz={:.2} bytes_per_ms={:.2} totals: messages={} bytes={}",
        stream, hz, kbps, published_now, bytes_now
    );
}

/// Handle a single WebSocket client connection
async fn handle_client(
    stream: TcpStream,
    peer_addr: SocketAddr,
    mut broadcast_rx: broadcast::Receiver<Vec<u8>>,
    clients: Arc<RwLock<HashMap<SocketAddr, broadcast::Sender<Vec<u8>>>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Perform WebSocket handshake with timeout protection
    let handshake_start = std::time::Instant::now();
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            warn!(
                "[WS-PUB] Client {} handshake failed after {:.2}ms: {}",
                peer_addr,
                handshake_start.elapsed().as_secs_f64() * 1000.0,
                e
            );
            return Err(Box::new(e));
        }
    };
    let handshake_duration = handshake_start.elapsed();
    if handshake_duration.as_millis() > 100 {
        warn!(
            "[WS-PUB] Client {} slow handshake: {:.2}ms",
            peer_addr,
            handshake_duration.as_secs_f64() * 1000.0
        );
    }

    let (mut write, mut read) = ws_stream.split();

    info!(
        "[WS-PUB] Client {} connected (handshake: {:.2}ms)",
        peer_addr,
        handshake_duration.as_secs_f64() * 1000.0
    );

    static LAGGED_TOTAL: AtomicU64 = AtomicU64::new(0);

    // Spawn a task to monitor the read side for connection closure
    let peer_addr_monitor = peer_addr;
    let read_task = tokio::spawn(async move {
        loop {
            match read.next().await {
                Some(Ok(msg)) => {
                    // Client sent a message (ping, pong, close, etc.)
                    match msg {
                        Message::Close(_) => {
                            debug!("[WS-PUB] Client {} sent close frame", peer_addr_monitor);
                            break;
                        }
                        Message::Ping(_data) => {
                            debug!("[WS-PUB] Client {} sent ping", peer_addr_monitor);
                            // Could respond with pong, but we're only using write side
                        }
                        Message::Pong(_) => {
                            debug!("[WS-PUB] Client {} sent pong", peer_addr_monitor);
                        }
                        _ => {
                            debug!(
                                "[WS-PUB] Client {} sent unexpected message type",
                                peer_addr_monitor
                            );
                        }
                    }
                }
                Some(Err(e)) => {
                    warn!(
                        "[WS-PUB] Client {} read error (connection may be closed): {}",
                        peer_addr_monitor, e
                    );
                    break;
                }
                None => {
                    debug!(
                        "[WS-PUB] Client {} read stream ended (connection closed)",
                        peer_addr_monitor
                    );
                    break;
                }
            }
        }
    });

    loop {
        match broadcast_rx.recv().await {
            Ok(data) => {
                let data_len = data.len();
                match write.send(Message::Binary(data)).await {
                    Ok(_) => {
                        // Successfully sent
                    }
                    Err(e) => {
                        // Log the error to understand why the connection is closing
                        warn!(
                            "[WS-PUB] Client {} send error (disconnecting): {} (message_size={} bytes)",
                            peer_addr, e, data_len
                        );
                        break;
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                // Critical real-time diagnostic: client is falling behind and the server
                // dropped `skipped` messages for this receiver.
                let n = LAGGED_TOTAL.fetch_add(1, Ordering::Relaxed) + 1;
                if n == 1 || n % 10 == 0 {
                    warn!(
                        "[WS-PUB] Client {} lagged: skipped_messages={} lag_events_total={}",
                        peer_addr, skipped, n
                    );
                }
                continue;
            }
            Err(broadcast::error::RecvError::Closed) => {
                warn!("[WS-PUB] Client {} broadcast channel closed", peer_addr);
                break;
            }
        }

        // Check if read task detected connection closure
        if read_task.is_finished() {
            debug!(
                "[WS-PUB] Client {} read task finished (connection closed)",
                peer_addr
            );
            break;
        }
    }

    // Abort the read task if we're breaking out of the loop
    read_task.abort();

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

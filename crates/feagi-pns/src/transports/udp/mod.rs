//! UDP Transport Implementation
//!
//! High-throughput, low-latency transport for visualization and sensory data.
//! Uses chunking for large payloads and optional reassembly for receivers.
//!
//! # Design
//! - **MTU-aware chunking**: Splits large payloads into UDP-sized chunks
//! - **Sequence numbering**: Each chunk has sequence number for reassembly
//! - **Best-effort delivery**: No retransmission (acceptable for viz data)
//! - **LZ4 compression**: Applied before chunking
//!
//! # Protocol
//! Each UDP packet format:
//! ```text
//! [4 bytes: message_id][4 bytes: chunk_index][4 bytes: total_chunks][payload...]
//! ```

use crate::core::{PNSError, Result, SharedFBC};
use crate::nonblocking::{compression, runtime::RuntimeHandle, NonBlockingTransport};
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

/// Maximum UDP payload size (conservative, accounts for IP/UDP headers)
const MAX_UDP_PAYLOAD: usize = 1400;

/// Header size (message_id + chunk_index + total_chunks)
const HEADER_SIZE: usize = 12;

/// Maximum data per chunk (payload - header)
const MAX_CHUNK_DATA: usize = MAX_UDP_PAYLOAD - HEADER_SIZE;

/// Timeout for incomplete message reassembly
const REASSEMBLY_TIMEOUT: Duration = Duration::from_secs(5);

/// Configuration for UDP transport
#[derive(Debug, Clone)]
pub struct UdpConfig {
    /// Local bind address for receiving
    pub bind_address: String,
    /// Remote peer address for sending
    pub peer_address: String,
    /// Enable compression (default: true)
    pub compress: bool,
    /// Maximum message size before chunking (default: 64KB)
    pub max_message_size: usize,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:5564".to_string(),
            peer_address: "127.0.0.1:5564".to_string(),
            compress: true,
            max_message_size: 65536,
        }
    }
}

/// UDP packet header
#[derive(Debug, Clone, Copy)]
struct PacketHeader {
    message_id: u32,
    chunk_index: u32,
    total_chunks: u32,
}

impl PacketHeader {
    fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.message_id.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.chunk_index.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.total_chunks.to_be_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < HEADER_SIZE {
            return None;
        }
        Some(Self {
            message_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            chunk_index: u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            total_chunks: u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        })
    }
}

/// Incomplete message being reassembled
struct IncompleteMessage {
    chunks: HashMap<u32, Vec<u8>>,
    total_chunks: u32,
    received_at: Instant,
}

/// UDP Transport
pub struct UdpTransport {
    config: UdpConfig,
    socket: Option<Arc<UdpSocket>>,
    runtime: RuntimeHandle,
    message_id_counter: Arc<RwLock<u32>>,
    /// Reassembly buffer for incoming chunked messages
    reassembly_buffer: Arc<RwLock<HashMap<u32, IncompleteMessage>>>,
}

impl UdpTransport {
    /// Create a new UDP transport
    pub fn new(config: UdpConfig, runtime: RuntimeHandle) -> Self {
        Self {
            config,
            socket: None,
            runtime,
            message_id_counter: Arc::new(RwLock::new(0)),
            reassembly_buffer: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Chunk a large payload into UDP-sized packets
    fn chunk_payload(&self, data: &[u8], message_id: u32) -> Vec<Vec<u8>> {
        let total_chunks = (data.len() + MAX_CHUNK_DATA - 1) / MAX_CHUNK_DATA;
        let mut chunks = Vec::with_capacity(total_chunks);

        for (chunk_index, chunk_data) in data.chunks(MAX_CHUNK_DATA).enumerate() {
            let header = PacketHeader {
                message_id,
                chunk_index: chunk_index as u32,
                total_chunks: total_chunks as u32,
            };

            let mut packet = Vec::with_capacity(HEADER_SIZE + chunk_data.len());
            packet.extend_from_slice(&header.to_bytes());
            packet.extend_from_slice(chunk_data);
            chunks.push(packet);
        }

        chunks
    }

    /// Send chunked data via UDP
    async fn send_chunked(&self, data: &[u8]) -> Result<()> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| PNSError::Transport("Socket not initialized".to_string()))?;

        // Get next message ID
        let message_id = {
            let mut counter = self.message_id_counter.write().await;
            *counter = counter.wrapping_add(1);
            *counter
        };

        // Optionally compress
        let data_to_send = if self.config.compress {
            compression::compress_lz4_async(data).await?
        } else {
            data.to_vec()
        };

        // Chunk the data
        let chunks = self.chunk_payload(&data_to_send, message_id);

        // Parse peer address
        let peer_addr: SocketAddr = self
            .config
            .peer_address
            .parse()
            .map_err(|e| PNSError::Config(format!("Invalid peer address: {}", e)))?;

        // Send all chunks
        for chunk in chunks {
            socket
                .send_to(&chunk, peer_addr)
                .await
                .map_err(|e| PNSError::Transport(format!("UDP send failed: {}", e)))?;
        }

        Ok(())
    }

    /// Receive and reassemble chunked messages
    async fn receive_chunked(&self) -> Result<Vec<u8>> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| PNSError::Transport("Socket not initialized".to_string()))?;

        let mut buf = vec![0u8; MAX_UDP_PAYLOAD];

        loop {
            let (len, _src_addr) = socket
                .recv_from(&mut buf)
                .await
                .map_err(|e| PNSError::Transport(format!("UDP recv failed: {}", e)))?;

            let packet = &buf[..len];

            // Parse header
            let header = PacketHeader::from_bytes(packet)
                .ok_or_else(|| PNSError::Transport("Invalid packet header".to_string()))?;

            let payload = &packet[HEADER_SIZE..];

            // Single chunk message (fast path)
            if header.total_chunks == 1 {
                return if self.config.compress {
                    compression::decompress_lz4_async(payload).await
                } else {
                    Ok(payload.to_vec())
                };
            }

            // Multi-chunk message (reassembly needed)
            let complete_message = {
                let mut buffer = self.reassembly_buffer.write().await;

                // Clean up old incomplete messages
                buffer.retain(|_, msg| msg.received_at.elapsed() < REASSEMBLY_TIMEOUT);

                // Get or create incomplete message
                let incomplete =
                    buffer
                        .entry(header.message_id)
                        .or_insert_with(|| IncompleteMessage {
                            chunks: HashMap::new(),
                            total_chunks: header.total_chunks,
                            received_at: Instant::now(),
                        });

                // Store chunk
                incomplete
                    .chunks
                    .insert(header.chunk_index, payload.to_vec());

                // Check if complete
                if incomplete.chunks.len() == header.total_chunks as usize {
                    // Reassemble in order
                    let mut reassembled = Vec::new();
                    for i in 0..header.total_chunks {
                        if let Some(chunk) = incomplete.chunks.get(&i) {
                            reassembled.extend_from_slice(chunk);
                        } else {
                            return Err(PNSError::Transport(format!(
                                "Missing chunk {} for message {}",
                                i, header.message_id
                            )));
                        }
                    }

                    // Remove from buffer
                    buffer.remove(&header.message_id);
                    Some(reassembled)
                } else {
                    None
                }
            };

            if let Some(data) = complete_message {
                return if self.config.compress {
                    compression::decompress_lz4_async(&data).await
                } else {
                    Ok(data)
                };
            }

            // Message incomplete, continue receiving
        }
    }
}

#[async_trait]
impl NonBlockingTransport for UdpTransport {
    fn backend_name(&self) -> &str {
        "udp"
    }

    async fn start(&mut self) -> Result<()> {
        // Bind socket
        let socket = UdpSocket::bind(&self.config.bind_address)
            .await
            .map_err(|e| PNSError::Transport(format!("Failed to bind UDP socket: {}", e)))?;

        println!(
            "🦀 [UDP] Bound to {} (peer: {})",
            self.config.bind_address, self.config.peer_address
        );

        self.socket = Some(Arc::new(socket));
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.socket = None;
        self.reassembly_buffer.write().await.clear();
        println!("🦀 [UDP] Transport stopped");
        Ok(())
    }

    async fn publish_visualization(&self, fbc: SharedFBC) -> Result<()> {
        let data = fbc.get_byte_ref();
        self.send_chunked(data).await
    }

    async fn publish_motor(&self, _agent_id: &str, fbc: SharedFBC) -> Result<()> {
        // Motor commands typically go to specific agents, would need routing
        let data = fbc.get_byte_ref();
        self.send_chunked(data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nonblocking::runtime as pns_runtime;
    use feagi_data_serialization::FeagiByteContainer;
    use tokio::runtime;

    #[test]
    fn test_packet_header_serialization() {
        let header = PacketHeader {
            message_id: 12345,
            chunk_index: 5,
            total_chunks: 10,
        };

        let bytes = header.to_bytes();
        let decoded = PacketHeader::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.message_id, 12345);
        assert_eq!(decoded.chunk_index, 5);
        assert_eq!(decoded.total_chunks, 10);
    }

    #[test]
    fn test_chunking() {
        let rt = pns_runtime::create_runtime().unwrap();
        let config = UdpConfig::default();
        let transport = UdpTransport::new(config, Arc::new(rt));

        // Create data larger than one chunk
        let data = vec![42u8; MAX_CHUNK_DATA * 3 + 100];
        let chunks = transport.chunk_payload(&data, 1);

        assert_eq!(chunks.len(), 4); // 3 full chunks + 1 partial

        // Verify each chunk has header
        for chunk in &chunks {
            assert!(chunk.len() >= HEADER_SIZE);
            let header = PacketHeader::from_bytes(chunk).unwrap();
            assert_eq!(header.message_id, 1);
            assert_eq!(header.total_chunks, 4);
        }
    }

    #[tokio::test]
    async fn test_udp_transport_lifecycle() {
        // Use a fake runtime handle (just for the struct, won't actually use it)
        // In real usage, runtime would be managed at PNS level
        let runtime = Arc::new(
            runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
        let config = UdpConfig {
            bind_address: "127.0.0.1:0".to_string(), // Random port
            peer_address: "127.0.0.1:5564".to_string(),
            compress: false,
            max_message_size: 1024,
        };

        let mut transport = UdpTransport::new(config, runtime.clone());

        // Start
        transport.start().await.unwrap();
        assert!(transport.socket.is_some());

        // Stop
        transport.stop().await.unwrap();
        assert!(transport.socket.is_none());

        // Leak runtime to avoid drop in async context (test only)
        std::mem::forget(runtime);
    }

    #[tokio::test]
    async fn test_send_small_message() {
        let runtime = Arc::new(
            runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
        let config = UdpConfig {
            bind_address: "127.0.0.1:0".to_string(),
            peer_address: "127.0.0.1:9999".to_string(), // Dummy peer
            compress: false,
            max_message_size: 1024,
        };

        let mut transport = UdpTransport::new(config, runtime.clone());
        transport.start().await.unwrap();

        // Create small FBC
        let fbc = Arc::new(FeagiByteContainer::new_empty());

        // Should not panic (even if no receiver)
        let result = transport.publish_visualization(fbc).await;
        // May succeed or fail depending on network, but shouldn't panic
        let _ = result;

        // Leak runtime to avoid drop in async context (test only)
        std::mem::forget(runtime);
    }
}

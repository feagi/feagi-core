// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ DEALER pattern (client-side request-reply)
//!
//! DEALER sockets are used for asynchronous request-reply patterns where the client
//! can send multiple requests without waiting for replies. Used with ROUTER servers.

use crate::transports::core::common::{ClientConfig, TransportConfig, TransportError, TransportResult};
use crate::transports::core::traits::{RequestReplyClient, Transport};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tracing::info;
use zeromq::{DealerSocket, Socket, SocketRecv, SocketSend, ZmqMessage};
/// ZMQ DEALER socket implementation (client-side)
pub struct ZmqDealer {
    runtime: Arc<Runtime>,
    config: ClientConfig,
    socket: Arc<Mutex<Option<DealerSocket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqDealer {
    /// Create a new DEALER socket
    pub fn new(runtime: Arc<Runtime>, config: ClientConfig) -> TransportResult<Self> {
        config.base.validate()?;

        Ok(Self {
            runtime,
            config,
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Create with default context
    pub fn with_address(address: impl Into<String>) -> TransportResult<Self> {
        let config = ClientConfig::new(address);
        let runtime = Arc::new(
            Runtime::new()
                .map_err(|e| TransportError::InitializationFailed(e.to_string()))?,
        );
        Self::new(runtime, config)
    }

    fn ensure_supported_options(&self) -> TransportResult<()> {
        let defaults = TransportConfig::default();
        if self.config.base.send_hwm != defaults.send_hwm
            || self.config.base.recv_hwm != defaults.recv_hwm
            || self.config.base.linger != defaults.linger
        {
            return Err(TransportError::InvalidConfig(format!(
                "zeromq transport does not support custom socket options (send_hwm={}, recv_hwm={}, linger={:?})",
                self.config.base.send_hwm,
                self.config.base.recv_hwm,
                self.config.base.linger
            )));
        }
        Ok(())
    }
}

impl Transport for ZmqDealer {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }

        self.ensure_supported_options()?;

        // Create DEALER socket
        let mut socket = DealerSocket::new();

        // Connect socket
        self.runtime
            .block_on(socket.connect(&self.config.base.address))
            .map_err(|e| TransportError::ConnectFailed(e.to_string()))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!("🦀 [ZMQ-DEALER] Connected to {}", self.config.base.address);

        Ok(())
    }

    fn stop(&mut self) -> TransportResult<()> {
        *self.running.lock() = false;
        *self.socket.lock() = None;
        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.lock()
    }

    fn transport_type(&self) -> &str {
        "zmq-dealer"
    }
}

impl RequestReplyClient for ZmqDealer {
    fn request(&self, data: &[u8]) -> TransportResult<Vec<u8>> {
        self.request_timeout(data, 0) // 0 = blocking
    }

    fn request_timeout(&self, data: &[u8], timeout_ms: u64) -> TransportResult<Vec<u8>> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_mut().ok_or(TransportError::NotRunning)?;

        // Check message size
        if let Some(max_size) = self.config.base.max_message_size {
            if data.len() > max_size {
                return Err(TransportError::MessageTooLarge {
                    size: data.len(),
                    max_size,
                });
            }
        }

        // Send request: [delimiter, request_data]
        let mut message = ZmqMessage::from(data.to_vec());
        message.prepend(&ZmqMessage::from(Vec::new()));
        self.runtime
            .block_on(sock.send(message))
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        // Receive reply
        let recv_future = sock.recv();
        let reply = if timeout_ms == 0 {
            self.runtime
                .block_on(recv_future)
                .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?
        } else {
            self.runtime
                .block_on(timeout(
                    std::time::Duration::from_millis(timeout_ms),
                    recv_future,
                ))
                .map_err(|_| TransportError::Timeout)?
                .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?
        };

        let mut frames = reply.into_vec();
        if frames.first().map(|frame| frame.is_empty()).unwrap_or(false) {
            frames.remove(0);
        }
        if frames.len() != 1 {
            return Err(TransportError::InvalidMessage(format!(
                "Expected single reply frame, got {}",
                frames.len()
            )));
        }

        Ok(frames.remove(0).to_vec())
    }

    fn send(&self, data: &[u8]) -> TransportResult<()> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_mut().ok_or(TransportError::NotRunning)?;

        // Send without waiting for reply
        let mut message = ZmqMessage::from(data.to_vec());
        message.prepend(&ZmqMessage::from(Vec::new()));
        self.runtime
            .block_on(sock.send(message))
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dealer_creation() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let config = ClientConfig::new("tcp://127.0.0.1:30000");
        let dealer = ZmqDealer::new(runtime, config);
        assert!(dealer.is_ok());
    }

    #[test]
    fn test_dealer_start_stop() {
        let mut dealer = ZmqDealer::with_address("tcp://127.0.0.1:30001").unwrap();
        assert!(!dealer.is_running());

        dealer.start().unwrap();
        assert!(dealer.is_running());

        dealer.stop().unwrap();
        assert!(!dealer.is_running());
    }
}

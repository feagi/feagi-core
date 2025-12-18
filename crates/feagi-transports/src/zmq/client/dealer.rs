// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ DEALER pattern (client-side request-reply)
//!
//! DEALER sockets are used for asynchronous request-reply patterns where the client
//! can send multiple requests without waiting for replies. Used with ROUTER servers.

use crate::common::{ClientConfig, TransportError, TransportResult};
use crate::traits::{RequestReplyClient, Transport};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::info;
/// ZMQ DEALER socket implementation (client-side)
pub struct ZmqDealer {
    context: Arc<zmq::Context>,
    config: ClientConfig,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqDealer {
    /// Create a new DEALER socket
    pub fn new(context: Arc<zmq::Context>, config: ClientConfig) -> TransportResult<Self> {
        config.base.validate()?;

        Ok(Self {
            context,
            config,
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Create with default context
    pub fn with_address(address: impl Into<String>) -> TransportResult<Self> {
        let context = Arc::new(zmq::Context::new());
        let config = ClientConfig::new(address);
        Self::new(context, config)
    }
}

impl Transport for ZmqDealer {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }

        // Create DEALER socket
        let socket = self.context.socket(zmq::DEALER)?;

        // Set socket options
        if let Some(linger) = self.config.base.linger {
            socket.set_linger(linger.as_millis() as i32)?;
        } else {
            socket.set_linger(0)?;
        }

        socket.set_rcvhwm(self.config.base.recv_hwm as i32)?;
        socket.set_sndhwm(self.config.base.send_hwm as i32)?;

        // Connect socket
        socket
            .connect(&self.config.base.address)
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
        let sock_guard = self.socket.lock();
        let sock = sock_guard.as_ref().ok_or(TransportError::NotRunning)?;

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
        sock.send(&Vec::<u8>::new(), zmq::SNDMORE)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        sock.send(data, 0)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        // Receive reply
        if timeout_ms > 0 {
            let poll_items = &mut [sock.as_poll_item(zmq::POLLIN)];
            zmq::poll(poll_items, timeout_ms as i64)?;

            if !poll_items[0].is_readable() {
                return Err(TransportError::Timeout);
            }
        }

        // Receive multipart reply: [delimiter, response_data]
        let mut msg_parts = Vec::new();
        let mut more = true;

        while more {
            let mut msg = zmq::Message::new();
            sock.recv(&mut msg, 0)?;
            msg_parts.push(msg.to_vec());
            more = sock.get_rcvmore()?;
        }

        if msg_parts.len() < 2 {
            return Err(TransportError::InvalidMessage(format!(
                "Expected at least 2 parts, got {}",
                msg_parts.len()
            )));
        }

        Ok(msg_parts[1].clone())
    }

    fn send(&self, data: &[u8]) -> TransportResult<()> {
        let sock_guard = self.socket.lock();
        let sock = sock_guard.as_ref().ok_or(TransportError::NotRunning)?;

        // Send without waiting for reply
        sock.send(&Vec::<u8>::new(), zmq::SNDMORE)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        sock.send(data, 0)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dealer_creation() {
        let context = Arc::new(zmq::Context::new());
        let config = ClientConfig::new("tcp://127.0.0.1:30000");
        let dealer = ZmqDealer::new(context, config);
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

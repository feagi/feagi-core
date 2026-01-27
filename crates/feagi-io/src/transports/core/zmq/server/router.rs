// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ ROUTER pattern (server-side request-reply)
//!
//! ROUTER sockets are used for asynchronous request-reply patterns where the server
//! can handle multiple clients concurrently. Each client is identified by a unique
//! identity frame, allowing the server to route replies back to the correct client.

use crate::transports::core::common::{
    ReplyHandle, ServerConfig, TransportConfig, TransportError, TransportResult,
};
use crate::transports::core::traits::{RequestReplyServer, Transport};
use futures_util::FutureExt;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tracing::info;
use zeromq::{RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage};
/// ZMQ ROUTER socket implementation (server-side)
pub struct ZmqRouter {
    runtime: Arc<Runtime>,
    config: ServerConfig,
    socket: Arc<Mutex<Option<RouterSocket>>>,
    running: Arc<Mutex<bool>>,
    pending_request: Arc<Mutex<Option<(Vec<u8>, Vec<u8>)>>>,
}

impl ZmqRouter {
    /// Create a new ROUTER socket
    pub fn new(runtime: Arc<Runtime>, config: ServerConfig) -> TransportResult<Self> {
        config.base.validate()?;

        Ok(Self {
            runtime,
            config,
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            pending_request: Arc::new(Mutex::new(None)),
        })
    }

    /// Create with default context
    pub fn with_address(address: impl Into<String>) -> TransportResult<Self> {
        let config = ServerConfig::new(address);
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

impl Transport for ZmqRouter {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }

        self.ensure_supported_options()?;

        // Create ROUTER socket
        let mut socket = RouterSocket::new();

        // Bind socket
        self.runtime
            .block_on(socket.bind(&self.config.base.address))
            .map_err(|e| TransportError::BindFailed(e.to_string()))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!("🦀 [ZMQ-ROUTER] Listening on {}", self.config.base.address);

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
        "zmq-router"
    }
}

impl RequestReplyServer for ZmqRouter {
    fn receive(&self) -> TransportResult<(Vec<u8>, Box<dyn ReplyHandle>)> {
        self.receive_timeout(0) // 0 = blocking
    }

    fn receive_timeout(&self, timeout_ms: u64) -> TransportResult<(Vec<u8>, Box<dyn ReplyHandle>)> {
        if let Some((request_data, identity)) = self.pending_request.lock().take() {
            let reply = ZmqRouterReplyHandle {
                socket: Arc::clone(&self.socket),
                identity,
                runtime: Arc::clone(&self.runtime),
            };
            return Ok((request_data, Box::new(reply)));
        }

        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_mut().ok_or(TransportError::NotRunning)?;

        let recv_future = sock.recv();
        let message = if timeout_ms == 0 {
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

        let mut frames = message.into_vec();
        if frames.is_empty() {
            return Err(TransportError::InvalidMessage(
                "Empty router message".to_string(),
            ));
        }

        let identity = frames.remove(0).to_vec();
        if frames.first().map(|frame| frame.is_empty()).unwrap_or(false) {
            frames.remove(0);
        }

        if frames.len() != 1 {
            return Err(TransportError::InvalidMessage(format!(
                "Expected single-frame payload, got {} frames",
                frames.len()
            )));
        }

        let request_data = frames.remove(0).to_vec();

        // Create reply handle
        let reply = ZmqRouterReplyHandle {
            socket: Arc::clone(&self.socket),
            identity,
            runtime: Arc::clone(&self.runtime),
        };

        Ok((request_data, Box::new(reply)))
    }

    fn poll(&self, timeout_ms: u64) -> TransportResult<bool> {
        if self.pending_request.lock().is_some() {
            return Ok(true);
        }

        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_mut().ok_or(TransportError::NotRunning)?;

        let recv_future = sock.recv();
        let message = if timeout_ms == 0 {
            match self.runtime.block_on(async { recv_future.now_or_never() }) {
                None => return Ok(false),
                Some(result) => result.map_err(|e| TransportError::ReceiveFailed(e.to_string()))?,
            }
        } else {
            self.runtime
                .block_on(timeout(
                    std::time::Duration::from_millis(timeout_ms),
                    recv_future,
                ))
                .map_err(|_| TransportError::Timeout)?
                .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?
        };

        let mut frames = message.into_vec();
        if frames.is_empty() {
            return Err(TransportError::InvalidMessage(
                "Empty router message".to_string(),
            ));
        }

        let identity = frames.remove(0).to_vec();
        if frames.first().map(|frame| frame.is_empty()).unwrap_or(false) {
            frames.remove(0);
        }
        if frames.len() != 1 {
            return Err(TransportError::InvalidMessage(
                "Unexpected multipart request payload".to_string(),
            ));
        }

        let request_data = frames.remove(0).to_vec();
        *self.pending_request.lock() = Some((request_data, identity));
        Ok(true)
    }
}

/// Reply handle for ROUTER socket
struct ZmqRouterReplyHandle {
    socket: Arc<Mutex<Option<RouterSocket>>>,
    identity: Vec<u8>,
    runtime: Arc<Runtime>,
}

impl ReplyHandle for ZmqRouterReplyHandle {
    fn send(&self, data: &[u8]) -> TransportResult<()> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_mut().ok_or(TransportError::NotRunning)?;

        let mut message = ZmqMessage::from(data.to_vec());
        message.prepend(&ZmqMessage::from(Vec::new()));
        message.prepend(&ZmqMessage::from(self.identity.clone()));

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
    fn test_router_creation() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let config = ServerConfig::new("tcp://127.0.0.1:30000");
        let router = ZmqRouter::new(runtime, config);
        assert!(router.is_ok());
    }

    #[test]
    #[ignore] // Flaky test: port conflicts when running in parallel (Address already in use)
    fn test_router_start_stop() {
        let mut router = ZmqRouter::with_address("tcp://127.0.0.1:30001").unwrap();
        assert!(!router.is_running());

        router.start().unwrap();
        assert!(router.is_running());

        router.stop().unwrap();
        assert!(!router.is_running());
    }
}

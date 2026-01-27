// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ PULL pattern (server-side push-pull)
//!
//! PULL sockets are used for receiving data from multiple PUSH clients.
//! Messages are load-balanced across connected clients.

use crate::transports::core::common::{ServerConfig, TransportConfig, TransportError, TransportResult};
use crate::transports::core::traits::{Pull, Transport};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tracing::info;
use zeromq::{PullSocket, Socket, SocketRecv};
/// ZMQ PULL socket implementation (receiver)
pub struct ZmqPull {
    runtime: Arc<Runtime>,
    config: ServerConfig,
    socket: Arc<Mutex<Option<PullSocket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqPull {
    /// Create a new PULL socket
    pub fn new(runtime: Arc<Runtime>, config: ServerConfig) -> TransportResult<Self> {
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

impl Transport for ZmqPull {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }

        self.ensure_supported_options()?;

        // Create PULL socket
        let mut socket = PullSocket::new();

        // Bind socket
        self.runtime
            .block_on(socket.bind(&self.config.base.address))
            .map_err(|e| TransportError::BindFailed(e.to_string()))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!("🦀 [ZMQ-PULL] Listening on {}", self.config.base.address);

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
        "zmq-pull"
    }
}

impl Pull for ZmqPull {
    fn pull(&self) -> TransportResult<Vec<u8>> {
        self.pull_timeout(0) // 0 = blocking
    }

    fn pull_timeout(&self, timeout_ms: u64) -> TransportResult<Vec<u8>> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_ref().ok_or(TransportError::NotRunning)?;

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
        if frames.len() != 1 {
            return Err(TransportError::InvalidMessage(
                "Unexpected multipart pull payload".to_string(),
            ));
        }
        Ok(frames.remove(0).to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_creation() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let config = ServerConfig::new("tcp://127.0.0.1:30020");
        let pull = ZmqPull::new(runtime, config);
        assert!(pull.is_ok());
    }

    #[test]
    fn test_pull_start_stop() {
        let mut pull = ZmqPull::with_address("tcp://127.0.0.1:30021").unwrap();
        assert!(!pull.is_running());

        pull.start().unwrap();
        assert!(pull.is_running());

        pull.stop().unwrap();
        assert!(!pull.is_running());
    }
}

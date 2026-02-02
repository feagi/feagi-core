// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ PUSH pattern (client-side push-pull)
//!
//! PUSH sockets are used for distributing data to PULL servers.
//! Messages are load-balanced across connected servers.

use crate::transports::core::common::{
    ClientConfig, TransportConfig, TransportError, TransportResult,
};
use crate::transports::core::traits::{Push, Transport};
use parking_lot::Mutex;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::runtime::Runtime;
use tokio::task::block_in_place;
use tokio::time::timeout;
use tracing::info;
use zeromq::{PushSocket, Socket, SocketSend, ZmqMessage};

fn block_on_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
    if Handle::try_current().is_ok() {
        block_in_place(|| Handle::current().block_on(future))
    } else {
        runtime.block_on(future)
    }
}
/// ZMQ PUSH socket implementation (sender)
pub struct ZmqPush {
    runtime: Arc<Runtime>,
    config: ClientConfig,
    socket: Arc<Mutex<Option<PushSocket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqPush {
    /// Create a new PUSH socket
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
            Runtime::new().map_err(|e| TransportError::InitializationFailed(e.to_string()))?,
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

impl Transport for ZmqPush {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }

        self.ensure_supported_options()?;

        // Create PUSH socket
        let mut socket = PushSocket::new();

        // Connect socket
        block_on_runtime(
            self.runtime.as_ref(),
            socket.connect(&self.config.base.address),
        )
        .map_err(|e| TransportError::ConnectFailed(e.to_string()))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!("🦀 [ZMQ-PUSH] Connected to {}", self.config.base.address);

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
        "zmq-push"
    }
}

impl Push for ZmqPush {
    fn push(&self, data: &[u8]) -> TransportResult<()> {
        self.push_timeout(data, 0) // 0 = blocking
    }

    fn push_timeout(&self, data: &[u8], timeout_ms: u64) -> TransportResult<()> {
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

        // Send message
        let message = ZmqMessage::from(data.to_vec());
        if timeout_ms > 0 {
            block_on_runtime(self.runtime.as_ref(), async {
                timeout(
                    std::time::Duration::from_millis(timeout_ms),
                    sock.send(message),
                )
                .await
            })
            .map_err(|_| TransportError::Timeout)?
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        } else {
            block_on_runtime(self.runtime.as_ref(), sock.send(message))
                .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_creation() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let config = ClientConfig::new("tcp://127.0.0.1:30020");
        let push = ZmqPush::new(runtime, config);
        assert!(push.is_ok());
    }

    #[test]
    fn test_push_start_stop() {
        let mut push = ZmqPush::with_address("tcp://127.0.0.1:30021").unwrap();
        assert!(!push.is_running());

        push.start().unwrap();
        assert!(push.is_running());

        push.stop().unwrap();
        assert!(!push.is_running());
    }
}

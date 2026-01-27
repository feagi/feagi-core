// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ PUB pattern (server-side publish-subscribe)
//!
//! PUB sockets are used for one-to-many distribution where the publisher
//! broadcasts messages to all connected subscribers. Subscribers can filter
//! messages by topic.

use crate::transports::core::common::{ServerConfig, TransportConfig, TransportError, TransportResult};
use crate::transports::core::traits::{Publisher, Transport};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tracing::info;
use zeromq::{PubSocket, Socket, SocketSend, ZmqMessage};
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use std::future::Future;

fn block_on_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
    if Handle::try_current().is_ok() {
        block_in_place(|| Handle::current().block_on(future))
    } else {
        runtime.block_on(future)
    }
}
/// ZMQ PUB socket implementation (publisher)
pub struct ZmqPub {
    runtime: Arc<Runtime>,
    config: ServerConfig,
    socket: Arc<Mutex<Option<PubSocket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqPub {
    /// Create a new PUB socket
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

impl Transport for ZmqPub {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }

        self.ensure_supported_options()?;

        // Create PUB socket
        let mut socket = PubSocket::new();

        // Bind socket
        block_on_runtime(self.runtime.as_ref(), socket.bind(&self.config.base.address))
            .map_err(|e| TransportError::BindFailed(e.to_string()))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!(
            "🦀 [ZMQ-PUB] Listening on {}",
            self.config.base.address
        );

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
        "zmq-pub"
    }
}

impl Publisher for ZmqPub {
    fn publish(&self, topic: &[u8], data: &[u8]) -> TransportResult<()> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard
            .as_ref()
            .ok_or(TransportError::NotRunning)?;

        // Check message size
        if let Some(max_size) = self.config.base.max_message_size {
            if data.len() > max_size {
                return Err(TransportError::MessageTooLarge {
                    size: data.len(),
                    max_size,
                });
            }
        }

        // Send multipart message: [topic, data]
        let mut message = ZmqMessage::from(data.to_vec());
        message.prepend(&ZmqMessage::from(topic.to_vec()));
        if let Some(timeout_duration) = self.config.base.timeout {
            block_on_runtime(self.runtime.as_ref(), async {
                timeout(timeout_duration, sock.send(message)).await
            })
                .map_err(|_| TransportError::Timeout)?
                .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        } else {
            block_on_runtime(self.runtime.as_ref(), sock.send(message))
                .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        }

        Ok(())
    }

    fn publish_simple(&self, data: &[u8]) -> TransportResult<()> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard
            .as_ref()
            .ok_or(TransportError::NotRunning)?;

        // Check message size
        if let Some(max_size) = self.config.base.max_message_size {
            if data.len() > max_size {
                return Err(TransportError::MessageTooLarge {
                    size: data.len(),
                    max_size,
                });
            }
        }

        let message = ZmqMessage::from(data.to_vec());
        if let Some(timeout_duration) = self.config.base.timeout {
            block_on_runtime(self.runtime.as_ref(), async {
                timeout(timeout_duration, sock.send(message)).await
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
    fn test_pub_creation() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let config = ServerConfig::new("tcp://127.0.0.1:30010");
        let pub_socket = ZmqPub::new(runtime, config);
        assert!(pub_socket.is_ok());
    }

    #[test]
    fn test_pub_start_stop() {
        let mut pub_socket = ZmqPub::with_address("tcp://127.0.0.1:30011").unwrap();
        assert!(!pub_socket.is_running());

        pub_socket.start().unwrap();
        assert!(pub_socket.is_running());

        pub_socket.stop().unwrap();
        assert!(!pub_socket.is_running());
    }
}




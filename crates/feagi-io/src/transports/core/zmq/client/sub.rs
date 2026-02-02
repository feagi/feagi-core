// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ SUB pattern (client-side publish-subscribe)
//!
//! SUB sockets are used for receiving broadcast messages from PUB servers.
//! Subscribers can filter messages by topic.

use crate::transports::core::common::{
    ClientConfig, TransportConfig, TransportError, TransportResult,
};
use crate::transports::core::traits::{Subscriber, Transport};
use parking_lot::Mutex;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::runtime::Runtime;
use tokio::task::block_in_place;
use tokio::time::timeout;
use tracing::info;
use zeromq::{Socket, SocketRecv, SubSocket};

fn block_on_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
    if Handle::try_current().is_ok() {
        block_in_place(|| Handle::current().block_on(future))
    } else {
        runtime.block_on(future)
    }
}
/// ZMQ SUB socket implementation (subscriber)
pub struct ZmqSub {
    runtime: Arc<Runtime>,
    config: ClientConfig,
    socket: Arc<Mutex<Option<SubSocket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqSub {
    /// Create a new SUB socket
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

impl Transport for ZmqSub {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }

        self.ensure_supported_options()?;

        // Create SUB socket
        let mut socket = SubSocket::new();

        // Connect socket (respect configured timeout)
        let connect_future = socket.connect(&self.config.base.address);
        if let Some(timeout_duration) = self.config.base.timeout {
            block_on_runtime(self.runtime.as_ref(), async {
                timeout(timeout_duration, connect_future).await
            })
            .map_err(|_| TransportError::Timeout)?
            .map_err(|e| TransportError::ConnectFailed(e.to_string()))?;
        } else {
            block_on_runtime(self.runtime.as_ref(), connect_future)
                .map_err(|e| TransportError::ConnectFailed(e.to_string()))?;
        }

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!("🦀 [ZMQ-SUB] Connected to {}", self.config.base.address);

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
        "zmq-sub"
    }
}

impl Subscriber for ZmqSub {
    fn subscribe(&mut self, topic: &[u8]) -> TransportResult<()> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_mut().ok_or(TransportError::NotRunning)?;
        let topic = std::str::from_utf8(topic)
            .map_err(|e| TransportError::InvalidMessage(e.to_string()))?;
        block_on_runtime(self.runtime.as_ref(), sock.subscribe(topic))
            .map_err(|e| TransportError::Other(e.to_string()))?;

        Ok(())
    }

    fn unsubscribe(&mut self, topic: &[u8]) -> TransportResult<()> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_mut().ok_or(TransportError::NotRunning)?;
        let topic = std::str::from_utf8(topic)
            .map_err(|e| TransportError::InvalidMessage(e.to_string()))?;
        block_on_runtime(self.runtime.as_ref(), sock.unsubscribe(topic))
            .map_err(|e| TransportError::Other(e.to_string()))?;

        Ok(())
    }

    fn receive(&self) -> TransportResult<(Vec<u8>, Vec<u8>)> {
        self.receive_timeout(0) // 0 = blocking
    }

    fn receive_timeout(&self, timeout_ms: u64) -> TransportResult<(Vec<u8>, Vec<u8>)> {
        let mut sock_guard = self.socket.lock();
        let sock = sock_guard.as_mut().ok_or(TransportError::NotRunning)?;

        let recv_future = sock.recv();
        let message = if timeout_ms == 0 {
            block_on_runtime(self.runtime.as_ref(), recv_future)
                .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?
        } else {
            block_on_runtime(self.runtime.as_ref(), async {
                timeout(std::time::Duration::from_millis(timeout_ms), recv_future).await
            })
            .map_err(|_| TransportError::Timeout)?
            .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?
        };

        let mut frames = message.into_vec();
        if frames.len() == 1 {
            return Ok((Vec::new(), frames.remove(0).to_vec()));
        }
        if frames.len() == 2 {
            return Ok((frames.remove(0).to_vec(), frames.remove(0).to_vec()));
        }

        Err(TransportError::InvalidMessage(
            "Unexpected multipart subscription payload".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transports::core::zmq::server::pub_socket::ZmqPub;
    use std::net::TcpListener;

    fn reserve_tcp_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
        let port = listener
            .local_addr()
            .expect("Failed to read local address")
            .port();
        drop(listener);
        port
    }

    #[test]
    fn test_sub_creation() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let config = ClientConfig::new("tcp://127.0.0.1:30010");
        let sub = ZmqSub::new(runtime, config);
        assert!(sub.is_ok());
    }

    #[test]
    fn test_sub_start_stop() {
        let port = reserve_tcp_port();
        let endpoint = format!("tcp://127.0.0.1:{port}");
        let mut pub_socket = ZmqPub::with_address(endpoint.clone()).unwrap();
        pub_socket.start().unwrap();

        let mut sub = ZmqSub::with_address(endpoint).unwrap();
        assert!(!sub.is_running());

        sub.start().unwrap();
        assert!(sub.is_running());

        sub.stop().unwrap();
        assert!(!sub.is_running());

        pub_socket.stop().unwrap();
        assert!(!pub_socket.is_running());
    }
}

// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ PUSH pattern (client-side push-pull)
//!
//! PUSH sockets are used for distributing data to PULL servers.
//! Messages are load-balanced across connected servers.

use crate::common::{ClientConfig, TransportError, TransportResult};
use crate::traits::{Push, Transport};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::info;
/// ZMQ PUSH socket implementation (sender)
pub struct ZmqPush {
    context: Arc<zmq::Context>,
    config: ClientConfig,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqPush {
    /// Create a new PUSH socket
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

impl Transport for ZmqPush {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }
        
        // Create PUSH socket
        let socket = self.context.socket(zmq::PUSH)?;
        
        // Set socket options
        socket.set_linger(0)?;
        socket.set_sndhwm(self.config.base.send_hwm as i32)?;
        socket.set_immediate(false)?;
        
        // Connect socket
        socket
            .connect(&self.config.base.address)
            .map_err(|e| TransportError::ConnectFailed(e.to_string()))?;
        
        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;
        
        info!(
            "🦀 [ZMQ-PUSH] Connected to {}",
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
        "zmq-push"
    }
}

impl Push for ZmqPush {
    fn push(&self, data: &[u8]) -> TransportResult<()> {
        self.push_timeout(data, 0) // 0 = blocking
    }
    
    fn push_timeout(&self, data: &[u8], timeout_ms: u64) -> TransportResult<()> {
        let sock_guard = self.socket.lock();
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
        
        // Set send timeout if specified
        if timeout_ms > 0 {
            sock.set_sndtimeo(timeout_ms as i32)?;
        }
        
        // Send message
        sock.send(data, 0)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_push_creation() {
        let context = Arc::new(zmq::Context::new());
        let config = ClientConfig::new("tcp://127.0.0.1:30020");
        let push = ZmqPush::new(context, config);
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




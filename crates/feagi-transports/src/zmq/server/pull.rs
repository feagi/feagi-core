//! ZMQ PULL pattern (server-side push-pull)
//!
//! PULL sockets are used for receiving data from multiple PUSH clients.
//! Messages are load-balanced across connected clients.

use crate::common::{ServerConfig, TransportError, TransportResult};
use crate::traits::{Pull, Transport};
use parking_lot::Mutex;
use std::sync::Arc;

/// ZMQ PULL socket implementation (receiver)
pub struct ZmqPull {
    context: Arc<zmq::Context>,
    config: ServerConfig,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqPull {
    /// Create a new PULL socket
    pub fn new(context: Arc<zmq::Context>, config: ServerConfig) -> TransportResult<Self> {
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
        let config = ServerConfig::new(address);
        Self::new(context, config)
    }
}

impl Transport for ZmqPull {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }
        
        // Create PULL socket
        let socket = self.context.socket(zmq::PULL)?;
        
        // Set socket options
        socket.set_linger(0)?;
        socket.set_rcvhwm(self.config.base.recv_hwm as i32)?;
        socket.set_immediate(false)?;
        
        // Bind socket
        socket
            .bind(&self.config.base.address)
            .map_err(|e| TransportError::BindFailed(e.to_string()))?;
        
        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;
        
        println!(
            "🦀 [ZMQ-PULL] Listening on {}",
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
        "zmq-pull"
    }
}

impl Pull for ZmqPull {
    fn pull(&self) -> TransportResult<Vec<u8>> {
        self.pull_timeout(0) // 0 = blocking
    }
    
    fn pull_timeout(&self, timeout_ms: u64) -> TransportResult<Vec<u8>> {
        let sock_guard = self.socket.lock();
        let sock = sock_guard
            .as_ref()
            .ok_or(TransportError::NotRunning)?;
        
        // Poll for messages if timeout specified
        if timeout_ms > 0 {
            let poll_items = &mut [sock.as_poll_item(zmq::POLLIN)];
            zmq::poll(poll_items, timeout_ms as i64)?;
            
            if !poll_items[0].is_readable() {
                return Err(TransportError::Timeout);
            }
        }
        
        // Receive message
        let mut msg = zmq::Message::new();
        sock.recv(&mut msg, 0)
            .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?;
        
        Ok(msg.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pull_creation() {
        let context = Arc::new(zmq::Context::new());
        let config = ServerConfig::new("tcp://127.0.0.1:30020");
        let pull = ZmqPull::new(context, config);
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



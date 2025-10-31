//! ZMQ SUB pattern (client-side publish-subscribe)
//!
//! SUB sockets are used for receiving broadcast messages from PUB servers.
//! Subscribers can filter messages by topic.

use crate::common::{ClientConfig, TransportError, TransportResult};
use crate::traits::{Subscriber, Transport};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::info;
/// ZMQ SUB socket implementation (subscriber)
pub struct ZmqSub {
    context: Arc<zmq::Context>,
    config: ClientConfig,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqSub {
    /// Create a new SUB socket
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

impl Transport for ZmqSub {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }
        
        // Create SUB socket
        let socket = self.context.socket(zmq::SUB)?;
        
        // Set socket options
        socket.set_linger(0)?;
        socket.set_rcvhwm(self.config.base.recv_hwm as i32)?;
        socket.set_conflate(false)?; // Keep all messages
        
        // Connect socket
        socket
            .connect(&self.config.base.address)
            .map_err(|e| TransportError::ConnectFailed(e.to_string()))?;
        
        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;
        
        info!(
            "🦀 [ZMQ-SUB] Connected to {}",
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
        "zmq-sub"
    }
}

impl Subscriber for ZmqSub {
    fn subscribe(&mut self, topic: &[u8]) -> TransportResult<()> {
        let sock_guard = self.socket.lock();
        let sock = sock_guard
            .as_ref()
            .ok_or(TransportError::NotRunning)?;
        
        sock.set_subscribe(topic)?;
        
        Ok(())
    }
    
    fn unsubscribe(&mut self, topic: &[u8]) -> TransportResult<()> {
        let sock_guard = self.socket.lock();
        let sock = sock_guard
            .as_ref()
            .ok_or(TransportError::NotRunning)?;
        
        sock.set_unsubscribe(topic)?;
        
        Ok(())
    }
    
    fn receive(&self) -> TransportResult<(Vec<u8>, Vec<u8>)> {
        self.receive_timeout(0) // 0 = blocking
    }
    
    fn receive_timeout(&self, timeout_ms: u64) -> TransportResult<(Vec<u8>, Vec<u8>)> {
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
        
        // Receive multipart message: [topic, data]
        let mut topic_msg = zmq::Message::new();
        sock.recv(&mut topic_msg, 0)
            .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?;
        
        let has_more = sock.get_rcvmore()?;
        
        if !has_more {
            // Single-part message (no topic separator)
            return Ok((Vec::new(), topic_msg.to_vec()));
        }
        
        let mut data_msg = zmq::Message::new();
        sock.recv(&mut data_msg, 0)
            .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?;
        
        Ok((topic_msg.to_vec(), data_msg.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sub_creation() {
        let context = Arc::new(zmq::Context::new());
        let config = ClientConfig::new("tcp://127.0.0.1:30010");
        let sub = ZmqSub::new(context, config);
        assert!(sub.is_ok());
    }
    
    #[test]
    fn test_sub_start_stop() {
        let mut sub = ZmqSub::with_address("tcp://127.0.0.1:30011").unwrap();
        assert!(!sub.is_running());
        
        sub.start().unwrap();
        assert!(sub.is_running());
        
        sub.stop().unwrap();
        assert!(!sub.is_running());
    }
}




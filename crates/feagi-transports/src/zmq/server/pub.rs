//! ZMQ PUB pattern (server-side publish-subscribe)
//!
//! PUB sockets are used for one-to-many distribution where the publisher
//! broadcasts messages to all connected subscribers. Subscribers can filter
//! messages by topic.

use crate::common::{ServerConfig, TransportError, TransportResult};
use crate::traits::{Publisher, Transport};
use parking_lot::Mutex;
use std::sync::Arc;

/// ZMQ PUB socket implementation (publisher)
pub struct ZmqPub {
    context: Arc<zmq::Context>,
    config: ServerConfig,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqPub {
    /// Create a new PUB socket
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

impl Transport for ZmqPub {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }
        
        // Create PUB socket
        let socket = self.context.socket(zmq::PUB)?;
        
        // Set socket options
        socket.set_linger(0)?; // No linger for real-time data
        socket.set_sndhwm(self.config.base.send_hwm as i32)?;
        socket.set_conflate(false)?; // Keep all messages (not just latest)
        
        if let Some(timeout) = self.config.base.timeout {
            socket.set_sndtimeo(timeout.as_millis() as i32)?;
        } else {
            socket.set_sndtimeo(-1)?; // Blocking
        }
        
        // Bind socket
        socket
            .bind(&self.config.base.address)
            .map_err(|e| TransportError::BindFailed(e.to_string()))?;
        
        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;
        
        println!(
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
        
        // Send multipart message: [topic, data]
        sock.send(topic, zmq::SNDMORE)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        sock.send(data, 0)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        
        Ok(())
    }
    
    fn publish_simple(&self, data: &[u8]) -> TransportResult<()> {
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
        
        sock.send(data, 0)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pub_creation() {
        let context = Arc::new(zmq::Context::new());
        let config = ServerConfig::new("tcp://127.0.0.1:30010");
        let pub_socket = ZmqPub::new(context, config);
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



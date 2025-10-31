//! ZMQ ROUTER pattern (server-side request-reply)
//!
//! ROUTER sockets are used for asynchronous request-reply patterns where the server
//! can handle multiple clients concurrently. Each client is identified by a unique
//! identity frame, allowing the server to route replies back to the correct client.

use crate::common::{ReplyHandle, ServerConfig, TransportError, TransportResult};
use crate::traits::{RequestReplyServer, Transport};
use parking_lot::Mutex;
use std::sync::Arc;

/// ZMQ ROUTER socket implementation (server-side)
pub struct ZmqRouter {
    context: Arc<zmq::Context>,
    config: ServerConfig,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
}

impl ZmqRouter {
    /// Create a new ROUTER socket
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

impl Transport for ZmqRouter {
    fn start(&mut self) -> TransportResult<()> {
        if *self.running.lock() {
            return Err(TransportError::AlreadyRunning);
        }
        
        // Create ROUTER socket
        let socket = self.context.socket(zmq::ROUTER)?;
        
        // Set socket options
        if let Some(linger) = self.config.base.linger {
            socket.set_linger(linger.as_millis() as i32)?;
        } else {
            socket.set_linger(0)?;
        }
        
        socket.set_router_mandatory(false)?;
        socket.set_rcvhwm(self.config.base.recv_hwm as i32)?;
        socket.set_sndhwm(self.config.base.send_hwm as i32)?;
        
        // Bind socket
        socket
            .bind(&self.config.base.address)
            .map_err(|e| TransportError::BindFailed(e.to_string()))?;
        
        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;
        
        println!(
            "🦀 [ZMQ-ROUTER] Listening on {}",
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
        "zmq-router"
    }
}

impl RequestReplyServer for ZmqRouter {
    fn receive(&self) -> TransportResult<(Vec<u8>, Box<dyn ReplyHandle>)> {
        self.receive_timeout(0) // 0 = blocking
    }
    
    fn receive_timeout(
        &self,
        timeout_ms: u64,
    ) -> TransportResult<(Vec<u8>, Box<dyn ReplyHandle>)> {
        let sock_guard = self.socket.lock();
        let sock = sock_guard
            .as_ref()
            .ok_or(TransportError::NotRunning)?;
        
        // Poll for messages
        if timeout_ms > 0 {
            let poll_items = &mut [sock.as_poll_item(zmq::POLLIN)];
            zmq::poll(poll_items, timeout_ms as i64)?;
            
            if !poll_items[0].is_readable() {
                return Err(TransportError::Timeout);
            }
        }
        
        // Receive multipart message: [identity, delimiter, request_data]
        let mut msg_parts = Vec::new();
        let mut more = true;
        
        while more {
            let mut msg = zmq::Message::new();
            sock.recv(&mut msg, 0)?;
            msg_parts.push(msg.to_vec());
            more = sock.get_rcvmore()?;
        }
        
        if msg_parts.len() < 3 {
            return Err(TransportError::InvalidMessage(format!(
                "Expected at least 3 parts, got {}",
                msg_parts.len()
            )));
        }
        
        // Extract identity and request data
        let identity = msg_parts[0].clone();
        let request_data = msg_parts[2].clone();
        
        // Create reply handle
        let reply = ZmqRouterReplyHandle {
            socket: Arc::clone(&self.socket),
            identity,
        };
        
        Ok((request_data, Box::new(reply)))
    }
    
    fn poll(&self, timeout_ms: u64) -> TransportResult<bool> {
        let sock_guard = self.socket.lock();
        let sock = sock_guard
            .as_ref()
            .ok_or(TransportError::NotRunning)?;
        
        let poll_items = &mut [sock.as_poll_item(zmq::POLLIN)];
        zmq::poll(poll_items, timeout_ms as i64)?;
        
        Ok(poll_items[0].is_readable())
    }
}

/// Reply handle for ROUTER socket
struct ZmqRouterReplyHandle {
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    identity: Vec<u8>,
}

impl ReplyHandle for ZmqRouterReplyHandle {
    fn send(&self, data: &[u8]) -> TransportResult<()> {
        let sock_guard = self.socket.lock();
        let sock = sock_guard
            .as_ref()
            .ok_or(TransportError::NotRunning)?;
        
        // Send multipart reply: [identity, delimiter, response_data]
        sock.send(&self.identity, zmq::SNDMORE)?;
        sock.send(&Vec::<u8>::new(), zmq::SNDMORE)?;
        sock.send(data, 0)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_creation() {
        let context = Arc::new(zmq::Context::new());
        let config = ServerConfig::new("tcp://127.0.0.1:30000");
        let router = ZmqRouter::new(context, config);
        assert!(router.is_ok());
    }
    
    #[test]
    fn test_router_start_stop() {
        let mut router = ZmqRouter::with_address("tcp://127.0.0.1:30001").unwrap();
        assert!(!router.is_running());
        
        router.start().unwrap();
        assert!(router.is_running());
        
        router.stop().unwrap();
        assert!(!router.is_running());
    }
}




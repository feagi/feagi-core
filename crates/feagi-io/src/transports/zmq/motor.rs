// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Motor stream for sending motor commands to agents (ZMQ fallback for remote clients)
// Uses PUB socket pattern for one-to-many distribution

use feagi_structures::FeagiDataError;
use parking_lot::Mutex;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::runtime::Runtime;
use tokio::task::block_in_place;
use tracing::{debug, error, info};
use zeromq::{PubSocket, Socket, SocketSend, ZmqMessage};

fn block_on_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
    if Handle::try_current().is_ok() {
        block_in_place(|| Handle::current().block_on(future))
    } else {
        runtime.block_on(future)
    }
}

/// Motor stream for publishing motor commands
#[derive(Clone)]
pub struct MotorStream {
    runtime: Arc<Runtime>,
    bind_address: String,
    socket: Arc<Mutex<Option<PubSocket>>>,
    running: Arc<Mutex<bool>>,
}

impl MotorStream {
    /// Create a new motor stream
    pub fn new(runtime: Arc<Runtime>, bind_address: &str) -> Result<Self, FeagiDataError> {
        Ok(Self {
            runtime,
            bind_address: bind_address.to_string(),
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Start the motor stream
    pub fn start(&self) -> Result<(), FeagiDataError> {
        if *self.running.lock() {
            return Err(FeagiDataError::BadParameters(
                "Motor stream already running".to_string(),
            ));
        }

        // Create PUB socket for broadcasting motor data
        let mut socket = PubSocket::new();
        block_on_runtime(self.runtime.as_ref(), socket.bind(&self.bind_address))
            .map_err(|e| super::bind_error_to_feagi_error(&self.bind_address, e))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!("🦀 [ZMQ-MOTOR] Listening on {}", self.bind_address);

        Ok(())
    }

    /// Stop the motor stream
    pub fn stop(&self) -> Result<(), FeagiDataError> {
        *self.running.lock() = false;
        *self.socket.lock() = None;
        Ok(())
    }

    /// Publish motor data to all subscribers
    pub fn publish(&self, data: &[u8]) -> Result<(), FeagiDataError> {
        // Fast path: If stream not running, don't try to send
        // This prevents errors when no motor agents are connected
        if !*self.running.lock() {
            return Ok(()); // Silently discard - this is expected when no motor agents connected
        }

        let mut sock_guard = self.socket.lock();
        let sock = match sock_guard.as_mut() {
            Some(s) => s,
            None => {
                return Err(FeagiDataError::BadParameters(
                    "Motor stream not started".to_string(),
                ))
            }
        };

        let message = ZmqMessage::from(data.to_vec());
        block_on_runtime(self.runtime.as_ref(), sock.send(message)).map_err(|e| {
            FeagiDataError::InternalError(format!("Failed to send motor data: {}", e))
        })?;

        Ok(())
    }

    /// Publish motor data with agent_id as ZMQ topic for filtering
    pub fn publish_with_topic(&self, topic: &[u8], data: &[u8]) -> Result<(), FeagiDataError> {
        // Fast path: If stream not running, don't try to send
        if !*self.running.lock() {
            return Ok(()); // Silently discard - no agents connected
        }

        let mut sock_guard = self.socket.lock();
        let sock = match sock_guard.as_mut() {
            Some(s) => s,
            None => {
                return Err(FeagiDataError::BadParameters(
                    "Motor stream not started".to_string(),
                ))
            }
        };

        // Send as multipart message: [topic, data]
        debug!(
            "[MOTOR-STREAM] 📤 Publishing multipart: topic='{}' ({} bytes), data={} bytes",
            String::from_utf8_lossy(topic),
            topic.len(),
            data.len()
        );

        let mut message = ZmqMessage::from(data.to_vec());
        message.prepend(&ZmqMessage::from(topic.to_vec()));
        block_on_runtime(self.runtime.as_ref(), sock.send(message)).map_err(|e| {
            error!("[MOTOR-STREAM] ❌ send failed: {}", e);
            FeagiDataError::InternalError(format!("Failed to send multipart motor data: {}", e))
        })?;
        debug!("[MOTOR-STREAM] ✅ Multipart sent successfully");

        Ok(())
    }

    /// Check if stream is running
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motor_stream_creation() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let stream = MotorStream::new(runtime, "tcp://127.0.0.1:30015");
        assert!(stream.is_ok());
    }
}

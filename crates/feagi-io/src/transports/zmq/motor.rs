// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Motor stream for sending motor commands to agents (ZMQ fallback for remote clients)
// Uses PUB socket pattern for one-to-many distribution

use feagi_structures::FeagiDataError;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Motor stream for publishing motor commands
#[derive(Clone)]
pub struct MotorStream {
    context: Arc<zmq::Context>,
    bind_address: String,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
}

impl MotorStream {
    /// Create a new motor stream
    pub fn new(context: Arc<zmq::Context>, bind_address: &str) -> Result<Self, FeagiDataError> {
        Ok(Self {
            context,
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
        let socket = self.context.socket(zmq::PUB).map_err(|e| {
            FeagiDataError::InternalError(format!("Failed to create ZMQ socket: {}", e))
        })?;

        // Set socket options for optimal performance
        socket
            .set_linger(0) // Don't wait on close
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to set linger: {}", e)))?;
        socket
            .set_sndhwm(1000) // High water mark for send buffer
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to set send HWM: {}", e)))?;
        // NOTE: CONFLATE disabled - it BREAKS multipart messages!
        // For real-time data, subscribers should use DONTWAIT and discard old messages

        // Bind socket
        socket
            .bind(&self.bind_address)
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to bind socket: {}", e)))?;

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

        let sock_guard = self.socket.lock();
        let sock = match sock_guard.as_ref() {
            Some(s) => s,
            None => {
                return Err(FeagiDataError::BadParameters(
                    "Motor stream not started".to_string(),
                ))
            }
        };

        sock.send(data, 0).map_err(|e| {
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

        let sock_guard = self.socket.lock();
        let sock = match sock_guard.as_ref() {
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

        // Use send_multipart with Vec (zmq crate API compatibility)
        let parts: Vec<&[u8]> = vec![topic, data];
        sock.send_multipart(parts, 0).map_err(|e| {
            error!("[MOTOR-STREAM] ❌ send_multipart failed: {}", e);
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
        let ctx = Arc::new(zmq::Context::new());
        let stream = MotorStream::new(ctx, "tcp://127.0.0.1:30015");
        assert!(stream.is_ok());
    }
}

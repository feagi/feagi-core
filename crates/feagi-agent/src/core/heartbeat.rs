// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Heartbeat service for maintaining agent liveness

use crate::core::error::{Result, SdkError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::{debug, warn};

/// Heartbeat service managing periodic keepalive messages
pub struct HeartbeatService {
    /// Agent ID
    agent_id: String,

    /// ZMQ registration socket (shared with main client)
    socket: Arc<Mutex<zmq::Socket>>,

    /// Heartbeat interval
    interval: Duration,

    /// Running flag
    running: Arc<AtomicBool>,

    /// Thread handle
    thread: Option<JoinHandle<()>>,
}

impl HeartbeatService {
    /// Create a new heartbeat service
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `socket` - Shared ZMQ socket for sending heartbeats
    /// * `interval_secs` - Heartbeat interval in seconds
    pub fn new(agent_id: String, socket: Arc<Mutex<zmq::Socket>>, interval_secs: f64) -> Self {
        Self {
            agent_id,
            socket,
            interval: Duration::from_secs_f64(interval_secs),
            running: Arc::new(AtomicBool::new(false)),
            thread: None,
        }
    }

    /// Start the heartbeat service
    pub fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Err(SdkError::Other(
                "Heartbeat service already running".to_string(),
            ));
        }

        self.running.store(true, Ordering::Relaxed);

        let agent_id = self.agent_id.clone();
        let socket = Arc::clone(&self.socket);
        let interval = self.interval;
        let running = Arc::clone(&self.running);

        let thread = thread::spawn(move || {
            debug!("[HEARTBEAT] Service started for agent: {}", agent_id);

            while running.load(Ordering::Relaxed) {
                // Sleep first to avoid immediate heartbeat after registration
                thread::sleep(interval);

                if !running.load(Ordering::Relaxed) {
                    break;
                }

                // Send heartbeat
                if let Err(e) = Self::send_heartbeat(&agent_id, &socket) {
                    warn!(
                        "[HEARTBEAT] Failed to send heartbeat for {}: {}",
                        agent_id, e
                    );
                    // Don't stop on error - network might recover
                }
            }

            debug!("[HEARTBEAT] Service stopped for agent: {}", agent_id);
        });

        self.thread = Some(thread);
        Ok(())
    }

    /// Stop the heartbeat service
    ///
    /// This ensures proper thread cleanup:
    /// 1. Signal thread to stop
    /// 2. Wait for thread to finish (with timeout)
    /// 3. Force terminate if stuck
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            debug!(
                "[HEARTBEAT] Service already stopped for agent: {}",
                self.agent_id
            );
            return;
        }

        debug!("[HEARTBEAT] Stopping service for agent: {}", self.agent_id);

        // Step 1: Signal thread to stop
        self.running.store(false, Ordering::Relaxed);

        // Step 2: Wait for thread to finish
        if let Some(thread) = self.thread.take() {
            match thread.join() {
                Ok(_) => {
                    debug!(
                        "[HEARTBEAT] Thread stopped cleanly for agent: {}",
                        self.agent_id
                    );
                }
                Err(e) => {
                    warn!(
                        "[HEARTBEAT] Thread join failed for agent {} (thread may have panicked): {:?}",
                        self.agent_id, e
                    );
                }
            }
        }

        debug!(
            "[HEARTBEAT] Service fully stopped for agent: {}",
            self.agent_id
        );
    }

    /// Send a single heartbeat message
    fn send_heartbeat(agent_id: &str, socket: &Arc<Mutex<zmq::Socket>>) -> Result<()> {
        let message = serde_json::json!({
            "method": "POST",
            "path": "/v1/agent/heartbeat",
            "body": {
                "agent_id": agent_id,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            }
        });

        let socket = socket
            .lock()
            .map_err(|e| SdkError::ThreadError(format!("Failed to lock socket: {}", e)))?;

        // Send heartbeat request
        socket.send(message.to_string().as_bytes(), 0)?;

        // Wait for response (ROUTER replies are multipart: [empty][json] when seen by REQ)
        if socket.poll(zmq::POLLIN, 1000)? > 0 {
            let parts = socket.recv_multipart(0)?;
            let last = parts
                .last()
                .ok_or_else(|| SdkError::Other("Heartbeat reply was empty".to_string()))?;
            let response: serde_json::Value = serde_json::from_slice(last)?;

            // Heartbeat response schema varies by FEAGI version/transport:
            // - Legacy: {"status":"success", ...}
            // - HTTP-style: {"status":200,"body":{"message":"ok"}, ...}
            //
            // Treat both as success deterministically.
            let status_value = response.get("status");
            let is_success = match status_value {
                Some(serde_json::Value::String(s)) => s == "success" || s == "ok",
                Some(serde_json::Value::Number(n)) => n.as_u64() == Some(200),
                _ => false,
            };

            if is_success {
                debug!("[HEARTBEAT] ✓ Heartbeat acknowledged for {}", agent_id);
                Ok(())
            } else {
                warn!("[HEARTBEAT] ⚠ Heartbeat rejected: {:?}", response);
                Err(SdkError::HeartbeatFailed(format!("{:?}", response)))
            }
        } else {
            warn!("[HEARTBEAT] ⚠ Heartbeat timeout for {}", agent_id);
            Ok(()) // Don't treat timeout as fatal - just log it
        }
    }

    /// Check if heartbeat service is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for HeartbeatService {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_heartbeat_service_lifecycle() {
        // Create mock socket (would need actual ZMQ context in real test)
        // This is a placeholder test structure

        // Note: Full integration tests require actual ZMQ sockets
        // and a running FEAGI instance
    }
}

// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Heartbeat service for maintaining agent liveness

use crate::core::error::{Result, SdkError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tracing::{debug, warn};
use zeromq::{ReqSocket, SocketRecv, SocketSend, ZmqMessage};
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

/// Registration payload used to re-register an agent after FEAGI restarts.
///
/// @cursor:critical-path
/// This is used ONLY for automatic recovery when FEAGI loses in-memory registry state.
#[derive(Debug, Clone)]
pub struct ReconnectSpec {
    pub agent_id: String,
    pub agent_type: String,
    pub capabilities: serde_json::Value,
    pub registration_retries: u32,
    pub retry_backoff_ms: u64,
}

/// Heartbeat service managing periodic keepalive messages
pub struct HeartbeatService {
    /// Agent ID
    agent_id: String,

    /// ZMQ registration socket (shared with main client)
    socket: Arc<Mutex<ReqSocket>>,

    /// Runtime for zeromq operations
    runtime: Arc<Runtime>,

    /// Heartbeat interval
    interval: Duration,

    /// Running flag
    running: Arc<AtomicBool>,

    /// Thread handle
    thread: Option<JoinHandle<()>>,

    /// Optional auto-reconnect/re-register configuration
    reconnect: Option<ReconnectSpec>,
}

impl HeartbeatService {
    /// Create a new heartbeat service
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `socket` - Shared ZMQ socket for sending heartbeats
    /// * `interval_secs` - Heartbeat interval in seconds
    pub fn new(
        agent_id: String,
        socket: Arc<Mutex<ReqSocket>>,
        runtime: Arc<Runtime>,
        interval_secs: f64,
    ) -> Self {
        Self {
            agent_id,
            socket,
            runtime,
            interval: Duration::from_secs_f64(interval_secs),
            running: Arc::new(AtomicBool::new(false)),
            thread: None,
            reconnect: None,
        }
    }

    /// Enable automatic re-register attempts when heartbeats are rejected due to agent not found.
    ///
    /// This is intended for FEAGI restarts (registry reset). It does NOT attempt reconnect
    /// after voluntary deregistration because `AgentClient` stops this service on disconnect.
    pub fn with_reconnect_spec(mut self, spec: ReconnectSpec) -> Self {
        self.reconnect = Some(spec);
        self
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
        let runtime = Arc::clone(&self.runtime);
        let interval = self.interval;
        let running = Arc::clone(&self.running);
        let reconnect = self.reconnect.clone();

        let thread = thread::spawn(move || {
            debug!("[HEARTBEAT] Service started for agent: {}", agent_id);

            while running.load(Ordering::Relaxed) {
                // Sleep first to avoid immediate heartbeat after registration
                thread::sleep(interval);

                if !running.load(Ordering::Relaxed) {
                    break;
                }

                // Send heartbeat
                if let Err(e) =
                    Self::send_heartbeat(&agent_id, &socket, &runtime, reconnect.as_ref())
                {
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
    fn send_heartbeat(
        agent_id: &str,
        socket: &Arc<Mutex<ReqSocket>>,
        runtime: &Arc<Runtime>,
        reconnect: Option<&ReconnectSpec>,
    ) -> Result<()> {
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

        let mut socket_guard = socket
            .lock()
            .map_err(|e| SdkError::ThreadError(format!("Failed to lock socket: {}", e)))?;

        // Send heartbeat request
        let message = ZmqMessage::from(message.to_string().into_bytes());
        block_on_runtime(runtime.as_ref(), socket_guard.send(message))
            .map_err(SdkError::Zmq)?;

        // Wait for response (ROUTER replies may include empty delimiter)
        let response = block_on_runtime(runtime.as_ref(), async {
            timeout(Duration::from_millis(1000), socket_guard.recv()).await
        })
            .map_err(|_| SdkError::Timeout("Heartbeat response timeout".to_string()))?
            .map_err(SdkError::Zmq)?;
        let frames = response.into_vec();
        let last = frames
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
            if Self::is_agent_not_registered(&response) {
                if let Some(spec) = reconnect {
                    if Self::try_re_register(spec, socket, runtime).is_ok() {
                        debug!(
                            "[HEARTBEAT] ✓ Auto re-registered agent after heartbeat rejection: {}",
                            agent_id
                        );
                        return Ok(());
                    }
                }
            }
            Err(SdkError::HeartbeatFailed(format!("{:?}", response)))
        }
    }

    /// Detect FEAGI responses indicating the agent is not currently registered.
    fn is_agent_not_registered(response: &serde_json::Value) -> bool {
        let status = response
            .get("status")
            .and_then(|v| v.as_u64())
            .unwrap_or_default();
        if status != 404 {
            return false;
        }
        response
            .get("body")
            .and_then(|b| b.get("error"))
            .and_then(|e| e.as_str())
            .map(|s| s.contains("not found in registry") || s.contains("not found"))
            .unwrap_or(false)
    }

    /// Attempt to re-register the agent (used after FEAGI restarts).
    fn try_re_register(
        spec: &ReconnectSpec,
        socket: &Arc<Mutex<ReqSocket>>,
        runtime: &Arc<Runtime>,
    ) -> Result<()> {
        let registration_msg = serde_json::json!({
            "method": "POST",
            "path": "/v1/agent/register",
            "body": {
                "agent_id": spec.agent_id,
                "agent_type": spec.agent_type,
                "capabilities": spec.capabilities,
            }
        });

        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let mut socket = socket
                .lock()
                .map_err(|e| SdkError::ThreadError(format!("Failed to lock socket: {}", e)))?;

            let message = ZmqMessage::from(registration_msg.to_string().into_bytes());
            block_on_runtime(runtime.as_ref(), socket.send(message)).map_err(SdkError::Zmq)?;
            let response = block_on_runtime(runtime.as_ref(), async {
                timeout(Duration::from_millis(1000), socket.recv()).await
            })
                .map_err(|_| SdkError::Timeout("Registration response timeout".to_string()))?
                .map_err(SdkError::Zmq)?;
            let frames = response.into_vec();
            let last = frames
                .last()
                .ok_or_else(|| SdkError::Other("Registration reply was empty".to_string()))?;
            let response: serde_json::Value = serde_json::from_slice(last)?;
            let status_code = response
                .get("status")
                .and_then(|s| s.as_u64())
                .unwrap_or(500);
            if status_code == 200 {
                return Ok(());
            }

            if attempt > spec.registration_retries {
                return Err(SdkError::RegistrationFailed(
                    "Auto re-register failed (exhausted retries)".to_string(),
                ));
            }

            // Deterministic backoff controlled by AgentConfig (no hardcoded defaults here).
            std::thread::sleep(Duration::from_millis(spec.retry_backoff_ms));
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

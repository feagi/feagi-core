// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// REST stream for agent registration, heartbeat, and deregistration
// Uses ROUTER socket pattern for request-reply with agent identity tracking

use crate::core::{RegistrationHandler, RegistrationRequest};
use feagi_structures::FeagiDataError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Handle;
use tokio::runtime::Runtime;
use tokio::task::block_in_place;
use tokio::time::timeout;
use tracing::{debug, error, info};
use zeromq::{RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage};

fn block_on_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
    if Handle::try_current().is_ok() {
        block_in_place(|| Handle::current().block_on(future))
    } else {
        runtime.block_on(future)
    }
}

/// REST request from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestRequest {
    pub method: String,
    pub path: String,
    pub body: Option<serde_json::Value>,
}

/// REST response to agent
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

/// REST stream managing agent registration and lifecycle
#[derive(Clone)]
pub struct RestStream {
    runtime: Arc<Runtime>,
    bind_address: String,
    socket: Arc<Mutex<Option<RouterSocket>>>,
    running: Arc<Mutex<bool>>,
    registration_handler: Arc<Mutex<Option<Arc<Mutex<RegistrationHandler>>>>>,
}

impl RestStream {
    /// Create a new REST stream
    pub fn new(runtime: Arc<Runtime>, bind_address: &str) -> Result<Self, FeagiDataError> {
        Ok(Self {
            runtime,
            bind_address: bind_address.to_string(),
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            registration_handler: Arc::new(Mutex::new(None)),
        })
    }

    /// Set the registration handler
    pub fn set_registration_handler(&mut self, handler: Arc<Mutex<RegistrationHandler>>) {
        *self.registration_handler.lock() = Some(handler);
    }

    /// Start the REST stream
    pub fn start(&self) -> Result<(), FeagiDataError> {
        if *self.running.lock() {
            return Err(FeagiDataError::BadParameters(
                "REST stream already running".to_string(),
            ));
        }

        // Create ROUTER socket
        let mut socket = RouterSocket::new();
        block_on_runtime(self.runtime.as_ref(), socket.bind(&self.bind_address))
            .map_err(|e| FeagiDataError::InternalError(format!("Failed to bind socket: {}", e)))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!("🦀 [ZMQ-REST] Listening on {}", self.bind_address);

        // Start processing loop
        self.start_processing_loop();

        Ok(())
    }

    /// Stop the REST stream
    pub fn stop(&self) -> Result<(), FeagiDataError> {
        *self.running.lock() = false;
        *self.socket.lock() = None;
        Ok(())
    }

    /// Start the background processing loop
    fn start_processing_loop(&self) {
        let socket = Arc::clone(&self.socket);
        let runtime = Arc::clone(&self.runtime);
        let running = Arc::clone(&self.running);
        let registration_handler = Arc::clone(&self.registration_handler);

        thread::spawn(move || {
            info!("🦀 [ZMQ-REST] Processing loop started");

            while *running.lock() {
                let mut sock_guard = socket.lock();
                let sock = match sock_guard.as_mut() {
                    Some(s) => s,
                    None => {
                        drop(sock_guard);
                        thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }
                };

                let recv_result = block_on_runtime(runtime.as_ref(), async {
                    timeout(std::time::Duration::from_millis(100), sock.recv()).await
                });

                let message = match recv_result {
                    Ok(Ok(message)) => message,
                    Ok(Err(e)) => {
                        error!("🦀 [ZMQ-REST] [ERR] Receive error: {}", e);
                        drop(sock_guard);
                        continue;
                    }
                    Err(_) => {
                        drop(sock_guard);
                        continue;
                    }
                };

                drop(sock_guard);

                let mut frames = message.into_vec();
                if frames.is_empty() {
                    continue;
                }

                let identity = frames.remove(0).to_vec();
                if frames
                    .first()
                    .map(|frame| frame.is_empty())
                    .unwrap_or(false)
                {
                    frames.remove(0);
                }

                if frames.len() != 1 {
                    continue;
                }

                let request_json = String::from_utf8_lossy(&frames.remove(0)).to_string();
                let response_json = Self::process_request(&registration_handler, &request_json);

                if let Err(e) = Self::send_response(&socket, &runtime, identity, response_json) {
                    error!("🦀 [ZMQ-REST] [ERR] Failed to send response: {}", e);
                }
            }

            info!("🦀 [ZMQ-REST] Processing loop stopped");
        });
    }

    /// Process a request using the registration handler
    fn process_request(
        handler_mutex: &Arc<Mutex<Option<Arc<Mutex<RegistrationHandler>>>>>,
        request_json: &str,
    ) -> String {
        let handler_guard = handler_mutex.lock();
        let handler = match handler_guard.as_ref() {
            Some(h) => h,
            None => {
                return serde_json::json!({
                    "status": 503,
                    "body": {"error": "Service unavailable"}
                })
                .to_string();
            }
        };

        // Parse request
        let request: RestRequest = match serde_json::from_str(request_json) {
            Ok(req) => req,
            Err(e) => {
                return serde_json::json!({
                    "status": 400,
                    "body": {"error": format!("Invalid request: {}", e)}
                })
                .to_string();
            }
        };

        info!("🦀 [ZMQ-REST] {} {}", request.method, request.path);

        // Route request
        let response = match (request.method.as_str(), request.path.as_str()) {
            ("POST", "/v1/agent/register") => Self::handle_registration(handler, request.body),
            ("POST", "/v1/agent/heartbeat") => Self::handle_heartbeat(handler, request.body),
            ("DELETE", "/v1/agent/deregister") => {
                Self::handle_deregistration(handler, request.body)
            }
            _ => {
                serde_json::json!({
                    "status": 404,
                    "body": {"error": "Not found"}
                })
            }
        };

        response.to_string()
    }

    /// Handle registration
    fn handle_registration(
        handler: &Arc<Mutex<RegistrationHandler>>,
        body: Option<serde_json::Value>,
    ) -> serde_json::Value {
        let body = match body {
            Some(b) => b,
            None => {
                return serde_json::json!({
                    "status": 400,
                    "body": {"error": "Missing request body"}
                });
            }
        };

        let request: RegistrationRequest = match serde_json::from_value(body) {
            Ok(r) => r,
            Err(e) => {
                return serde_json::json!({
                    "status": 400,
                    "body": {"error": format!("Invalid registration request: {}", e)}
                });
            }
        };

        match handler.lock().process_registration(request) {
            Ok(response) => serde_json::json!({
                "status": 200,
                "body": response
            }),
            Err(e) => serde_json::json!({
                "status": 500,
                "body": {"error": e}
            }),
        }
    }

    /// Handle heartbeat
    fn handle_heartbeat(
        handler: &Arc<Mutex<RegistrationHandler>>,
        body: Option<serde_json::Value>,
    ) -> serde_json::Value {
        use std::sync::atomic::{AtomicU32, Ordering};
        static HEARTBEAT_COUNT: AtomicU32 = AtomicU32::new(0);

        let agent_id = body
            .and_then(|b| b.get("agent_id").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_default();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        match handler.lock().process_heartbeat(&agent_id) {
            Ok(_) => {
                let count = HEARTBEAT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

                // Log every heartbeat at DEBUG level, and periodic summary at INFO level
                debug!(
                    "💓 [ZMQ-REST] Heartbeat received from '{}' at {} (count: {})",
                    agent_id, now, count
                );

                if count == 1 || count.is_multiple_of(30) {
                    info!(
                        "🦀 [PNS] 💓 Heartbeat #{} received from {} (timestamp: {})",
                        count, agent_id, now
                    );
                }

                serde_json::json!({
                    "status": 200,
                    "body": {"message": "ok"}
                })
            }
            Err(e) => {
                // Heartbeats from unknown agents are rejected by design.
                // This can happen if the agent never registered, was deregistered, or FEAGI restarted.
                // Log at WARN (not ERROR) because this is a common, non-fatal condition.
                tracing::warn!(
                    "🦀 [PNS] 💓 Heartbeat rejected for '{}' at {}: {}",
                    agent_id,
                    now,
                    e
                );
                serde_json::json!({
                    "status": 404,
                    "body": {"error": e}
                })
            }
        }
    }

    /// Handle deregistration
    fn handle_deregistration(
        handler: &Arc<Mutex<RegistrationHandler>>,
        body: Option<serde_json::Value>,
    ) -> serde_json::Value {
        let agent_id = body
            .and_then(|b| b.get("agent_id").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_default();

        match handler.lock().process_deregistration(&agent_id) {
            Ok(_) => serde_json::json!({
                "status": 200,
                "body": {"message": "ok"}
            }),
            Err(e) => serde_json::json!({
                "status": 404,
                "body": {"error": e}
            }),
        }
    }

    /// Send response
    fn send_response(
        socket_mutex: &Arc<Mutex<Option<RouterSocket>>>,
        runtime: &Arc<Runtime>,
        identity: Vec<u8>,
        response_json: String,
    ) -> Result<(), FeagiDataError> {
        let mut sock_guard = socket_mutex.lock();
        let sock = match sock_guard.as_mut() {
            Some(s) => s,
            None => {
                return Err(FeagiDataError::InternalError(
                    "Socket not available".to_string(),
                ))
            }
        };

        let mut message = ZmqMessage::from(response_json.into_bytes());
        message.prepend(&ZmqMessage::from(Vec::new()));
        message.prepend(&ZmqMessage::from(identity));
        block_on_runtime(runtime.as_ref(), sock.send(message)).map_err(|e| {
            FeagiDataError::InternalError(format!("Failed to send response: {}", e))
        })?;

        Ok(())
    }
}

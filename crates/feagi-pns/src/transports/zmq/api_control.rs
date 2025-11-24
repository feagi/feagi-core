//! API Control stream using feagi-transports
//!
//! This is a refactored version using the feagi-transports abstraction layer.
//! Domain logic (request routing, NPU queries, RPC) is preserved, but transport
//! primitives are now provided by feagi-transports.

use feagi_transports::prelude::*;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use tracing::{info, error};

/// API request from FastAPI process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub method: String,      // HTTP method: GET, POST, PUT, DELETE
    pub path: String,         // Endpoint path: /v1/npu/...
    pub body: Option<serde_json::Value>,
    pub query_params: Option<serde_json::Value>,
}

/// API response to FastAPI process
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

/// RPC callback type for Python CoreAPIService calls
pub type RpcCallback = Arc<Mutex<Option<Box<dyn Fn(&str, serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>>>>;

/// API Control stream managing API process communication
#[derive(Clone)]
pub struct ApiControlStream {
    /// ZMQ Router transport from feagi-transports
    router: Arc<Mutex<Option<ZmqRouter>>>,
    running: Arc<Mutex<bool>>,
    /// Reference to Rust NPU for direct queries (no Python overhead!)
    npu: Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>>>>,
    /// RPC callback to Python CoreAPIService (for generic method calls)
    rpc_callback: RpcCallback,
}

impl ApiControlStream {
    /// Create a new API control stream
    pub fn new(context: Arc<zmq::Context>, bind_address: &str) -> Result<Self, String> {
        // Create transport config
        let config = ServerConfig::new(bind_address)
            .base
            .with_recv_hwm(10000)
            .with_send_hwm(10000);
        
        let server_config = ServerConfig {
            base: config,
            max_connections: 0,
            track_connections: true,
        };
        
        // Create ZMQ router using feagi-transports
        let router = ZmqRouter::new(context, server_config)
            .map_err(|e| format!("Failed to create router: {}", e))?;
        
        Ok(Self {
            router: Arc::new(Mutex::new(Some(router))),
            running: Arc::new(Mutex::new(false)),
            npu: Arc::new(Mutex::new(None)),
            rpc_callback: Arc::new(Mutex::new(None)),
        })
    }

    /// Set the Rust NPU reference for direct queries
    pub fn set_npu(&mut self, npu: Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>) {
        *self.npu.lock() = Some(npu);
        info!("🦀 [API-CONTROL] NPU connected for direct queries");
    }

    /// Set RPC callback for generic CoreAPIService method calls
    pub fn set_rpc_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str, serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    {
        *self.rpc_callback.lock() = Some(Box::new(callback));
        info!("🦀 [API-CONTROL] RPC callback registered for CoreAPIService");
    }

    /// Start the API control stream
    pub fn start(&self) -> Result<(), String> {
        if *self.running.lock() {
            return Err("API control stream already running".to_string());
        }

        // Start the router transport
        let mut router_guard = self.router.lock();
        if let Some(router) = router_guard.as_mut() {
            router.start().map_err(|e| e.to_string())?;
        } else {
            return Err("Router not initialized".to_string());
        }
        drop(router_guard);

        *self.running.lock() = true;

        info!("🦀 [ZMQ-API-CONTROL] Listening (via feagi-transports)");

        // Start processing loop
        self.start_processing_loop();

        Ok(())
    }

    /// Stop the API control stream
    pub fn stop(&self) -> Result<(), String> {
        *self.running.lock() = false;
        
        let mut router_guard = self.router.lock();
        if let Some(router) = router_guard.as_mut() {
            router.stop().map_err(|e| e.to_string())?;
        }
        *router_guard = None;
        
        Ok(())
    }

    /// Start the background processing loop
    fn start_processing_loop(&self) {
        let router = Arc::clone(&self.router);
        let running = Arc::clone(&self.running);
        let npu = Arc::clone(&self.npu);
        let rpc_callback = Arc::clone(&self.rpc_callback);

        thread::spawn(move || {
            info!("🦀 [ZMQ-API-CONTROL] Processing loop started");

            while *running.lock() {
                // Try to receive request with timeout
                let router_guard = router.lock();
                let result = if let Some(ref r) = *router_guard {
                    r.receive_timeout(100)
                } else {
                    drop(router_guard);
                    thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                };
                drop(router_guard);

                match result {
                    Ok((request_data, reply_handle)) => {
                        let request_json = String::from_utf8_lossy(&request_data).to_string();
                        
                        info!("🦀 [ZMQ-API-CONTROL] 📨 Received request ({} bytes)", request_json.len());
                        info!("🦀 [ZMQ-API-CONTROL] 📨 Request: {}", &request_json[..request_json.len().min(200)]);

                        let response_json = Self::process_request(&npu, &rpc_callback, &request_json);
                        
                        info!("🦀 [ZMQ-API-CONTROL] 📤 Sending response ({} bytes)", response_json.len());

                        if let Err(e) = reply_handle.send(response_json.as_bytes()) {
                            error!("🦀 [ZMQ-API-CONTROL] [ERR] Failed to send response: {}", e);
                        }
                    }
                    Err(TransportError::Timeout) => {
                        // Timeout is normal, just continue
                        continue;
                    }
                    Err(e) => {
                        error!("🦀 [ZMQ-API-CONTROL] [ERR] Receive error: {}", e);
                    }
                }
            }

            info!("🦀 [ZMQ-API-CONTROL] Processing loop stopped");
        });
    }

    /// Process a request using the NPU (domain logic - unchanged)
    fn process_request(
        npu_mutex: &Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>>>>,
        rpc_callback: &RpcCallback,
        request_json: &str,
    ) -> String {
        let npu_guard = npu_mutex.lock();
        let npu_arc = match npu_guard.as_ref() {
            Some(n) => n,
            None => {
                return serde_json::json!({
                    "status": 503,
                    "body": {"error": "NPU not available"}
                })
                .to_string();
            }
        };

        // Parse request
        let request: ApiRequest = match serde_json::from_str(request_json) {
            Ok(req) => req,
            Err(e) => {
                return serde_json::json!({
                    "status": 400,
                    "body": {"error": format!("Invalid request: {}", e)}
                })
                .to_string();
            }
        };

        // Route request to appropriate handler
        let response = Self::route_request(npu_arc, rpc_callback, &request);

        response.to_string()
    }

    /// Route API request to appropriate handler (domain logic - unchanged)
    fn route_request(
        npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>,
        rpc_callback: &RpcCallback,
        request: &ApiRequest,
    ) -> serde_json::Value {
        // Log high-level request (for debugging)
        static FIRST_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            info!("🦀 [API-CONTROL] First request: {} {}", request.method, request.path);
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Route to handlers (domain logic preserved)
        match (request.method.as_str(), request.path.as_str()) {
            // Direct Rust queries (optimal performance, no Python)
            ("GET", "/v1/npu/stats") => Self::handle_npu_stats(npu),
            ("GET", "/v1/npu/cortical_areas") => Self::handle_cortical_areas(npu),
            ("GET", path) if path.starts_with("/v1/npu/cortical_area/") => {
                Self::handle_cortical_area_info(npu, path)
            }
            ("GET", "/v1/npu/fire_queue") => Self::handle_fire_queue(npu),
            ("GET", "/v1/health") => Self::handle_health_check(),
            // Internal state manager queries (for API subprocess)
            ("GET", "/internal/state/brain_readiness") => Self::handle_brain_readiness(npu),
            ("GET", "/internal/state/burst_engine_state") => Self::handle_burst_engine_state(npu),
            ("GET", "/internal/state/genome_state") => Self::handle_genome_state(npu),
            ("GET", "/internal/state/brain_stats") => Self::handle_brain_stats(npu),
            // Generic RPC endpoint (forwards to Python CoreAPIService)
            ("POST", "/rpc/core_api") => Self::handle_rpc(rpc_callback, request),
            _ => {
                serde_json::json!({
                    "status": 404,
                    "body": {"error": "Endpoint not implemented in API control stream"}
                })
            }
        }
    }

    // All handler methods below are domain logic - unchanged from original

    fn handle_npu_stats(npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        
        serde_json::json!({
            "status": 200,
            "body": {
                "burst_count": npu_lock.get_burst_count(),
                "power_amount": npu_lock.get_power_amount(),
                "cortical_area_count": npu_lock.get_registered_cortical_area_count(),
            }
        })
    }

    fn handle_cortical_areas(npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        let area_count = npu_lock.get_registered_cortical_area_count();
        
        serde_json::json!({
            "status": 200,
            "body": {
                "cortical_area_count": area_count,
                "message": "Detailed cortical area listing not yet implemented"
            }
        })
    }

    fn handle_cortical_area_info(_npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>, path: &str) -> serde_json::Value {
        let area_name = path.strip_prefix("/v1/npu/cortical_area/").unwrap_or("");
        
        serde_json::json!({
            "status": 501,
            "body": {
                "error": format!("Cortical area info for '{}' not yet implemented", area_name)
            }
        })
    }

    fn handle_fire_queue(_npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>) -> serde_json::Value {
        serde_json::json!({
            "status": 501,
            "body": {"error": "Fire queue query not yet implemented"}
        })
    }

    fn handle_health_check() -> serde_json::Value {
        serde_json::json!({
            "status": 200,
            "body": {
                "status": "ok",
                "service": "API Control Stream"
            }
        })
    }

    fn handle_brain_readiness(npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        let is_ready = npu_lock.get_burst_count() > 0 || npu_lock.get_neuron_count() > 0;
        serde_json::json!({
            "status": 200,
            "body": {"value": is_ready}
        })
    }

    fn handle_burst_engine_state(npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        let state = if npu_lock.get_burst_count() > 0 || npu_lock.get_neuron_count() > 0 { 2 } else { 0 };
        serde_json::json!({
            "status": 200,
            "body": {"value": state}
        })
    }

    fn handle_genome_state(npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        let state = if npu_lock.get_registered_cortical_area_count() > 0 { 2 } else { 0 };
        serde_json::json!({
            "status": 200,
            "body": {"value": state}
        })
    }

    fn handle_brain_stats(npu: &Arc<std::sync::Mutex<feagi_burst_engine::DynamicNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        serde_json::json!({
            "status": 200,
            "body": {
                "neuron_count": npu_lock.get_neuron_count(),
                "synapse_count": npu_lock.get_synapse_count(),
                "cortical_area_count": npu_lock.get_registered_cortical_area_count(),
                "memory_neuron_count": 0,
                "non_memory_neuron_count": npu_lock.get_neuron_count(),
            }
        })
    }

    fn handle_rpc(rpc_callback: &RpcCallback, request: &ApiRequest) -> serde_json::Value {
        info!("🦀 [API-CONTROL-RPC] Received RPC request");
        
        let rpc_payload = match &request.body {
            Some(body) => {
                info!("🦀 [API-CONTROL-RPC] Request has body: {:?}", body);
                body.clone()
            }
            None => {
                error!("🦀 [API-CONTROL-RPC] ERROR: Request missing body");
                return serde_json::json!({
                    "status": 400,
                    "body": {"error": "RPC request missing body"}
                });
            }
        };

        let method_name = match rpc_payload.get("method").and_then(|m| m.as_str()) {
            Some(m) => {
                info!("🦀 [API-CONTROL-RPC] Method name: {}", m);
                m.to_string()
            }
            None => {
                error!("🦀 [API-CONTROL-RPC] ERROR: Payload missing 'method' field");
                return serde_json::json!({
                    "status": 400,
                    "body": {"error": "RPC payload missing 'method' field"}
                });
            }
        };

        let callback_guard = rpc_callback.lock();
        let callback = match callback_guard.as_ref() {
            Some(cb) => {
                info!("🦀 [API-CONTROL-RPC] Callback registered, calling Python handler");
                cb
            }
            None => {
                error!("🦀 [API-CONTROL-RPC] ERROR: No RPC callback registered");
                return serde_json::json!({
                    "status": 503,
                    "body": {"error": "RPC callback not registered"}
                });
            }
        };

        info!("🦀 [API-CONTROL-RPC] Invoking Python handler for method: {}", method_name);
        match callback(&method_name, rpc_payload) {
            Ok(result) => {
                info!("🦀 [API-CONTROL-RPC] Python handler returned success");
                serde_json::json!({
                    "status": 200,
                    "body": result
                })
            }
            Err(e) => {
                error!("🦀 [API-CONTROL-RPC] Python handler returned error: {}", e);
                serde_json::json!({
                    "status": 500,
                    "body": {"error": e}
                })
            }
        }
    }
}


// API Control stream for serving FastAPI process via ZMQ
// Uses ROUTER socket pattern for request-reply with API process identity tracking
//
// This stream allows the FastAPI process to run in a separate Python process
// and communicate with FEAGI core via ZMQ, eliminating Python GIL contention.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;

/// API request from FastAPI process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub method: String,      // HTTP method: GET, POST, PUT, DELETE
    pub path: String,         // Endpoint path: /v1/npu/...
    pub body: Option<serde_json::Value>,
    pub query_params: Option<serde_json::Value>,
}

/// API response to FastAPI process
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
    context: Arc<zmq::Context>,
    bind_address: String,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
    /// Reference to Rust NPU for direct queries (no Python overhead!)
    npu: Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>>>>,
    /// RPC callback to Python CoreAPIService (for generic method calls)
    rpc_callback: RpcCallback,
}

impl ApiControlStream {
    /// Create a new API control stream
    pub fn new(context: Arc<zmq::Context>, bind_address: &str) -> Result<Self, String> {
        Ok(Self {
            context,
            bind_address: bind_address.to_string(),
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            npu: Arc::new(Mutex::new(None)),
            rpc_callback: Arc::new(Mutex::new(None)),
        })
    }

    /// Set the Rust NPU reference for direct queries
    pub fn set_npu(&mut self, npu: Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) {
        *self.npu.lock() = Some(npu);
        println!("🦀 [API-CONTROL] NPU connected for direct queries");
    }

    /// Set RPC callback for generic CoreAPIService method calls
    pub fn set_rpc_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str, serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    {
        *self.rpc_callback.lock() = Some(Box::new(callback));
        println!("🦀 [API-CONTROL] RPC callback registered for CoreAPIService");
    }

    /// Start the API control stream
    pub fn start(&self) -> Result<(), String> {
        if *self.running.lock() {
            return Err("API control stream already running".to_string());
        }

        // Create ROUTER socket
        let socket = self
            .context
            .socket(zmq::ROUTER)
            .map_err(|e| e.to_string())?;

        socket.set_linger(1000).map_err(|e| e.to_string())?;
        socket
            .set_router_mandatory(false)
            .map_err(|e| e.to_string())?;
        socket
            .set_rcvhwm(10000) // High water mark for receive buffer
            .map_err(|e| e.to_string())?;
        socket
            .set_sndhwm(10000) // High water mark for send buffer
            .map_err(|e| e.to_string())?;

        socket.bind(&self.bind_address).map_err(|e| e.to_string())?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        println!("🦀 [ZMQ-API-CONTROL] Listening on {}", self.bind_address);

        // Start processing loop
        self.start_processing_loop();

        Ok(())
    }

    /// Stop the API control stream
    pub fn stop(&self) -> Result<(), String> {
        *self.running.lock() = false;
        *self.socket.lock() = None;
        Ok(())
    }

    /// Start the background processing loop
    fn start_processing_loop(&self) {
        let socket = Arc::clone(&self.socket);
        let running = Arc::clone(&self.running);
        let npu = Arc::clone(&self.npu);
        let rpc_callback = Arc::clone(&self.rpc_callback);

        thread::spawn(move || {
            println!("🦀 [ZMQ-API-CONTROL] Processing loop started");

            while *running.lock() {
                let sock_guard = socket.lock();
                let sock = match sock_guard.as_ref() {
                    Some(s) => s,
                    None => {
                        drop(sock_guard);
                        thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }
                };

                // Poll for messages
                let poll_items = &mut [sock.as_poll_item(zmq::POLLIN)];
                if let Err(e) = zmq::poll(poll_items, 100) {
                    eprintln!("🦀 [ZMQ-API-CONTROL] [ERR] Poll error: {}", e);
                    continue;
                }

                if !poll_items[0].is_readable() {
                    drop(sock_guard);
                    continue;
                }

                // Receive multipart message: [identity, delimiter, request_json]
                let mut msg_parts = Vec::new();
                let mut more = true;

                while more {
                    let mut msg = zmq::Message::new();
                    match sock.recv(&mut msg, 0) {
                        Ok(()) => {
                            msg_parts.push(msg);
                            more = sock.get_rcvmore().unwrap_or(false);
                        }
                        Err(e) => {
                            eprintln!("🦀 [ZMQ-API-CONTROL] [ERR] Receive error: {}", e);
                            break;
                        }
                    }
                }

                drop(sock_guard);

                // Process request
                if msg_parts.len() >= 3 {
                    let identity = msg_parts[0].to_vec();
                    let request_json = String::from_utf8_lossy(&msg_parts[2]).to_string();
                    
                    println!("🦀 [ZMQ-API-CONTROL] 📨 Received request ({} bytes)", request_json.len());
                    println!("🦀 [ZMQ-API-CONTROL] 📨 Request: {}", &request_json[..request_json.len().min(200)]);

                    let response_json = Self::process_request(&npu, &rpc_callback, &request_json);
                    
                    println!("🦀 [ZMQ-API-CONTROL] 📤 Sending response ({} bytes)", response_json.len());

                    if let Err(e) = Self::send_response(&socket, identity, response_json) {
                        eprintln!("🦀 [ZMQ-API-CONTROL] [ERR] Failed to send response: {}", e);
                    }
                } else {
                    println!("🦀 [ZMQ-API-CONTROL] ⚠️ Received malformed message (parts: {})", msg_parts.len());
                }
            }

            println!("🦀 [ZMQ-API-CONTROL] Processing loop stopped");
        });
    }

    /// Process a request using the NPU
    fn process_request(
        npu_mutex: &Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>>>>,
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

    /// Route API request to appropriate handler
    fn route_request(
        npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>,
        rpc_callback: &RpcCallback,
        request: &ApiRequest,
    ) -> serde_json::Value {
        // Log high-level request (for debugging)
        static FIRST_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            println!("🦀 [API-CONTROL] First request: {} {}", request.method, request.path);
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Route to handlers
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

    /// Handle NPU stats query
    fn handle_npu_stats(npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) -> serde_json::Value {
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

    /// Handle cortical areas list query
    fn handle_cortical_areas(npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) -> serde_json::Value {
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

    /// Handle cortical area info query
    fn handle_cortical_area_info(_npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>, path: &str) -> serde_json::Value {
        let area_name = path.strip_prefix("/v1/npu/cortical_area/").unwrap_or("");
        
        serde_json::json!({
            "status": 501,
            "body": {
                "error": format!("Cortical area info for '{}' not yet implemented", area_name)
            }
        })
    }

    /// Handle fire queue query
    fn handle_fire_queue(_npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) -> serde_json::Value {
        serde_json::json!({
            "status": 501,
            "body": {"error": "Fire queue query not yet implemented"}
        })
    }

    /// Handle health check
    fn handle_health_check() -> serde_json::Value {
        serde_json::json!({
            "status": 200,
            "body": {
                "status": "ok",
                "service": "API Control Stream"
            }
        })
    }

    /// Handle brain readiness query
    fn handle_brain_readiness(npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        // Brain is ready if burst loop has started (burst_count > 0) or neurons exist
        let is_ready = npu_lock.get_burst_count() > 0 || npu_lock.get_neuron_count() > 0;
        serde_json::json!({
            "status": 200,
            "body": {"value": is_ready}
        })
    }

    /// Handle burst engine state query
    fn handle_burst_engine_state(npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        // 0=NOT_STARTED, 2=READY (if burst count > 0 or neurons exist)
        let state = if npu_lock.get_burst_count() > 0 || npu_lock.get_neuron_count() > 0 { 2 } else { 0 };
        serde_json::json!({
            "status": 200,
            "body": {"value": state}
        })
    }

    /// Handle genome state query
    fn handle_genome_state(npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        let state = if npu_lock.get_registered_cortical_area_count() > 0 { 2 } else { 0 }; // 0=NOT_LOADED, 2=LOADED
        serde_json::json!({
            "status": 200,
            "body": {"value": state}
        })
    }

    /// Handle brain stats query
    fn handle_brain_stats(npu: &Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) -> serde_json::Value {
        let npu_lock = npu.lock().unwrap();
        serde_json::json!({
            "status": 200,
            "body": {
                "neuron_count": npu_lock.get_neuron_count(),
                "synapse_count": npu_lock.get_synapse_count(),
                "cortical_area_count": npu_lock.get_registered_cortical_area_count(),
                "memory_neuron_count": 0, // TODO: Track memory neurons
                "non_memory_neuron_count": npu_lock.get_neuron_count(),
            }
        })
    }

    /// Handle generic RPC call to Python CoreAPIService
    fn handle_rpc(rpc_callback: &RpcCallback, request: &ApiRequest) -> serde_json::Value {
        println!("🦀 [API-CONTROL-RPC] Received RPC request");
        
        // Extract RPC payload from request body
        let rpc_payload = match &request.body {
            Some(body) => {
                println!("🦀 [API-CONTROL-RPC] Request has body: {:?}", body);
                body.clone()
            }
            None => {
                println!("🦀 [API-CONTROL-RPC] ERROR: Request missing body");
                return serde_json::json!({
                    "status": 400,
                    "body": {"error": "RPC request missing body"}
                });
            }
        };

        // Extract method name from RPC payload
        let method_name = match rpc_payload.get("method").and_then(|m| m.as_str()) {
            Some(m) => {
                println!("🦀 [API-CONTROL-RPC] Method name: {}", m);
                m.to_string()
            }
            None => {
                println!("🦀 [API-CONTROL-RPC] ERROR: Payload missing 'method' field");
                return serde_json::json!({
                    "status": 400,
                    "body": {"error": "RPC payload missing 'method' field"}
                });
            }
        };

        // Get callback
        let callback_guard = rpc_callback.lock();
        let callback = match callback_guard.as_ref() {
            Some(cb) => {
                println!("🦀 [API-CONTROL-RPC] Callback registered, calling Python handler");
                cb
            }
            None => {
                println!("🦀 [API-CONTROL-RPC] ERROR: No RPC callback registered");
                return serde_json::json!({
                    "status": 503,
                    "body": {"error": "RPC callback not registered"}
                });
            }
        };

        // Call Python CoreAPIService via callback
        println!("🦀 [API-CONTROL-RPC] Invoking Python handler for method: {}", method_name);
        match callback(&method_name, rpc_payload) {
            Ok(result) => {
                println!("🦀 [API-CONTROL-RPC] Python handler returned success");
                serde_json::json!({
                    "status": 200,
                    "body": result
                })
            }
            Err(e) => {
                println!("🦀 [API-CONTROL-RPC] Python handler returned error: {}", e);
                serde_json::json!({
                    "status": 500,
                    "body": {"error": e}
                })
            }
        }
    }

    /// Send response
    fn send_response(
        socket_mutex: &Arc<Mutex<Option<zmq::Socket>>>,
        identity: Vec<u8>,
        response_json: String,
    ) -> Result<(), String> {
        let sock_guard = socket_mutex.lock();
        let sock = match sock_guard.as_ref() {
            Some(s) => s,
            None => return Err("Socket not available".to_string()),
        };

        sock.send(&identity, zmq::SNDMORE)
            .map_err(|e| e.to_string())?;
        sock.send(&Vec::<u8>::new(), zmq::SNDMORE)
            .map_err(|e| e.to_string())?;
        sock.send(response_json.as_bytes(), 0)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}


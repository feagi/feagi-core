// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ Transport Adapter for FEAGI API
//!
//! This adapter uses feagi-transports to handle ZMQ communication and routes
//! requests to the unified endpoint layer. It provides an alternative to HTTP
//! for control plane communication.

use crate::common::{ApiError, ApiRequest, ApiResponse};
use crate::endpoints;
use crate::transports::http::server::ApiState;
use feagi_transports::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;

/// ZMQ Transport Adapter for the API
pub struct ZmqApiAdapter {
    /// ZMQ ROUTER transport from feagi-transports
    router: Arc<Mutex<Option<ZmqRouter>>>,

    /// API state with all services
    state: ApiState,

    /// Running flag
    running: Arc<Mutex<bool>>,
}

impl ZmqApiAdapter {
    /// Create a new ZMQ API adapter
    pub fn new(
        context: Arc<zmq::Context>,
        bind_address: &str,
        state: ApiState,
    ) -> Result<Self, String> {
        // Create transport config
        let config = ServerConfig::new(bind_address)
            .base
            .with_recv_hwm(10000)
            .with_send_hwm(10000)
            .with_linger(std::time::Duration::from_secs(1));

        let server_config = ServerConfig {
            base: config,
            max_connections: 0,
            track_connections: true,
        };

        // Create ZMQ router using feagi-transports
        let router = ZmqRouter::new(context, server_config)
            .map_err(|e| format!("Failed to create ZMQ router: {}", e))?;

        Ok(Self {
            router: Arc::new(Mutex::new(Some(router))),
            state,
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Start the ZMQ adapter
    pub fn start(&self) -> Result<(), String> {
        if *self.running.lock() {
            return Err("ZMQ adapter already running".to_string());
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

        info!("🦀 [ZMQ-API] Adapter started (using feagi-transports)");

        // Start request handling loop
        self.start_request_loop();

        Ok(())
    }

    /// Stop the ZMQ adapter
    pub fn stop(&self) -> Result<(), String> {
        *self.running.lock() = false;

        let mut router_guard = self.router.lock();
        if let Some(router) = router_guard.as_mut() {
            router.stop().map_err(|e| e.to_string())?;
        }
        *router_guard = None;

        info!("🦀 [ZMQ-API] Adapter stopped");

        Ok(())
    }

    /// Start the background request handling loop
    fn start_request_loop(&self) {
        let router = Arc::clone(&self.router);
        let state = self.state.clone();
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            info!("🦀 [ZMQ-API] Request loop started");

            while *running.lock() {
                // Receive request with timeout
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
                        // Parse API request
                        let api_request: ApiRequest = match serde_json::from_slice(&request_data) {
                            Ok(req) => req,
                            Err(e) => {
                                let error_response = ApiResponse::<()>::error(
                                    ApiError::bad_request(&format!("Invalid request: {}", e)),
                                );
                                if let Ok(response_json) = serde_json::to_vec(&error_response) {
                                    let _ = reply_handle.send(&response_json);
                                }
                                continue;
                            }
                        };

                        info!("🦀 [ZMQ-API] {} {}", api_request.method, api_request.path);

                        // Route to endpoint handlers
                        let api_response = Self::route_request(&api_request, &state);

                        // Send response
                        match serde_json::to_vec(&api_response) {
                            Ok(response_json) => {
                                if let Err(e) = reply_handle.send(&response_json) {
                                    error!("🦀 [ZMQ-API] [ERR] Failed to send response: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("🦀 [ZMQ-API] [ERR] Failed to serialize response: {}", e);
                            }
                        }
                    }
                    Err(TransportError::Timeout) => {
                        // Timeout is normal, just continue
                        continue;
                    }
                    Err(e) => {
                        error!("🦀 [ZMQ-API] [ERR] Receive error: {}", e);
                    }
                }
            }

            info!("🦀 [ZMQ-API] Request loop stopped");
        });
    }

    /// Route API request to appropriate endpoint handler
    fn route_request(request: &ApiRequest, state: &ApiState) -> ApiResponse<serde_json::Value> {
        // Route based on method and path
        match (request.method.as_str(), request.path.as_str()) {
            // Health check
            ("GET", "/v1/health") => {
                let result = endpoints::health::health_check(services);
                Self::convert_result(result)
            }

            // Cortical areas
            ("GET", "/v1/cortical_areas") => {
                let result = endpoints::cortical_areas::list_cortical_areas(services);
                Self::convert_result(result)
            }
            ("GET", path) if path.starts_with("/v1/cortical_area/") => {
                let area_id = path.strip_prefix("/v1/cortical_area/").unwrap_or("");
                let result = endpoints::cortical_areas::get_cortical_area(services, area_id);
                Self::convert_result(result)
            }

            // Brain regions
            ("GET", "/v1/brain_regions") => {
                let result = endpoints::brain_regions::list_brain_regions(services);
                Self::convert_result(result)
            }
            ("GET", path) if path.starts_with("/v1/brain_region/") => {
                let region_id = path.strip_prefix("/v1/brain_region/").unwrap_or("");
                let result = endpoints::brain_regions::get_brain_region(services, region_id);
                Self::convert_result(result)
            }

            // Runtime control
            ("GET", "/v1/runtime/status") => {
                let result = endpoints::runtime::get_runtime_status(services);
                Self::convert_result(result)
            }
            ("POST", "/v1/runtime/start") => {
                let result = endpoints::runtime::start_runtime(services);
                Self::convert_result(result)
            }
            ("POST", "/v1/runtime/stop") => {
                let result = endpoints::runtime::stop_runtime(services);
                Self::convert_result(result)
            }

            // Analytics
            ("GET", "/v1/system/health") => {
                let result = endpoints::analytics::get_system_health(services);
                Self::convert_result(result)
            }

            // Not found
            _ => ApiResponse::error(ApiError::not_found(&format!(
                "Endpoint not found: {} {}",
                request.method, request.path
            ))),
        }
    }

    /// Convert endpoint result to API response with JSON value
    fn convert_result<T: serde::Serialize>(
        result: Result<ApiResponse<T>, ApiError>,
    ) -> ApiResponse<serde_json::Value> {
        match result {
            Ok(response) => {
                // Convert data to JSON value
                match serde_json::to_value(&response.data) {
                    Ok(json_value) => ApiResponse {
                        success: response.success,
                        data: json_value,
                        error: response.error,
                        timestamp: response.timestamp,
                    },
                    Err(e) => ApiResponse::error(ApiError::internal_error(&format!(
                        "Failed to serialize response: {}",
                        e
                    ))),
                }
            }
            Err(error) => ApiResponse::error(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zmq_adapter_creation() {
        let context = Arc::new(zmq::Context::new());
        let services = Arc::new(ServiceRegistry::default());

        let adapter = ZmqApiAdapter::new(context, "tcp://127.0.0.1:32000", services);

        assert!(adapter.is_ok());
    }
}

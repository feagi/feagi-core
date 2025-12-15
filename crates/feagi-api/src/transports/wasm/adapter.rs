// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM API Adapter
//!
//! Routes REST API calls to transport-agnostic endpoint functions.
//! NO DUPLICATION - reuses the same endpoint logic as HTTP and ZMQ adapters.
//!
//! NOTE: Currently endpoints use axum types directly, so this adapter can only
//! work when http feature is enabled. To make it work without http feature,
//! endpoints need to use type aliases instead of axum types directly.

use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;
use serde_json::Value;

// Use axum types when http feature is enabled
#[cfg(feature = "http")]
use axum::{extract::State, response::Json};

// Use WASM-compatible types when http feature is disabled
#[cfg(not(feature = "http"))]
use crate::transports::wasm::types::{State, Json};

/// WASM Transport Adapter
///
/// Routes REST API calls to transport-agnostic endpoint functions.
/// Returns JSON strings matching HTTP REST API responses exactly.
pub struct WasmApiAdapter {
    /// API state with all services
    state: ApiState,
}

impl WasmApiAdapter {
    /// Create a new WASM API adapter
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }

    /// Handle REST API call
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, PUT, DELETE)
    /// * `path` - API path (e.g., "/v1/cortical_area/cortical_area/geometry")
    /// * `body` - Request body (JSON string, empty for GET)
    ///
    /// # Returns
    /// * `String` - JSON response string (matches HTTP format exactly)
    ///
    /// # Errors
    /// * Returns error JSON string if endpoint not found or request fails
    pub async fn handle_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<String, String> {
        // Import endpoints module - now works with or without http feature!
        use crate::endpoints;
        
        // Route to appropriate endpoint function (reuses endpoint logic!)
        let result: ApiResult<serde_json::Value> = match (method, path) {
            // ========================================================================
            // SYSTEM ENDPOINTS - REUSING endpoint logic
            // ========================================================================
            ("GET", "/v1/system/health_check") => {
                endpoints::system::get_health_check(State(self.state.clone())).await
                    .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // CORTICAL AREA ENDPOINTS - REUSING endpoint logic
            // ========================================================================
            ("GET", "/v1/cortical_area/cortical_area/geometry") => {
                endpoints::cortical_area::get_cortical_area_geometry(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            ("GET", "/v1/cortical_area/ipu/types") => {
                endpoints::cortical_area::get_ipu_types(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            ("GET", "/v1/cortical_area/opu/types") => {
                endpoints::cortical_area::get_opu_types(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            ("GET", "/v1/cortical_area/cortical_area_id_list") => {
                endpoints::cortical_area::get_cortical_area_id_list(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            ("GET", "/v1/cortical_area/cortical_area_name_list") => {
                endpoints::cortical_area::get_cortical_area_name_list(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // MORPHOLOGY ENDPOINTS - REUSING endpoint logic
            // ========================================================================
            ("GET", "/v1/morphology/morphologies") => {
                endpoints::morphology::get_morphologies(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // MAPPING ENDPOINTS - REUSING endpoint logic
            // ========================================================================
            ("GET", "/v1/cortical_area/cortical_map_detailed") => {
                endpoints::cortical_area::get_cortical_map_detailed(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // REGION ENDPOINTS - REUSING endpoint logic
            // ========================================================================
            ("GET", "/v1/region/regions_members") => {
                endpoints::region::get_regions_members(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // GENOME ENDPOINTS - REUSING endpoint logic
            // ========================================================================
            ("POST", "/v1/genome/save") => {
                let payload: std::collections::HashMap<String, String> = match serde_json::from_str(body.unwrap_or("{}")) {
                    Ok(p) => p,
                    Err(e) => return Err(format!("Invalid JSON: {}", e)),
                };
                endpoints::genome::post_save(
                    State(self.state.clone()),
                    Json(payload),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            ("GET", "/v1/genome/file_name") => {
                endpoints::genome::get_file_name(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // BURST ENGINE ENDPOINTS - REUSING endpoint logic
            // ========================================================================
            ("GET", "/v1/burst_engine/simulation_timestep") => {
                endpoints::burst_engine::get_simulation_timestep(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // AGENT ENDPOINTS - REUSING endpoint logic
            // ========================================================================
            ("GET", "/v1/agent/list") => {
                endpoints::agent::list_agents(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            ("POST", "/v1/agent/manual_stimulation") => {
                use crate::v1::agent_dtos::ManualStimulationRequest;
                let payload: ManualStimulationRequest = match serde_json::from_str(body.unwrap_or("{}")) {
                    Ok(p) => p,
                    Err(e) => return Err(format!("Invalid JSON: {}", e)),
                };
                endpoints::agent::manual_stimulation(
                    State(self.state.clone()),
                    Json(payload),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // UNKNOWN ENDPOINT
            // ========================================================================
            _ => {
                return Err(format!(
                    "Unknown endpoint: {} {}",
                    method, path
                ));
            }
        };

        // Serialize result to JSON string
        match result {
            Ok(data) => {
                serde_json::to_string_pretty(&data)
                    .map_err(|e| format!("Serialization error: {}", e))
            }
            Err(e) => {
                // Convert ApiError to error JSON
                let error_response = serde_json::json!({
                    "error": true,
                    "message": e.message.clone(),
                    "code": e.code,
                });
                serde_json::to_string_pretty(&error_response)
                    .map_err(|e| format!("Serialization error: {}", e))
            }
        }
    }
    
}

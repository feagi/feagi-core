// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM API Adapter
//!
//! Routes REST API calls to transport-agnostic endpoint functions.
//! No duplication - uses the same endpoint logic as HTTP and ZMQ adapters.

use crate::common::{ApiError, ApiResult};
use crate::endpoints;
use crate::transports::http::server::ApiState;
use axum::extract::State;
use serde_json::Value;
use std::collections::HashMap;

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
        // Route to appropriate endpoint function
        let result: ApiResult<serde_json::Value> = match (method, path) {
            // ========================================================================
            // SYSTEM ENDPOINTS
            // ========================================================================
            ("GET", "/v1/system/health_check") => {
                endpoints::system::get_health_check(State(self.state.clone())).await
                    .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // CORTICAL AREA ENDPOINTS
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
            // MORPHOLOGY ENDPOINTS
            // ========================================================================
            ("GET", "/v1/morphology/morphologies") => {
                endpoints::morphology::get_morphologies(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // MAPPING ENDPOINTS
            // ========================================================================
            ("GET", "/v1/cortical_area/cortical_map_detailed") => {
                endpoints::cortical_area::get_cortical_map_detailed(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // REGION ENDPOINTS
            // ========================================================================
            ("GET", "/v1/region/regions_members") => {
                endpoints::region::get_regions_members(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // GENOME ENDPOINTS
            // ========================================================================
            ("POST", "/v1/genome/save") => {
                let payload: std::collections::HashMap<String, String> = match serde_json::from_str(body.unwrap_or("{}")) {
                    Ok(p) => p,
                    Err(e) => return Err(format!("Invalid JSON: {}", e)),
                };
                endpoints::genome::post_save(
                    State(self.state.clone()),
                    axum::extract::Json(payload),
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
            // BURST ENGINE ENDPOINTS
            // ========================================================================
            // Note: Burst processing is handled directly by FeagiEngine in feagi-wasm
            // This endpoint is for compatibility but may not be fully implemented

            ("GET", "/v1/burst_engine/simulation_timestep") => {
                endpoints::burst_engine::get_simulation_timestep(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            // ========================================================================
            // AGENT ENDPOINTS
            // ========================================================================
            ("GET", "/v1/agent/list") => {
                endpoints::agent::list_agents(
                    State(self.state.clone()),
                )
                .await
                .map(|json| serde_json::to_value(json.0).unwrap_or(serde_json::Value::Null))
            }

            ("POST", "/v1/agent/manual_stimulation") => {
                let payload: crate::v1::agent_dtos::ManualStimulationRequest = match serde_json::from_str(body.unwrap_or("{}")) {
                    Ok(p) => p,
                    Err(e) => return Err(format!("Invalid JSON: {}", e)),
                };
                endpoints::agent::manual_stimulation(
                    State(self.state.clone()),
                    axum::extract::Json(payload),
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


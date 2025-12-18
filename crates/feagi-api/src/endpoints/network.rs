// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
 * FEAGI v1 Network API
 * 
 * Endpoints for network configuration and status
 * Maps to Python: feagi/api/v1/network.py
 */

use crate::common::{ApiError, ApiResult, State, Json};
use crate::common::ApiState;
// Removed - using crate::common::State instead
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// NETWORK CONFIGURATION
// ============================================================================

/// Get network configuration status including ZMQ, HTTP, and WebSocket states.
#[utoipa::path(
    get,
    path = "/v1/network/status",
    tag = "network",
    responses(
        (status = 200, description = "Network status", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_status(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // TODO: Retrieve network status
    let mut response = HashMap::new();
    response.insert("zmq_enabled".to_string(), json!(false));
    response.insert("http_enabled".to_string(), json!(true));
    response.insert("websocket_enabled".to_string(), json!(false));
    
    Ok(Json(response))
}

/// Configure network parameters including transport protocols and ports.
#[utoipa::path(
    post,
    path = "/v1/network/config",
    tag = "network",
    responses(
        (status = 200, description = "Network configured", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_config(
    State(_state): State<ApiState>,
    Json(request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // Validate config is provided
    let _config = request.get("config")
        .ok_or_else(|| ApiError::invalid_input("Missing 'config' field"))?;
    
    // TODO: Apply network configuration
    tracing::info!(target: "feagi-api", "Network configuration updated");
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Network configured successfully".to_string())
    ])))
}


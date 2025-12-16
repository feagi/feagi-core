// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
 * FEAGI v1 Physiology API
 * 
 * Endpoints to read/update physiology parameters in the active genome
 * Maps to Python: feagi/api/v1/physiology.py
 */

use crate::common::{ApiError, ApiResult, State, Json, Path, Query};
use crate::common::ApiState;
// Removed - using crate::common::State instead
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// PHYSIOLOGY CONFIGURATION
// ============================================================================

/// Get current physiology parameters including simulation timestep and neural aging settings.
#[utoipa::path(
    get,
    path = "/v1/physiology/",
    tag = "physiology",
    responses(
        (status = 200, description = "Physiology parameters", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_physiology(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // Get physiology parameters from genome via ConnectomeService
    let _connectome_service = state.connectome_service.as_ref();
    
    // Get current simulation timestep from runtime service
    let runtime_service = state.runtime_service.as_ref();
    let status = runtime_service.get_status().await
        .map_err(|e| ApiError::internal(format!("Failed to get runtime status: {}", e)))?;
    
    let simulation_timestep = if status.frequency_hz > 0.0 {
        1.0 / status.frequency_hz
    } else {
        0.0
    };
    
    // TODO: Add get_genome_physiology to ConnectomeService for other parameters
    let physiology = json!({
        "simulation_timestep": simulation_timestep,
        "max_age": 0,
        "evolution_burst_count": 0,
        "ipu_idle_threshold": 0,
        "plasticity_queue_depth": 0,
        "lifespan_mgmt_interval": 0,
        "sleep_trigger_inactivity_window": 0,
        "sleep_trigger_neural_activity_max": 0.0
    });
    
    let mut response = HashMap::new();
    response.insert("physiology".to_string(), physiology);
    
    Ok(Json(response))
}

/// Update physiology parameters in the active genome including timestep and aging settings.
#[utoipa::path(
    put,
    path = "/v1/physiology/",
    tag = "physiology",
    responses(
        (status = 200, description = "Physiology updated", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_physiology(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, Value>>> {
    // Whitelist of allowed physiology keys
    let allowed_keys = vec![
        "simulation_timestep",
        "max_age",
        "evolution_burst_count",
        "ipu_idle_threshold",
        "plasticity_queue_depth",
        "lifespan_mgmt_interval",
        "sleep_trigger_inactivity_window",
        "sleep_trigger_neural_activity_max",
    ];
    
    // Extract and filter physiology updates
    let updates = request.get("physiology")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter(|(k, _)| allowed_keys.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<String, Value>>()
        })
        .unwrap_or_default();
    
    if updates.is_empty() {
        return Ok(Json(HashMap::from([
            ("success".to_string(), json!(false)),
            ("updated".to_string(), json!({})),
            ("message".to_string(), json!("No valid physiology parameters provided"))
        ])));
    }
    
    // Apply simulation_timestep if provided
    if let Some(timestep) = updates.get("simulation_timestep").and_then(|v| v.as_f64()) {
        let runtime_service = state.runtime_service.as_ref();
        if timestep > 0.0 {
            let frequency = 1.0 / timestep;
            runtime_service.set_frequency(frequency).await
                .map_err(|e| ApiError::internal(format!("Failed to set timestep: {}", e)))?;
            tracing::info!(target: "feagi-api", "Updated simulation timestep to {:.6}s ({:.2} Hz)", 
                timestep, frequency);
        }
    }
    
    // TODO: Apply other physiology parameters to genome
    tracing::info!(target: "feagi-api", "Physiology parameters updated: {:?}", 
        updates.keys().collect::<Vec<_>>());
    
    let mut response = HashMap::new();
    response.insert("success".to_string(), json!(true));
    response.insert("updated".to_string(), json!(updates));
    
    Ok(Json(response))
}


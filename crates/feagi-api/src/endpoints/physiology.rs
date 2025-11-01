/*!
 * FEAGI v1 Physiology API
 * 
 * Endpoints to read/update physiology parameters in the active genome
 * Maps to Python: feagi/api/v1/physiology.py
 */

use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// PHYSIOLOGY CONFIGURATION
// ============================================================================

/// GET /v1/physiology/
/// Get current physiology parameters from genome
#[utoipa::path(
    get,
    path = "/v1/physiology/",
    tag = "physiology",
    responses(
        (status = 200, description = "Physiology parameters", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_physiology(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // TODO: Retrieve physiology from active genome
    let mut response = HashMap::new();
    response.insert("physiology".to_string(), json!({}));
    
    Ok(Json(response))
}

/// PUT /v1/physiology/
/// Update physiology parameters in active genome
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
    State(_state): State<ApiState>,
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
    
    // TODO: Apply updates to active genome and persist
    
    let mut response = HashMap::new();
    response.insert("success".to_string(), json!(true));
    response.insert("updated".to_string(), json!(updates));
    
    Ok(Json(response))
}


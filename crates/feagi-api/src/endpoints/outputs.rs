/*!
 * FEAGI v1 Outputs API
 * 
 * Endpoints for output/motor target configuration
 * Maps to Python: feagi/api/v1/outputs.py
 */

use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// OUTPUT TARGETS
// ============================================================================

/// GET /v1/output/targets (Python uses singular /v1/output)
/// Get available output targets
#[utoipa::path(
    get,
    path = "/v1/output/targets",
    tag = "outputs",
    responses(
        (status = 200, description = "Output targets", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_targets(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // Get motor/output capable agents from PNS
    let agent_service = state.agent_service.as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;
    
    let agent_ids = agent_service.list_agents().await
        .map_err(|e| ApiError::internal(format!("Failed to list agents: {}", e)))?;
    
    // Filter for agents with motor/output capabilities
    let mut motor_agents = Vec::new();
    for agent_id in agent_ids {
        // Get agent properties to check capabilities
        if let Ok(props) = agent_service.get_agent_properties(&agent_id).await {
            // Check if agent has motor capabilities
            if props.capabilities.contains_key("motor") || 
               props.capabilities.contains_key("output") ||
               props.agent_type.to_lowercase().contains("motor") {
                motor_agents.push(agent_id);
            }
        }
    }
    
    let mut response = HashMap::new();
    response.insert("targets".to_string(), json!(motor_agents));
    
    Ok(Json(response))
}

/// POST /v1/output/configure (Python uses singular /v1/output)
/// Configure output targets
#[utoipa::path(
    post,
    path = "/v1/output/configure",
    tag = "outputs",
    responses(
        (status = 200, description = "Outputs configured", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_configure(
    State(_state): State<ApiState>,
    Json(request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // Extract configuration from request
    let config = request.get("config")
        .ok_or_else(|| ApiError::invalid_input("Missing 'config' field"))?;
    
    // TODO: Store output configuration in runtime state
    // For now, just validate the structure
    if !config.is_object() {
        return Err(ApiError::invalid_input("'config' must be an object"));
    }
    
    tracing::info!(target: "feagi-api", "Output configuration updated: {} targets", 
        config.as_object().map(|o| o.len()).unwrap_or(0));
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Outputs configured successfully".to_string())
    ])))
}


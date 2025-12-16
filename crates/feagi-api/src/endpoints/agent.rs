// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Agent API endpoints - Exact port from Python `/v1/agent/*` routes
//!
//! These endpoints match the Python implementation at:
//! feagi-py/feagi/api/v1/feagi_agent.py

use std::collections::HashMap;

use crate::common::{ApiError, ApiResult, State, Json, Query, Path};
use crate::common::ApiState;
use crate::v1::agent_dtos::*;
use feagi_services::traits::agent_service::{
    AgentRegistration, HeartbeatRequest as ServiceHeartbeatRequest,
};

/// POST /v1/agent/register
/// 
/// Register a new agent with FEAGI
#[utoipa::path(
    post,
    path = "/v1/agent/register",
    request_body = AgentRegistrationRequest,
    responses(
        (status = 200, description = "Agent registered successfully", body = AgentRegistrationResponse),
        (status = 500, description = "Registration failed", body = String)
    ),
    tag = "agent"
)]
pub async fn register_agent(
    State(state): State<ApiState>,
    Json(request): Json<AgentRegistrationRequest>,
) -> ApiResult<Json<AgentRegistrationResponse>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    let registration = AgentRegistration {
        agent_id: request.agent_id.clone(),
        agent_type: request.agent_type,
        agent_data_port: request.agent_data_port,
        agent_version: request.agent_version,
        controller_version: request.controller_version,
        agent_ip: request.agent_ip,
        capabilities: request.capabilities,
        metadata: request.metadata,
        chosen_transport: request.chosen_transport,
    };

    match agent_service.register_agent(registration).await {
        Ok(response) => {
            // Convert service TransportConfig to API TransportConfig
            let transports = response.transports.map(|ts| {
                ts.into_iter().map(|t| {
                    crate::v1::agent_dtos::TransportConfig {
                        transport_type: t.transport_type,
                        enabled: t.enabled,
                        ports: t.ports,
                        host: t.host,
                    }
                }).collect()
            });
            
            Ok(Json(AgentRegistrationResponse {
                status: response.status,
                message: response.message,
                success: response.success,
                transport: response.transport,
                rates: response.rates,
                transports,
                recommended_transport: response.recommended_transport,
                zmq_ports: response.zmq_ports,
                shm_paths: response.shm_paths,
                cortical_areas: response.cortical_areas,
            }))
        },
        Err(e) => {
            // Check if error is about unsupported transport (validation error)
            let error_msg = e.to_string();
            if error_msg.contains("not supported") || error_msg.contains("disabled") {
                Err(ApiError::invalid_input(error_msg))
            } else {
                Err(ApiError::internal(format!("Registration failed: {}", e)))
            }
        }
    }
}

/// POST /v1/agent/heartbeat
///
/// Record a heartbeat to keep agent registered
#[utoipa::path(
    post,
    path = "/v1/agent/heartbeat",
    request_body = HeartbeatRequest,
    responses(
        (status = 200, description = "Heartbeat recorded", body = HeartbeatResponse),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Heartbeat failed")
    ),
    tag = "agent"
)]
pub async fn heartbeat(
    State(state): State<ApiState>,
    Json(request): Json<HeartbeatRequest>,
) -> ApiResult<Json<HeartbeatResponse>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    let service_request = ServiceHeartbeatRequest {
        agent_id: request.agent_id.clone(),
    };

    match agent_service.heartbeat(service_request).await {
        Ok(_) => Ok(Json(HeartbeatResponse {
            message: "heartbeat_ok".to_string(),
            success: true,
        })),
        Err(e) => Err(ApiError::not_found("agent", &format!("{}", e))),
    }
}

/// GET /v1/agent/list
///
/// List all registered agents
#[utoipa::path(
    get,
    path = "/v1/agent/list",
    responses(
        (status = 200, description = "List of agent IDs", body = Vec<String>),
        (status = 503, description = "Registration service unavailable")
    ),
    tag = "agent"
)]
pub async fn list_agents(
    State(state): State<ApiState>,
) -> ApiResult<Json<Vec<String>>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    match agent_service.list_agents().await {
        Ok(agent_ids) => Ok(Json(agent_ids)),
        Err(e) => Err(ApiError::internal(format!(
            "Failed to list agents: {}",
            e
        ))),
    }
}

/// GET /v1/agent/properties
///
/// Get properties for a specific agent (query parameter version)
#[utoipa::path(
    get,
    path = "/v1/agent/properties",
    params(
        ("agent_id" = String, Query, description = "Agent ID to get properties for")
    ),
    responses(
        (status = 200, description = "Agent properties", body = AgentPropertiesResponse),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Failed to get agent properties")
    ),
    tag = "agent"
)]
pub async fn get_agent_properties(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<AgentPropertiesResponse>> {
    let agent_id = params
        .get("agent_id")
        .ok_or_else(|| ApiError::invalid_input("Missing agent_id query parameter"))?;

    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    match agent_service.get_agent_properties(agent_id).await {
        Ok(properties) => Ok(Json(AgentPropertiesResponse {
            agent_type: properties.agent_type,
            agent_ip: properties.agent_ip,
            agent_data_port: properties.agent_data_port,
            agent_router_address: properties.agent_router_address,
            agent_version: properties.agent_version,
            controller_version: properties.controller_version,
            capabilities: properties.capabilities,
            chosen_transport: properties.chosen_transport,
        })),
        Err(e) => Err(ApiError::not_found("agent", &format!("{}", e))),
    }
}

/// GET /v1/agent/shared_mem
///
/// Get shared memory information for all agents
#[utoipa::path(
    get,
    path = "/v1/agent/shared_mem",
    responses(
        (status = 200, description = "Shared memory info", body = HashMap<String, HashMap<String, serde_json::Value>>),
        (status = 500, description = "Failed to get shared memory info")
    ),
    tag = "agent"
)]
pub async fn get_shared_memory(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, HashMap<String, serde_json::Value>>>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    match agent_service.get_shared_memory_info().await {
        Ok(shm_info) => Ok(Json(shm_info)),
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get shared memory info: {}",
            e
        ))),
    }
}

/// DELETE /v1/agent/deregister
///
/// Deregister an agent (body-based, not query parameter)
#[utoipa::path(
    delete,
    path = "/v1/agent/deregister",
    request_body = AgentDeregistrationRequest,
    responses(
        (status = 200, description = "Agent deregistered successfully", body = SuccessResponse),
        (status = 500, description = "Deregistration failed")
    ),
    tag = "agent"
)]
pub async fn deregister_agent(
    State(state): State<ApiState>,
    Json(request): Json<AgentDeregistrationRequest>,
) -> ApiResult<Json<SuccessResponse>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    match agent_service.deregister_agent(&request.agent_id).await {
        Ok(_) => Ok(Json(SuccessResponse {
            message: format!("Agent '{}' deregistered successfully", request.agent_id),
            success: Some(true),
        })),
        Err(e) => Err(ApiError::internal(format!("Deregistration failed: {}", e))),
    }
}

/// POST /v1/agent/manual_stimulation
///
/// Trigger manual neural stimulation across multiple cortical areas
#[utoipa::path(
    post,
    path = "/v1/agent/manual_stimulation",
    request_body = ManualStimulationRequest,
    responses(
        (status = 200, description = "Manual stimulation result", body = ManualStimulationResponse),
        (status = 500, description = "Stimulation failed")
    ),
    tag = "agent"
)]
pub async fn manual_stimulation(
    State(state): State<ApiState>,
    Json(request): Json<ManualStimulationRequest>,
) -> ApiResult<Json<ManualStimulationResponse>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    // Ensure runtime service is connected to agent service (if not already connected)
    // This allows runtime_service to be set after AgentServiceImpl is wrapped in Arc
    agent_service.try_set_runtime_service(state.runtime_service.clone());

    match agent_service
        .manual_stimulation(request.stimulation_payload)
        .await
    {
        Ok(result) => {
            let success = result
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let total_coordinates = result
                .get("total_coordinates")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            let successful_areas = result
                .get("successful_areas")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            let failed_areas = result
                .get("failed_areas")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let error = result
                .get("error")
                .and_then(|v| v.as_str())
                .map(String::from);

            Ok(Json(ManualStimulationResponse {
                success,
                total_coordinates,
                successful_areas,
                failed_areas,
                error,
            }))
        }
        Err(e) => Err(ApiError::internal(format!("Stimulation failed: {}", e))),
    }
}

/// GET /v1/agent/fq_sampler_status
///
/// Get comprehensive FQ sampler coordination status
#[utoipa::path(
    get,
    path = "/v1/agent/fq_sampler_status",
    responses(
        (status = 200, description = "FQ sampler status", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Failed to get FQ sampler status")
    ),
    tag = "agent"
)]
pub async fn get_fq_sampler_status(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;
    
    let runtime_service = state.runtime_service.as_ref();
    
    // Get all agents
    let agent_ids = agent_service.list_agents().await
        .map_err(|e| ApiError::internal(format!("Failed to list agents: {}", e)))?;
    
    // Get FCL sampler config from RuntimeService
    let (fcl_frequency, fcl_consumer) = runtime_service.get_fcl_sampler_config().await
        .map_err(|e| ApiError::internal(format!("Failed to get sampler config: {}", e)))?;
    
    // Build response matching Python structure
    let mut visualization_agents = Vec::new();
    let mut motor_agents = Vec::new();
    
    for agent_id in &agent_ids {
        if let Ok(props) = agent_service.get_agent_properties(agent_id).await {
            if props.capabilities.contains_key("visualization") {
                visualization_agents.push(agent_id.clone());
            }
            if props.capabilities.contains_key("motor") {
                motor_agents.push(agent_id.clone());
            }
        }
    }
    
    let mut fq_coordination = HashMap::new();
    
    let mut viz_sampler = HashMap::new();
    viz_sampler.insert("enabled".to_string(), serde_json::json!(!visualization_agents.is_empty()));
    viz_sampler.insert("reason".to_string(), serde_json::json!(
        if visualization_agents.is_empty() {
            "No visualization agents connected".to_string()
        } else {
            format!("{} visualization agent(s) connected", visualization_agents.len())
        }
    ));
    viz_sampler.insert("agents_requiring".to_string(), serde_json::json!(visualization_agents));
    viz_sampler.insert("frequency_hz".to_string(), serde_json::json!(fcl_frequency));
    fq_coordination.insert("visualization_fq_sampler".to_string(), serde_json::json!(viz_sampler));
    
    let mut motor_sampler = HashMap::new();
    motor_sampler.insert("enabled".to_string(), serde_json::json!(!motor_agents.is_empty()));
    motor_sampler.insert("reason".to_string(), serde_json::json!(
        if motor_agents.is_empty() {
            "No motor agents connected".to_string()
        } else {
            format!("{} motor agent(s) connected", motor_agents.len())
        }
    ));
    motor_sampler.insert("agents_requiring".to_string(), serde_json::json!(motor_agents));
    motor_sampler.insert("frequency_hz".to_string(), serde_json::json!(100.0));
    fq_coordination.insert("motor_fq_sampler".to_string(), serde_json::json!(motor_sampler));
    
    let mut response = HashMap::new();
    response.insert("fq_sampler_coordination".to_string(), serde_json::json!(fq_coordination));
    response.insert("agent_registry".to_string(), serde_json::json!({
        "total_agents": agent_ids.len(),
        "agent_ids": agent_ids
    }));
    response.insert("system_status".to_string(), serde_json::json!("coordinated_via_registration_manager"));
    response.insert("fcl_sampler_consumer".to_string(), serde_json::json!(fcl_consumer));
    
    Ok(Json(response))
}

/// GET /v1/agent/capabilities
///
/// List all supported agent capabilities
#[utoipa::path(
    get,
    path = "/v1/agent/capabilities",
    responses(
        (status = 200, description = "List of capabilities", body = HashMap<String, Vec<String>>),
        (status = 500, description = "Failed to get capabilities")
    ),
    tag = "agent"
)]
pub async fn get_capabilities(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    let mut response = HashMap::new();
    response.insert("agent_types".to_string(), vec![
        "sensory".to_string(),
        "motor".to_string(),
        "both".to_string(),
        "visualization".to_string(),
        "infrastructure".to_string(),
    ]);
    response.insert("capability_types".to_string(), vec![
        "vision".to_string(),
        "motor".to_string(),
        "visualization".to_string(),
        "sensory".to_string(),
    ]);
    
    Ok(Json(response))
}

/// GET /v1/agent/info/{agent_id}
///
/// Get detailed agent information
#[utoipa::path(
    get,
    path = "/v1/agent/info/{agent_id}",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Agent detailed info", body = HashMap<String, serde_json::Value>),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Failed to get agent info")
    ),
    tag = "agent"
)]
pub async fn get_agent_info(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;
    
    let properties = agent_service.get_agent_properties(&agent_id).await
        .map_err(|e| ApiError::not_found("agent", &e.to_string()))?;
    
    let mut response = HashMap::new();
    response.insert("agent_id".to_string(), serde_json::json!(agent_id));
    response.insert("agent_type".to_string(), serde_json::json!(properties.agent_type));
    response.insert("agent_ip".to_string(), serde_json::json!(properties.agent_ip));
    response.insert("agent_data_port".to_string(), serde_json::json!(properties.agent_data_port));
    response.insert("capabilities".to_string(), serde_json::json!(properties.capabilities));
    response.insert("agent_version".to_string(), serde_json::json!(properties.agent_version));
    response.insert("controller_version".to_string(), serde_json::json!(properties.controller_version));
    response.insert("status".to_string(), serde_json::json!("active"));
    if let Some(ref transport) = properties.chosen_transport {
        response.insert("chosen_transport".to_string(), serde_json::json!(transport));
    }
    
    Ok(Json(response))
}

/// GET /v1/agent/properties/{agent_id}
///
/// Get agent properties (path parameter version)
#[utoipa::path(
    get,
    path = "/v1/agent/properties/{agent_id}",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Agent properties", body = AgentPropertiesResponse),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Failed to get agent properties")
    ),
    tag = "agent"
)]
pub async fn get_agent_properties_path(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<AgentPropertiesResponse>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    match agent_service.get_agent_properties(&agent_id).await {
        Ok(properties) => Ok(Json(AgentPropertiesResponse {
            agent_type: properties.agent_type,
            agent_ip: properties.agent_ip,
            agent_data_port: properties.agent_data_port,
            agent_router_address: properties.agent_router_address,
            agent_version: properties.agent_version,
            controller_version: properties.controller_version,
            capabilities: properties.capabilities,
            chosen_transport: properties.chosen_transport,
        })),
        Err(e) => Err(ApiError::not_found("agent", &format!("{}", e))),
    }
}

/// POST /v1/agent/configure
///
/// Configure agent parameters
#[utoipa::path(
    post,
    path = "/v1/agent/configure",
    responses(
        (status = 200, description = "Agent configured", body = HashMap<String, String>),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Failed to configure agent")
    ),
    tag = "agent"
)]
pub async fn post_configure(
    State(_state): State<ApiState>,
    Json(config): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    tracing::info!(target: "feagi-api", "Agent configuration requested: {} params", config.len());
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Agent configuration updated".to_string()),
        ("status".to_string(), "not_yet_implemented".to_string())
    ])))
}


// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Agent API endpoints - Exact port from Python `/v1/agent/*` routes
//!
//! These endpoints match the Python implementation at:
//! feagi-py/feagi/api/v1/feagi_agent.py

use axum::{
    extract::{Query, State},
    response::Json,
};
use std::collections::HashMap;

use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;
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
    };

    match agent_service.register_agent(registration).await {
        Ok(response) => Ok(Json(AgentRegistrationResponse {
            status: response.status,
            message: response.message,
            success: response.success,
            transport: response.transport,
            rates: response.rates,
        })),
        Err(e) => Err(ApiError::internal(format!("Registration failed: {}", e))),
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


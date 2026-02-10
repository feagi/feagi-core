// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Agent API endpoints - Exact port from Python `/v1/agent/*` routes
//!
//! These endpoints match the Python implementation at:
//! feagi-py/feagi/api/v1/feagi_agent.py

use std::collections::HashMap;

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, Query, State};
use crate::v1::agent_dtos::*;
use feagi_services::traits::agent_service::HeartbeatRequest as ServiceHeartbeatRequest;
use base64::Engine;
use tracing::{error, info, warn};

#[cfg(feature = "feagi-agent")]
use feagi_agent::{
    AgentCapabilities as RegistrationCapabilities, AgentDescriptor, AuthToken,
};
#[cfg(feature = "feagi-agent")]
use crate::common::agent_registration::{
    auto_create_cortical_areas_from_device_registrations as auto_create_cortical_areas_shared,
    derive_motor_cortical_ids_from_device_registrations,
};
#[cfg(feature = "feagi-agent")]
use feagi_serialization::SessionID;
#[cfg(feature = "feagi-agent")]

fn parse_agent_descriptor(agent_id: &str) -> ApiResult<AgentDescriptor> {
    #[cfg(feature = "feagi-agent")]
    {
        AgentDescriptor::try_from_base64(agent_id).map_err(|e| {
            ApiError::invalid_input(format!(
                "Invalid agent_id (expected AgentDescriptor base64): {e}"
            ))
        })
    }
    #[cfg(not(feature = "feagi-agent"))]
    {
        Err(ApiError::internal("feagi-agent feature not enabled"))
    }
}

#[cfg(feature = "feagi-agent")]
fn parse_auth_token(request: &AgentRegistrationRequest) -> ApiResult<AuthToken> {
    let token_b64 = request
        .auth_token
        .as_deref()
        .ok_or_else(|| ApiError::invalid_input("Missing auth_token for registration"))?;
    AuthToken::from_base64(token_b64).ok_or_else(|| {
        ApiError::invalid_input("Invalid auth_token (expected base64 32-byte token)")
    })
}

#[cfg(feature = "feagi-agent")]
fn derive_capabilities_from_device_registrations(
    device_registrations: &serde_json::Value,
) -> ApiResult<Vec<RegistrationCapabilities>> {
    let obj = device_registrations.as_object().ok_or_else(|| {
        ApiError::invalid_input("device_registrations must be a JSON object")
    })?;

    let input_units = obj
        .get("input_units_and_encoder_properties")
        .and_then(|v| v.as_object());
    let output_units = obj
        .get("output_units_and_decoder_properties")
        .and_then(|v| v.as_object());
    let feedbacks = obj.get("feedbacks").and_then(|v| v.as_object());

    let mut capabilities = Vec::new();
    if input_units.map(|m| !m.is_empty()).unwrap_or(false) {
        capabilities.push(RegistrationCapabilities::SendSensorData);
    }
    if output_units.map(|m| !m.is_empty()).unwrap_or(false) {
        capabilities.push(RegistrationCapabilities::ReceiveMotorData);
    }
    if feedbacks.map(|m| !m.is_empty()).unwrap_or(false) {
        capabilities.push(RegistrationCapabilities::ReceiveNeuronVisualizations);
    }

    if capabilities.is_empty() {
        return Err(ApiError::invalid_input(
            "device_registrations does not declare any input/output/feedback units",
        ));
    }

    Ok(capabilities)
}

/// Derive capabilities for visualization-only agents (no device_registrations).
/// Requires `capabilities.visualization` with valid `rate_hz`. Auth is still required by caller.
#[cfg(feature = "feagi-agent")]
fn derive_capabilities_from_visualization_capability(
    request: &AgentRegistrationRequest,
) -> ApiResult<Vec<RegistrationCapabilities>> {
    let viz = request
        .capabilities
        .get("visualization")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            ApiError::invalid_input(
                "visualization-only registration requires capabilities.visualization object",
            )
        })?;
    let rate_hz = viz.get("rate_hz").and_then(|v| v.as_f64()).ok_or_else(|| {
        ApiError::invalid_input(
            "capabilities.visualization must include rate_hz (number > 0)",
        )
    })?;
    if rate_hz <= 0.0 {
        return Err(ApiError::invalid_input(
            "capabilities.visualization.rate_hz must be > 0",
        ));
    }
    Ok(vec![RegistrationCapabilities::ReceiveNeuronVisualizations])
}

#[cfg(feature = "feagi-agent")]
fn parse_capability_rate_hz(
    capabilities: &HashMap<String, serde_json::Value>,
    capability_key: &str,
) -> ApiResult<Option<f64>> {
    let Some(capability_value) = capabilities.get(capability_key) else {
        return Ok(None);
    };

    let Some(rate_value) = capability_value.get("rate_hz") else {
        return Ok(None);
    };

    let rate_hz = rate_value.as_f64().ok_or_else(|| {
        ApiError::invalid_input(format!(
            "Invalid rate_hz for capability '{}': expected number",
            capability_key
        ))
    })?;

    if rate_hz <= 0.0 {
        return Err(ApiError::invalid_input(format!(
            "Invalid rate_hz for capability '{}': must be > 0",
            capability_key
        )));
    }

    Ok(Some(rate_hz))
}

#[cfg(feature = "feagi-agent")]
fn capability_key(capability: &RegistrationCapabilities) -> &'static str {
    match capability {
        RegistrationCapabilities::SendSensorData => "send_sensor_data",
        RegistrationCapabilities::ReceiveMotorData => "receive_motor_data",
        RegistrationCapabilities::ReceiveNeuronVisualizations => "receive_neuron_visualizations",
        RegistrationCapabilities::ReceiveSystemMessages => "receive_system_messages",
    }
}

fn get_agent_name_from_id(agent_id: &str) -> ApiResult<String> {
    #[cfg(feature = "feagi-agent")]
    {
        let descriptor = parse_agent_descriptor(agent_id)?;
        let agent_name = descriptor.agent_name().to_string();
        if agent_name.is_empty() {
            return Err(ApiError::invalid_input(format!(
                "Agent '{}' does not contain a readable name",
                agent_id
            )));
        }
        Ok(agent_name)
    }
    #[cfg(not(feature = "feagi-agent"))]
    {
        Err(ApiError::internal(
            "Agent name requires feagi-agent feature".to_string(),
        ))
    }
}

async fn auto_create_cortical_areas_from_device_registrations(
    state: &ApiState,
    device_registrations: &serde_json::Value,
) {
    #[cfg(feature = "feagi-agent")]
    auto_create_cortical_areas_shared(state, device_registrations).await;
}

/// Register a new agent with FEAGI and receive connection details including transport configuration and ports.
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
    info!(
        "🦀 [API] Registration request received for agent '{}' (type: {})",
        request.agent_id, request.agent_type
    );

    if state.agent_handler.is_none() {
        return Err(ApiError::internal("Agent handler not available"));
    }

    let agent_descriptor = parse_agent_descriptor(&request.agent_id)?;
    
    // Extract device_registrations from capabilities
    let device_registrations_opt = request
        .capabilities
        .get("device_registrations")
        .and_then(|v| v.as_object().map(|_| v.clone()));

    // Store device registrations in handler if provided
    if let Some(device_regs) = &device_registrations_opt {
        if let Some(handler) = &state.agent_handler {
            let mut handler_guard = handler.lock().unwrap();
            handler_guard.set_device_registrations_by_descriptor(
                agent_descriptor.clone(),
                device_regs.clone()
            );
            info!("✅ [API] Stored device registrations for agent '{}'", request.agent_id);
            
            drop(handler_guard);
            
            // Trigger auto-creation of cortical areas
            auto_create_cortical_areas_from_device_registrations(&state, device_regs).await;
        }
    }

    // Get available endpoints from handler
    let endpoints_map = if let Some(handler) = &state.agent_handler {
        let handler_guard = handler.lock().unwrap();
        let transport_endpoints = handler_guard.get_transport_endpoints();
        
        let mut response_map = HashMap::new();
        for (protocol, endpoints) in transport_endpoints {
            let protocol_name = format!("{:?}", protocol).to_lowercase();
            response_map.insert(protocol_name, serde_json::json!(endpoints));
        }
        response_map
    } else {
        HashMap::new()
    };

    Ok(Json(AgentRegistrationResponse {
        status: "success".to_string(),
        message: "Agent configuration stored. Connect via ZMQ/WebSocket for full registration".to_string(),
        success: true,
        transport: Some(endpoints_map),
        rates: None,
        transports: None,
        recommended_transport: None,
        shm_paths: None,
        cortical_areas: serde_json::json!({}),
    }))
}

/// Send a heartbeat to keep the agent registered and prevent timeout disconnection.
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

/// Get a list of all currently registered agent IDs.
#[utoipa::path(
    get,
    path = "/v1/agent/list",
    responses(
        (status = 200, description = "List of agent IDs", body = Vec<String>),
        (status = 503, description = "Registration service unavailable")
    ),
    tag = "agent"
)]
pub async fn list_agents(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    match agent_service.list_agents().await {
        Ok(agent_ids) => Ok(Json(agent_ids)),
        Err(e) => Err(ApiError::internal(format!("Failed to list agents: {}", e))),
    }
}

/// Get agent properties including type, capabilities, version, and connection details. Uses query parameter ?agent_id=xxx.
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

    let agent_name = get_agent_name_from_id(agent_id)?;
    match agent_service.get_agent_properties(agent_id).await {
        Ok(properties) => Ok(Json(AgentPropertiesResponse {
            agent_name,
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

/// Get shared memory configuration and paths for all registered agents using shared memory transport.
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

/// Deregister an agent from FEAGI and clean up its resources.
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

/// Manually stimulate neurons at specific coordinates across multiple cortical areas for testing and debugging.
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

/// Get Fire Queue (FQ) sampler coordination status including visualization and motor sampling configuration.
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
    let agent_ids = agent_service
        .list_agents()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list agents: {}", e)))?;

    // Get FCL sampler config from RuntimeService
    let (fcl_frequency, fcl_consumer) = runtime_service
        .get_fcl_sampler_config()
        .await
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
    viz_sampler.insert(
        "enabled".to_string(),
        serde_json::json!(!visualization_agents.is_empty()),
    );
    viz_sampler.insert(
        "reason".to_string(),
        serde_json::json!(if visualization_agents.is_empty() {
            "No visualization agents connected".to_string()
        } else {
            format!(
                "{} visualization agent(s) connected",
                visualization_agents.len()
            )
        }),
    );
    viz_sampler.insert(
        "agents_requiring".to_string(),
        serde_json::json!(visualization_agents),
    );
    viz_sampler.insert("frequency_hz".to_string(), serde_json::json!(fcl_frequency));
    fq_coordination.insert(
        "visualization_fq_sampler".to_string(),
        serde_json::json!(viz_sampler),
    );

    let mut motor_sampler = HashMap::new();
    motor_sampler.insert(
        "enabled".to_string(),
        serde_json::json!(!motor_agents.is_empty()),
    );
    motor_sampler.insert(
        "reason".to_string(),
        serde_json::json!(if motor_agents.is_empty() {
            "No motor agents connected".to_string()
        } else {
            format!("{} motor agent(s) connected", motor_agents.len())
        }),
    );
    motor_sampler.insert(
        "agents_requiring".to_string(),
        serde_json::json!(motor_agents),
    );
    motor_sampler.insert("frequency_hz".to_string(), serde_json::json!(100.0));
    fq_coordination.insert(
        "motor_fq_sampler".to_string(),
        serde_json::json!(motor_sampler),
    );

    let mut response = HashMap::new();
    response.insert(
        "fq_sampler_coordination".to_string(),
        serde_json::json!(fq_coordination),
    );
    response.insert(
        "agent_registry".to_string(),
        serde_json::json!({
            "total_agents": agent_ids.len(),
            "agent_ids": agent_ids
        }),
    );
    response.insert(
        "system_status".to_string(),
        serde_json::json!("coordinated_via_registration_manager"),
    );
    response.insert(
        "fcl_sampler_consumer".to_string(),
        serde_json::json!(fcl_consumer),
    );

    Ok(Json(response))
}

/// Get list of all supported agent types and capability types (sensory, motor, visualization, etc.).
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
    response.insert(
        "agent_types".to_string(),
        vec![
            "sensory".to_string(),
            "motor".to_string(),
            "both".to_string(),
            "visualization".to_string(),
            "infrastructure".to_string(),
        ],
    );
    response.insert(
        "capability_types".to_string(),
        vec![
            "vision".to_string(),
            "motor".to_string(),
            "visualization".to_string(),
            "sensory".to_string(),
        ],
    );

    Ok(Json(response))
}

/// Get capabilities for all agents with optional filtering and payload includes.
#[utoipa::path(
    get,
    path = "/v1/agent/capabilities/all",
    params(
        ("agent_type" = Option<String>, Query, description = "Filter by agent type (exact match)"),
        ("capability" = Option<String>, Query, description = "Filter by capability key(s), comma-separated"),
        ("include_device_registrations" = Option<bool>, Query, description = "Include device registration payloads per agent")
    ),
    responses(
        (status = 200, description = "Agent capabilities map", body = HashMap<String, AgentCapabilitiesSummary>),
        (status = 400, description = "Invalid query"),
        (status = 500, description = "Failed to get agent capabilities")
    ),
    tag = "agent"
)]
pub async fn get_all_agent_capabilities(
    State(state): State<ApiState>,
    Query(params): Query<AgentCapabilitiesAllQuery>,
) -> ApiResult<Json<HashMap<String, AgentCapabilitiesSummary>>> {
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    let include_device_registrations = params.include_device_registrations.unwrap_or(false);
    let capability_filters: Option<Vec<String>> = params.capability.as_ref().and_then(|value| {
        let filters: Vec<String> = value
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(String::from)
            .collect();
        if filters.is_empty() {
            None
        } else {
            Some(filters)
        }
    });

    let agent_ids = agent_service
        .list_agents()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list agents: {}", e)))?;

    let mut response: HashMap<String, AgentCapabilitiesSummary> = HashMap::new();

    for agent_id in agent_ids {
        let agent_name = get_agent_name_from_id(&agent_id)?;
        let properties = match agent_service.get_agent_properties(&agent_id).await {
            Ok(props) => props,
            Err(_) => continue,
        };

        if let Some(ref agent_type_filter) = params.agent_type {
            if properties.agent_type != *agent_type_filter {
                continue;
            }
        }

        if let Some(ref filters) = capability_filters {
            let has_match = filters
                .iter()
                .any(|capability| properties.capabilities.contains_key(capability));
            if !has_match {
                continue;
            }
        }

        let device_registrations = if include_device_registrations {
            #[cfg(feature = "feagi-agent")]
            {
                Some(export_device_registrations_from_connector(
                    &state, &agent_id,
                )?)
            }
            #[cfg(not(feature = "feagi-agent"))]
            {
                None
            }
        } else {
            None
        };
        response.insert(
            agent_id,
            AgentCapabilitiesSummary {
                agent_name,
                capabilities: properties.capabilities,
                device_registrations,
            },
        );
    }

    Ok(Json(response))
}

#[cfg(feature = "feagi-agent")]
fn export_device_registrations_from_connector(
    state: &ApiState,
    agent_id: &str,
) -> ApiResult<serde_json::Value> {
    let agent_descriptor = parse_agent_descriptor(agent_id)?;
    
    if let Some(handler) = &state.agent_handler {
        let handler_guard = handler.lock().unwrap();
        if let Some(regs) = handler_guard.get_device_registrations_by_descriptor(&agent_descriptor) {
            return Ok(regs.clone());
        }
    }
    
    Err(ApiError::not_found(
        "device_registrations",
        &format!("No device registrations found for agent '{}'", agent_id),
    ))
}

/// Get comprehensive agent information including status, capabilities, version, and connection details.
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

    let properties = agent_service
        .get_agent_properties(&agent_id)
        .await
        .map_err(|e| ApiError::not_found("agent", &e.to_string()))?;

    let mut response = HashMap::new();
    response.insert("agent_id".to_string(), serde_json::json!(agent_id));
    response.insert(
        "agent_name".to_string(),
        serde_json::json!(get_agent_name_from_id(&agent_id)?),
    );
    response.insert(
        "agent_type".to_string(),
        serde_json::json!(properties.agent_type),
    );
    response.insert(
        "agent_ip".to_string(),
        serde_json::json!(properties.agent_ip),
    );
    response.insert(
        "agent_data_port".to_string(),
        serde_json::json!(properties.agent_data_port),
    );
    response.insert(
        "capabilities".to_string(),
        serde_json::json!(properties.capabilities),
    );
    response.insert(
        "agent_version".to_string(),
        serde_json::json!(properties.agent_version),
    );
    response.insert(
        "controller_version".to_string(),
        serde_json::json!(properties.controller_version),
    );
    response.insert("status".to_string(), serde_json::json!("active"));
    if let Some(ref transport) = properties.chosen_transport {
        response.insert("chosen_transport".to_string(), serde_json::json!(transport));
    }

    Ok(Json(response))
}

/// Get agent properties using path parameter. Same as /v1/agent/properties but with agent_id in the URL path.
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
            agent_name: get_agent_name_from_id(&agent_id)?,
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

/// Configure agent parameters and settings. (Not yet implemented)
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
        (
            "message".to_string(),
            "Agent configuration updated".to_string(),
        ),
        ("status".to_string(), "not_yet_implemented".to_string()),
    ])))
}

/// Export device registrations for an agent
///
/// Returns the complete device registration configuration including
/// sensor and motor device registrations, encoder/decoder properties,
/// and feedback configurations in the format compatible with
/// ConnectorAgent::get_device_registration_json.
#[utoipa::path(
    get,
    path = "/v1/agent/{agent_id}/device_registrations",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Device registrations exported successfully", body = DeviceRegistrationExportResponse),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Failed to export device registrations")
    ),
    tag = "agent"
)]
pub async fn export_device_registrations(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<DeviceRegistrationExportResponse>> {
    info!(
        "🦀 [API] Device registration export requested for agent '{}'",
        agent_id
    );

    // Verify agent exists only if AgentService is available.
    //
    // Rationale: live contract tests and minimal deployments can run without an AgentService.
    // In that case, device registration import/export should still work as a pure per-agent
    // configuration store (ConnectorAgent-backed).
    if let Some(agent_service) = state.agent_service.as_ref() {
        let _properties = agent_service
            .get_agent_properties(&agent_id)
            .await
            .map_err(|e| ApiError::not_found("agent", &e.to_string()))?;
    } else {
        info!(
            "ℹ️ [API] Agent service not available; skipping agent existence check for export (agent '{}')",
            agent_id
        );
    }

    // Get device registrations from agent_handler
    #[cfg(feature = "feagi-agent")]
    let device_registrations = {
        let agent_descriptor = parse_agent_descriptor(&agent_id)?;
        
        if let Some(handler) = &state.agent_handler {
            let handler_guard = handler.lock().unwrap();
            if let Some(regs) = handler_guard.get_device_registrations_by_descriptor(&agent_descriptor) {
                info!("📤 [API] Found device registrations for agent '{}'", agent_id);
                regs.clone()
            } else {
                warn!("⚠️ [API] No device registrations found for agent '{}'", agent_id);
                serde_json::json!({
                    "input_units_and_encoder_properties": {},
                    "output_units_and_decoder_properties": {},
                    "feedbacks": []
                })
            }
        } else {
            warn!("⚠️ [API] No agent_handler available");
            serde_json::json!({
                "input_units_and_encoder_properties": {},
                "output_units_and_decoder_properties": {},
                "feedbacks": []
            })
        }
    };

    #[cfg(not(feature = "feagi-agent"))]
    // @architecture:acceptable - fallback when feature is disabled
    // Returns empty structure when feagi-agent feature is not compiled in
    let device_registrations = serde_json::json!({
        "input_units_and_encoder_properties": {},
        "output_units_and_decoder_properties": {},
        "feedbacks": []
    });

    info!(
        "✅ [API] Device registration export succeeded for agent '{}'",
        agent_id
    );

    Ok(Json(DeviceRegistrationExportResponse {
        device_registrations,
        agent_id,
    }))
}

/// Import device registrations for an agent
///
/// Imports a device registration configuration, replacing all existing
/// device registrations for the agent. The configuration must be in
/// the format compatible with ConnectorAgent::set_device_registrations_from_json.
///
/// # Warning
/// This operation **wipes all existing registered devices** before importing
/// the new configuration.
#[utoipa::path(
    post,
    path = "/v1/agent/{agent_id}/device_registrations",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    request_body = DeviceRegistrationImportRequest,
    responses(
        (status = 200, description = "Device registrations imported successfully", body = DeviceRegistrationImportResponse),
        (status = 400, description = "Invalid device registration configuration"),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Failed to import device registrations")
    ),
    tag = "agent"
)]
pub async fn import_device_registrations(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
    Json(request): Json<DeviceRegistrationImportRequest>,
) -> ApiResult<Json<DeviceRegistrationImportResponse>> {
    info!(
        "🦀 [API] Device registration import requested for agent '{}'",
        agent_id
    );

    // Verify agent exists only if AgentService is available (see export_device_registrations).
    if let Some(agent_service) = state.agent_service.as_ref() {
        let _properties = agent_service
            .get_agent_properties(&agent_id)
            .await
            .map_err(|e| ApiError::not_found("agent", &e.to_string()))?;
    } else {
        info!(
            "ℹ️ [API] Agent service not available; skipping agent existence check for import (agent '{}')",
            agent_id
        );
    }

    // Validate the device registration JSON structure
    // Check that it has the expected fields
    if !request.device_registrations.is_object() {
        return Err(ApiError::invalid_input(
            "Device registrations must be a JSON object",
        ));
    }

    // Validate required fields exist
    let obj = request.device_registrations.as_object().unwrap();
    if !obj.contains_key("input_units_and_encoder_properties")
        || !obj.contains_key("output_units_and_decoder_properties")
        || !obj.contains_key("feedbacks")
    {
        return Err(ApiError::invalid_input(
            "Device registrations must contain: input_units_and_encoder_properties, output_units_and_decoder_properties, and feedbacks",
        ));
    }

    // Store device registrations in agent_handler
    #[cfg(feature = "feagi-agent")]
    {
        let agent_descriptor = parse_agent_descriptor(&agent_id)?;
        
        if let Some(handler) = &state.agent_handler {
            let mut handler_guard = handler.lock().unwrap();
            handler_guard.set_device_registrations_by_descriptor(
                agent_descriptor,
                request.device_registrations.clone()
            );
            info!("📥 [API] Imported device registrations for agent '{}'", agent_id);
        } else {
            warn!("⚠️ [API] No agent_handler available to store device registrations");
        }

        auto_create_cortical_areas_from_device_registrations(&state, &request.device_registrations)
            .await;

        Ok(Json(DeviceRegistrationImportResponse {
            success: true,
            message: format!(
                "Device registrations imported successfully for agent '{}'",
                agent_id
            ),
            agent_id,
        }))
    }

    #[cfg(not(feature = "feagi-agent"))]
    {
        info!(
            "✅ [API] Device registration import succeeded for agent '{}' (feagi-agent feature not enabled)",
            agent_id
        );
        Ok(Json(DeviceRegistrationImportResponse {
            success: true,
            message: format!(
                "Device registrations imported successfully for agent '{}' (feature not enabled)",
                agent_id
            ),
            agent_id,
        }))
    }
}

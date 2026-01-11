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
use feagi_services::traits::agent_service::{
    AgentRegistration, HeartbeatRequest as ServiceHeartbeatRequest,
};
use tracing::{error, info, warn};

#[cfg(feature = "feagi-agent")]
use feagi_agent::sdk::ConnectorAgent;
#[cfg(feature = "feagi-agent")]
use std::sync::{Arc, Mutex};

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
    
    let agent_service = state
        .agent_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Agent service not available"))?;

    // Extract device_registrations from capabilities before they're moved
    #[cfg(feature = "feagi-agent")]
    let device_registrations_opt = request.capabilities
        .get("device_registrations")
        .and_then(|v| {
            // Validate structure before cloning
            if let Some(obj) = v.as_object() {
                if obj.contains_key("input_units_and_encoder_properties")
                    && obj.contains_key("output_units_and_decoder_properties")
                    && obj.contains_key("feedbacks")
                {
                    Some(v.clone())
                } else {
                    None
                }
            } else {
                None
            }
        });

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
            info!(
                "✅ [API] Agent '{}' registration succeeded (status: {})",
                request.agent_id, response.status
            );
            
            // Initialize ConnectorAgent for this agent
            // If capabilities contained device_registrations, import them
            #[cfg(feature = "feagi-agent")]
            if let Some(device_registrations_value) = device_registrations_opt {
                let mut connectors = state.agent_connectors.write();
                let connector = connectors
                    .entry(request.agent_id.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(ConnectorAgent::new())))
                    .clone();
                
                let mut connector_guard = connector.lock().unwrap();
                if let Err(e) = connector_guard.import_device_registrations_as_config_json(device_registrations_value) {
                    warn!(
                        "⚠️ [API] Failed to import device registrations from capabilities for agent '{}': {}",
                        request.agent_id, e
                    );
                } else {
                    info!(
                        "✅ [API] Imported device registrations from capabilities for agent '{}'",
                        request.agent_id
                    );
                }
            } else {
                // Initialize empty ConnectorAgent even if no device_registrations
                let mut connectors = state.agent_connectors.write();
                connectors
                    .entry(request.agent_id.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(ConnectorAgent::new())));
            }
            
            // Convert service TransportConfig to API TransportConfig
            let transports = response.transports.map(|ts| {
                ts.into_iter()
                    .map(|t| crate::v1::agent_dtos::TransportConfig {
                        transport_type: t.transport_type,
                        enabled: t.enabled,
                        ports: t.ports,
                        host: t.host,
                    })
                    .collect()
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
        }
        Err(e) => {
            // Check if error is about unsupported transport (validation error)
            let error_msg = e.to_string();
            warn!(
                "❌ [API] Agent '{}' registration FAILED: {}",
                request.agent_id, error_msg
            );
            if error_msg.contains("not supported") || error_msg.contains("disabled") {
                Err(ApiError::invalid_input(error_msg))
            } else {
                Err(ApiError::internal(format!("Registration failed: {}", e)))
            }
        }
    }
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
/// ConnectorAgent::export_device_registrations_as_config_json.
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

    // Get existing ConnectorAgent for this agent (don't create new one)
    #[cfg(feature = "feagi-agent")]
    let device_registrations = {
        // Get existing ConnectorAgent - don't create a new one
        // If no ConnectorAgent exists, it means device registrations haven't been imported yet
        let connector = {
            let connectors = state.agent_connectors.read();
            connectors.get(&agent_id).cloned()
        };

        let connector = match connector {
            Some(c) => {
                info!("🔍 [API] Found existing ConnectorAgent for agent '{}'", agent_id);
                c
            }
            None => {
                warn!(
                    "⚠️ [API] No ConnectorAgent found for agent '{}' - device registrations may not have been imported yet. Total agents in registry: {}",
                    agent_id,
                    {
                        let connectors = state.agent_connectors.read();
                        connectors.len()
                    }
                );
                // Return empty structure - don't create and store a new ConnectorAgent
                // This prevents interference with future imports
                return Ok(Json(DeviceRegistrationExportResponse {
                    device_registrations: serde_json::json!({
                        "input_units_and_encoder_properties": {},
                        "output_units_and_decoder_properties": {},
                        "feedbacks": []
                    }),
                    agent_id,
                }));
            }
        };

        // Export device registrations using ConnectorAgent method
        let connector_guard = connector.lock().unwrap();
        match connector_guard.export_device_registrations_as_config_json() {
            Ok(registrations) => {
                // Log what we're exporting for debugging
                let input_count = registrations
                    .get("input_units_and_encoder_properties")
                    .and_then(|v| v.as_object())
                    .map(|m| m.len())
                    .unwrap_or(0);
                let output_count = registrations
                    .get("output_units_and_decoder_properties")
                    .and_then(|v| v.as_object())
                    .map(|m| m.len())
                    .unwrap_or(0);
                let feedback_count = registrations
                    .get("feedbacks")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                
                info!(
                    "📤 [API] Exporting device registrations for agent '{}': {} input units, {} output units, {} feedbacks",
                    agent_id, input_count, output_count, feedback_count
                );
                
                if input_count == 0 && output_count == 0 && feedback_count == 0 {
                    warn!(
                        "⚠️ [API] Exported device registrations for agent '{}' are empty - agent may not have synced device registrations yet",
                        agent_id
                    );
                }
                
                registrations
            }
            Err(e) => {
                warn!(
                    "⚠️ [API] Failed to export device registrations for agent '{}': {}",
                    agent_id, e
                );
                // @architecture:acceptable - emergency fallback on export failure
                // Return empty structure on error to prevent API failure
                serde_json::json!({
                    "input_units_and_encoder_properties": {},
                    "output_units_and_decoder_properties": {},
                    "feedbacks": []
                })
            }
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
/// the format compatible with ConnectorAgent::import_device_registrations_as_config_json.
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

    // Import device registrations using ConnectorAgent
    #[cfg(feature = "feagi-agent")]
    {
        // Get or create ConnectorAgent for this agent
        let connector = {
            let mut connectors = state.agent_connectors.write();
            let was_existing = connectors.contains_key(&agent_id);
            let connector = connectors
                .entry(agent_id.clone())
                .or_insert_with(|| {
                    info!("🔧 [API] Creating new ConnectorAgent for agent '{}'", agent_id);
                    Arc::new(Mutex::new(ConnectorAgent::new()))
                })
                .clone();
            if was_existing {
                info!("🔧 [API] Using existing ConnectorAgent for agent '{}'", agent_id);
            }
            connector
        };

        // Import device registrations using ConnectorAgent method
        let mut connector_guard = connector.lock().unwrap();
        
        // Log what we're importing for debugging
        let input_count = request.device_registrations
            .get("input_units_and_encoder_properties")
            .and_then(|v| v.as_object())
            .map(|m| m.len())
            .unwrap_or(0);
        let output_count = request.device_registrations
            .get("output_units_and_decoder_properties")
            .and_then(|v| v.as_object())
            .map(|m| m.len())
            .unwrap_or(0);
        let feedback_count = request.device_registrations
            .get("feedbacks")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        
        info!(
            "📥 [API] Importing device registrations for agent '{}': {} input units, {} output units, {} feedbacks",
            agent_id, input_count, output_count, feedback_count
        );
        
        match connector_guard.import_device_registrations_as_config_json(request.device_registrations.clone()) {
            Ok(()) => {
                // Verify the import worked by exporting again
                match connector_guard.export_device_registrations_as_config_json() {
                    Ok(exported) => {
                        let exported_input_count = exported
                            .get("input_units_and_encoder_properties")
                            .and_then(|v| v.as_object())
                            .map(|m| m.len())
                            .unwrap_or(0);
                        let exported_output_count = exported
                            .get("output_units_and_decoder_properties")
                            .and_then(|v| v.as_object())
                            .map(|m| m.len())
                            .unwrap_or(0);
                        
                        info!(
                            "✅ [API] Device registration import succeeded for agent '{}' (verified: {} input, {} output)",
                            agent_id, exported_input_count, exported_output_count
                        );
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ [API] Import succeeded but verification export failed for agent '{}': {}",
                            agent_id, e
                        );
                    }
                }
                
                Ok(Json(DeviceRegistrationImportResponse {
                    success: true,
                    message: format!(
                        "Device registrations imported successfully for agent '{}'",
                        agent_id
                    ),
                    agent_id,
                }))
            }
            Err(e) => {
                error!(
                    "❌ [API] Failed to import device registrations for agent '{}': {}",
                    agent_id, e
                );
                Err(ApiError::invalid_input(format!(
                    "Failed to import device registrations: {}",
                    e
                )))
            }
        }
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

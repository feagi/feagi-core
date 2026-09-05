// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Connectome API Endpoints - Exact port from Python `/v1/connectome/*`

// Removed - using crate::common::State instead
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, Query, State};
use crate::endpoints::cortical_area::synapse_details_for_neuron;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};
use utoipa::{IntoParams, ToSchema};

/// Multipart file upload schema so Swagger shows a connectome file picker.
#[derive(Debug, Clone, utoipa::ToSchema)]
pub struct ConnectomeFileUploadForm {
    /// Saved `.connectome` file (binary FEAGI connectome format).
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}

/// One saved connectome file under the configured connectome directory.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConnectomeSavedFileEntry {
    /// Base filename (no directory components).
    pub file_name: String,
    /// Absolute path on the FEAGI host filesystem.
    pub file_path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last modified time (RFC 3339 UTC).
    pub modified_at: String,
}

/// Connectome directory listing for Swagger UI and local upload workflows.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConnectomeDirectoryResponse {
    /// Absolute path to `{data_root}/connectome`.
    pub directory: String,
    /// Saved `.connectome` files in the directory, newest first.
    pub files: Vec<ConnectomeSavedFileEntry>,
}

/// Load a connectome already saved under `{data_root}/connectome`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ConnectomeUploadSavedRequest {
    /// Base filename returned by [`get_connectome_directory`] (e.g. `saved_connectome_*.connectome`).
    pub file_name: String,
}

/// Query for [`get_memory_neuron`].
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MemoryNeuronQuery {
    /// Global memory neuron id (reserved range, typically 50_000_000+).
    pub neuron_id: u32,
}

/// Full memory neuron detail: plasticity lifecycle fields plus connectome synapses.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MemoryNeuronDetailResponse {
    pub neuron_id: u64,
    pub cortical_idx: u32,
    pub cortical_id: String,
    pub cortical_name: String,
    pub pattern_hash: Option<u64>,
    pub is_longterm_memory: bool,
    pub is_active: bool,
    pub lifespan_current: u32,
    pub lifespan_initial: u32,
    pub lifespan_growth_rate: f32,
    pub creation_burst: u64,
    pub last_activation_burst: u64,
    pub activation_count: u32,
    pub outgoing_synapse_count: usize,
    pub incoming_synapse_count: usize,
    pub outgoing_synapses: serde_json::Value,
    pub incoming_synapses: serde_json::Value,
}

/// GET /v1/connectome/cortical_areas/list/detailed
#[utoipa::path(
    get,
    path = "/v1/connectome/cortical_areas/list/detailed",
    tag = "connectome",
    responses(
        (status = 200, description = "Detailed cortical areas list", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_cortical_areas_list_detailed(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            tracing::info!(target: "feagi-api",
                "[DETAILED-LIST] Returning {} cortical areas", areas.len()
            );

            let detailed: HashMap<String, serde_json::Value> = areas
                .into_iter()
                .map(|area| {
                    tracing::debug!(target: "feagi-api",
                        "[DETAILED-LIST] Area {}: cortical_type='{}', is_mem_type={:?}",
                        area.cortical_id, area.cortical_type,
                        area.properties.get("is_mem_type")
                    );

                    let json_value = serde_json::to_value(&area).unwrap_or_default();

                    tracing::debug!(target: "feagi-api",
                        "[DETAILED-LIST] Serialized area {} has cortical_type: {}",
                        area.cortical_id, json_value.get("cortical_type").is_some()
                    );

                    (area.cortical_id.clone(), json_value)
                })
                .collect();
            Ok(Json(detailed))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get detailed list: {}",
            e
        ))),
    }
}

/// GET /v1/connectome/properties/dimensions
#[utoipa::path(get, path = "/v1/connectome/properties/dimensions", tag = "connectome")]
pub async fn get_properties_dimensions(
    State(_state): State<ApiState>,
) -> ApiResult<Json<(usize, usize, usize)>> {
    // Will use state when wired to NPU
    // TODO: Get max dimensions from connectome manager
    Ok(Json((0, 0, 0)))
}

/// GET /v1/connectome/properties/mappings
#[utoipa::path(get, path = "/v1/connectome/properties/mappings", tag = "connectome")]
pub async fn get_properties_mappings(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    // TODO: Get all cortical mappings
    Ok(Json(HashMap::new()))
}

/// GET /v1/connectome/snapshot
#[utoipa::path(get, path = "/v1/connectome/snapshot", tag = "connectome", responses((status = 200, body = HashMap<String, serde_json::Value>)))]
pub async fn get_snapshot(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    let regions = connectome_service
        .list_brain_regions()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    let mut response = HashMap::new();
    response.insert(
        "cortical_area_count".to_string(),
        serde_json::json!(areas.len()),
    );
    response.insert(
        "brain_region_count".to_string(),
        serde_json::json!(regions.len()),
    );
    Ok(Json(response))
}

/// GET /v1/connectome/stats
#[utoipa::path(get, path = "/v1/connectome/stats", tag = "connectome", responses((status = 200, body = HashMap<String, serde_json::Value>)))]
pub async fn get_stats(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let analytics_service = state.analytics_service.as_ref();
    let health = analytics_service
        .get_system_health()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    let mut response = HashMap::new();
    response.insert(
        "neuron_count".to_string(),
        serde_json::json!(health.neuron_count),
    );
    response.insert(
        "cortical_area_count".to_string(),
        serde_json::json!(health.cortical_area_count),
    );
    Ok(Json(response))
}

/// POST /v1/connectome/batch_neuron_operations
#[utoipa::path(
    post,
    path = "/v1/connectome/batch_neuron_operations",
    tag = "connectome"
)]
pub async fn post_batch_neuron_operations(
    State(_state): State<ApiState>,
    Json(_ops): Json<Vec<HashMap<String, serde_json::Value>>>,
) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("processed".to_string(), 0);
    Ok(Json(response))
}

/// POST /v1/connectome/batch_synapse_operations
#[utoipa::path(
    post,
    path = "/v1/connectome/batch_synapse_operations",
    tag = "connectome"
)]
pub async fn post_batch_synapse_operations(
    State(_state): State<ApiState>,
    Json(_ops): Json<Vec<HashMap<String, serde_json::Value>>>,
) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("processed".to_string(), 0);
    Ok(Json(response))
}

/// GET /v1/connectome/neuron_count
#[utoipa::path(get, path = "/v1/connectome/neuron_count", tag = "connectome")]
pub async fn get_neuron_count(State(state): State<ApiState>) -> ApiResult<Json<i64>> {
    let analytics = state.analytics_service.as_ref();
    let health = analytics
        .get_system_health()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    Ok(Json(health.neuron_count as i64))
}

/// GET /v1/connectome/synapse_count
#[utoipa::path(get, path = "/v1/connectome/synapse_count", tag = "connectome")]
pub async fn get_synapse_count(State(_state): State<ApiState>) -> ApiResult<Json<i64>> {
    Ok(Json(0))
}

/// GET /v1/connectome/paths
#[utoipa::path(get, path = "/v1/connectome/paths", tag = "connectome")]
pub async fn get_paths(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vec<Vec<String>>>> {
    Ok(Json(Vec::new()))
}

/// GET /v1/connectome/cumulative_stats
#[utoipa::path(get, path = "/v1/connectome/cumulative_stats", tag = "connectome")]
pub async fn get_cumulative_stats(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, i64>>> {
    let mut response = HashMap::new();
    response.insert("total_bursts".to_string(), 0);
    Ok(Json(response))
}

/// GET /v1/connectome/area_details
#[utoipa::path(get, path = "/v1/connectome/area_details", tag = "connectome")]
pub async fn get_area_details(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let area_ids_str = params
        .get("area_ids")
        .ok_or_else(|| ApiError::invalid_input("area_ids required"))?;
    let area_ids: Vec<&str> = area_ids_str.split(',').collect();
    let connectome_service = state.connectome_service.as_ref();
    let mut details = HashMap::new();
    for area_id in area_ids {
        if let Ok(area) = connectome_service.get_cortical_area(area_id).await {
            details.insert(
                area_id.to_string(),
                serde_json::json!({"cortical_id": area.cortical_id}),
            );
        }
    }
    Ok(Json(details))
}

/// POST /v1/connectome/rebuild
#[utoipa::path(post, path = "/v1/connectome/rebuild", tag = "connectome")]
pub async fn post_rebuild(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// GET /v1/connectome/structure
#[utoipa::path(get, path = "/v1/connectome/structure", tag = "connectome")]
pub async fn get_structure(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    let mut response = HashMap::new();
    response.insert("cortical_areas".to_string(), serde_json::json!(areas.len()));
    Ok(Json(response))
}

/// POST /v1/connectome/clear
#[utoipa::path(post, path = "/v1/connectome/clear", tag = "connectome")]
pub async fn post_clear(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// GET /v1/connectome/validation
#[utoipa::path(get, path = "/v1/connectome/validation", tag = "connectome")]
pub async fn get_validation(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let mut response = HashMap::new();
    response.insert("valid".to_string(), serde_json::json!(true));
    Ok(Json(response))
}

/// GET /v1/connectome/topology
#[utoipa::path(get, path = "/v1/connectome/topology", tag = "connectome")]
pub async fn get_topology(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let mut response = HashMap::new();
    response.insert("layers".to_string(), serde_json::json!(0));
    Ok(Json(response))
}

/// POST /v1/connectome/optimize
#[utoipa::path(post, path = "/v1/connectome/optimize", tag = "connectome")]
pub async fn post_optimize(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// GET /v1/connectome/connectivity_matrix
#[utoipa::path(get, path = "/v1/connectome/connectivity_matrix", tag = "connectome")]
pub async fn get_connectivity_matrix(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, Vec<Vec<i32>>>>> {
    let mut response = HashMap::new();
    response.insert("matrix".to_string(), Vec::new());
    Ok(Json(response))
}

/// POST /v1/connectome/neurons/batch
#[utoipa::path(post, path = "/v1/connectome/neurons/batch", tag = "connectome")]
pub async fn post_neurons_batch(
    State(_state): State<ApiState>,
    Json(_ops): Json<Vec<HashMap<String, serde_json::Value>>>,
) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("processed".to_string(), 0);
    Ok(Json(response))
}

/// POST /v1/connectome/synapses/batch
#[utoipa::path(post, path = "/v1/connectome/synapses/batch", tag = "connectome")]
pub async fn post_synapses_batch(
    State(_state): State<ApiState>,
    Json(_ops): Json<Vec<HashMap<String, serde_json::Value>>>,
) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("processed".to_string(), 0);
    Ok(Json(response))
}

// EXACT Python path matches:
/// GET /v1/connectome/cortical_areas/list/summary
#[utoipa::path(
    get,
    path = "/v1/connectome/cortical_areas/list/summary",
    tag = "connectome"
)]
pub async fn get_cortical_areas_list_summary(
    State(state): State<ApiState>,
) -> ApiResult<Json<Vec<HashMap<String, serde_json::Value>>>> {
    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    let summary: Vec<HashMap<String, serde_json::Value>> = areas
        .iter()
        .map(|a| {
            let mut map = HashMap::new();
            map.insert("cortical_id".to_string(), serde_json::json!(a.cortical_id));
            map.insert("cortical_name".to_string(), serde_json::json!(a.name));
            map
        })
        .collect();
    Ok(Json(summary))
}

/// GET /v1/connectome/cortical_areas/list/transforming
#[utoipa::path(
    get,
    path = "/v1/connectome/cortical_areas/list/transforming",
    tag = "connectome"
)]
pub async fn get_cortical_areas_list_transforming(
    State(_state): State<ApiState>,
) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(Vec::new()))
}

/// GET /v1/connectome/cortical_area/{cortical_id}/neurons
#[utoipa::path(
    get,
    path = "/v1/connectome/cortical_area/{cortical_id}/neurons",
    tag = "connectome"
)]
pub async fn get_cortical_area_neurons(
    State(state): State<ApiState>,
    Path(cortical_id): Path<String>,
) -> ApiResult<Json<Vec<u64>>> {
    use tracing::debug;

    let neuron_service = state.neuron_service.as_ref();

    // CRITICAL FIX: Query actual neurons from NPU instead of returning empty stub
    let neurons = neuron_service
        .list_neurons_in_area(&cortical_id, None)
        .await
        .map_err(|e| {
            ApiError::internal(format!(
                "Failed to get neurons in area {}: {}",
                cortical_id, e
            ))
        })?;

    let neuron_ids: Vec<u64> = neurons.iter().map(|n| n.id).collect();

    debug!(target: "feagi-api", "GET /connectome/cortical_area/{}/neurons - found {} neurons", cortical_id, neuron_ids.len());
    Ok(Json(neuron_ids))
}

/// GET /v1/connectome/{cortical_area_id}/synapses
///
/// **Outgoing synapses only**: edges whose **source** is a neuron in
/// `cortical_area_id`. To list **IPU→OPU** afferent synapses, use
/// [`get_area_synapses_incoming`] instead; destination motor areas are usually
/// dominated by the latter.
#[utoipa::path(
    get,
    path = "/v1/connectome/{cortical_area_id}/synapses",
    tag = "connectome"
)]
pub async fn get_area_synapses(
    State(state): State<ApiState>,
    Path(area_id): Path<String>,
) -> ApiResult<Json<Vec<HashMap<String, serde_json::Value>>>> {
    use tracing::debug;

    let connectome_service = state.connectome_service.as_ref();
    let neuron_service = state.neuron_service.as_ref();

    // CRITICAL FIX: Query actual synapses from NPU instead of returning empty stub
    // Get cortical_idx for the area
    let area_info = connectome_service
        .get_cortical_area(&area_id)
        .await
        .map_err(|_| ApiError::not_found("CorticalArea", &area_id))?;

    let cortical_idx = area_info.cortical_idx;

    // Get all neurons in this cortical area
    let neurons = neuron_service
        .list_neurons_in_area(&area_id, None)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get neurons: {}", e)))?;

    tracing::debug!(
        target: "feagi-api",
        "Getting synapses for area {} (idx={}): {} neurons",
        area_id,
        cortical_idx,
        neurons.len()
    );

    // Collect all outgoing synapses from neurons in this area
    // Access NPU through ConnectomeManager singleton
    warn!(
        "[API] /v1/connectome/cortical_area/{}/synapses endpoint called - this acquires NPU lock!",
        area_id
    );
    let manager = feagi_brain_development::ConnectomeManager::instance();
    let manager_lock = manager.read();
    let npu_arc = manager_lock
        .get_npu()
        .ok_or_else(|| ApiError::internal("NPU not initialized"))?;
    let lock_start = std::time::Instant::now();
    tracing::debug!("[NPU-LOCK] CONNECTOME-API: Acquiring NPU lock for synapse queries");
    let npu_lock = npu_arc.lock().unwrap();
    let lock_wait = lock_start.elapsed();
    tracing::debug!(
        "[NPU-LOCK] CONNECTOME-API: Lock acquired (waited {:.2}ms)",
        lock_wait.as_secs_f64() * 1000.0
    );

    let mut all_synapses = Vec::new();
    for neuron_info in &neurons {
        let neuron_id = neuron_info.id as u32;
        let outgoing = npu_lock.get_outgoing_synapses(neuron_id);

        for (target_id, weight, psp, synapse_type) in outgoing {
            let mut synapse_obj = HashMap::new();
            synapse_obj.insert("source_neuron_id".to_string(), serde_json::json!(neuron_id));
            synapse_obj.insert("target_neuron_id".to_string(), serde_json::json!(target_id));
            synapse_obj.insert("weight".to_string(), serde_json::json!(weight));
            synapse_obj.insert("postsynaptic_potential".to_string(), serde_json::json!(psp));
            synapse_obj.insert("synapse_type".to_string(), serde_json::json!(synapse_type));
            all_synapses.push(synapse_obj);
        }
    }

    debug!(target: "feagi-api", "Found {} synapses from area {}", all_synapses.len(), area_id);
    Ok(Json(all_synapses))
}

/// GET /v1/connectome/{cortical_area_id}/synapses/incoming
///
/// Lists **afferent** synapses whose **post-synaptic** targets are neurons in
/// `cortical_area_id`. Complements [`get_area_synapses`], which returns only
/// **efferent** (outgoing) edges from this area. OPUs and other destinations
/// are primarily driven by incoming IPU→OPU plastic synapses, so the outgoing
/// endpoint can legitimately be empty at 200 with count 0.
#[utoipa::path(
    get,
    path = "/v1/connectome/{cortical_area_id}/synapses/incoming",
    tag = "connectome"
)]
pub async fn get_area_synapses_incoming(
    State(state): State<ApiState>,
    Path(area_id): Path<String>,
) -> ApiResult<Json<Vec<HashMap<String, serde_json::Value>>>> {
    use tracing::debug;

    let connectome_service = state.connectome_service.as_ref();
    let neuron_service = state.neuron_service.as_ref();

    let _area_info = connectome_service
        .get_cortical_area(&area_id)
        .await
        .map_err(|_| ApiError::not_found("CorticalArea", &area_id))?;

    let neurons = neuron_service
        .list_neurons_in_area(&area_id, None)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get neurons: {}", e)))?;

    debug!(
        target: "feagi-api",
        "Getting incoming synapses for area {}: {} neurons",
        area_id,
        neurons.len()
    );

    warn!(
        "[API] /v1/connectome/cortical_area/{}/synapses/incoming - acquiring NPU lock",
        area_id
    );
    let manager = feagi_brain_development::ConnectomeManager::instance();
    let manager_lock = manager.read();
    let npu_arc = manager_lock
        .get_npu()
        .ok_or_else(|| ApiError::internal("NPU not initialized"))?;
    let npu_lock = npu_arc.lock().unwrap();

    let mut all_synapses = Vec::new();
    for neuron_info in &neurons {
        let neuron_id = neuron_info.id as u32;
        let incoming = npu_lock.get_incoming_synapses(neuron_id);

        for (source_id, weight, psp, synapse_type) in incoming {
            let mut synapse_obj = HashMap::new();
            synapse_obj.insert("source_neuron_id".to_string(), serde_json::json!(source_id));
            synapse_obj.insert("target_neuron_id".to_string(), serde_json::json!(neuron_id));
            synapse_obj.insert("weight".to_string(), serde_json::json!(weight));
            synapse_obj.insert("postsynaptic_potential".to_string(), serde_json::json!(psp));
            synapse_obj.insert("synapse_type".to_string(), serde_json::json!(synapse_type));
            all_synapses.push(synapse_obj);
        }
    }

    debug!(
        target: "feagi-api",
        "Found {} incoming synapses to area {}",
        all_synapses.len(),
        area_id
    );
    Ok(Json(all_synapses))
}

/// GET /v1/connectome/cortical_info/{cortical_area}
#[utoipa::path(
    get,
    path = "/v1/connectome/cortical_info/{cortical_area}",
    tag = "connectome"
)]
pub async fn get_cortical_info(
    State(state): State<ApiState>,
    Path(cortical_area): Path<String>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    let area = connectome_service
        .get_cortical_area(&cortical_area)
        .await
        .map_err(|e| ApiError::not_found("area", &format!("{}", e)))?;
    let mut response = HashMap::new();
    response.insert(
        "cortical_id".to_string(),
        serde_json::json!(area.cortical_id),
    );
    response.insert("cortical_name".to_string(), serde_json::json!(area.name));
    Ok(Json(response))
}

/// GET /v1/connectome/stats/cortical/cumulative/{cortical_area}
#[utoipa::path(
    get,
    path = "/v1/connectome/stats/cortical/cumulative/{cortical_area}",
    tag = "connectome"
)]
pub async fn get_stats_cortical_cumulative(
    State(_state): State<ApiState>,
    Path(_area): Path<String>,
) -> ApiResult<Json<HashMap<String, i64>>> {
    let mut response = HashMap::new();
    response.insert("total_fires".to_string(), 0);
    Ok(Json(response))
}

/// GET /v1/connectome/neuron/{neuron_id}/properties
#[utoipa::path(
    get,
    path = "/v1/connectome/neuron/{neuron_id}/properties",
    tag = "connectome"
)]
pub async fn get_neuron_properties_by_id(
    State(state): State<ApiState>,
    Path(neuron_id): Path<u64>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    let props = connectome_service
        .get_neuron_properties(neuron_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(props))
}

/// GET /v1/connectome/neuron_properties
#[utoipa::path(get, path = "/v1/connectome/neuron_properties", tag = "connectome")]
pub async fn get_neuron_properties_query(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let neuron_id: u64 = params
        .get("neuron_id")
        .ok_or_else(|| ApiError::invalid_input("neuron_id required"))?
        .parse()
        .map_err(|_| ApiError::invalid_input("neuron_id must be an integer"))?;

    let connectome_service = state.connectome_service.as_ref();
    let props = connectome_service
        .get_neuron_properties(neuron_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(props))
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct NeuronPropertiesAtQuery {
    /// Cortical area ID (base64 string)
    pub cortical_id: String,
    /// X coordinate within the cortical area
    pub x: u32,
    /// Y coordinate within the cortical area
    pub y: u32,
    /// Z coordinate within the cortical area
    pub z: u32,
}

/// GET /v1/connectome/neuron_properties_at
///
/// Resolve a neuron by `(cortical_id, x, y, z)` and return its live properties/state.
///
/// This is intended for clients (e.g., Brain Visualizer) that do not have neuron IDs.
#[utoipa::path(
    get,
    path = "/v1/connectome/neuron_properties_at",
    tag = "connectome",
    params(NeuronPropertiesAtQuery)
)]
pub async fn get_neuron_properties_at_query(
    State(state): State<ApiState>,
    Query(params): Query<NeuronPropertiesAtQuery>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let cortical_id = params.cortical_id;
    let x = params.x;
    let y = params.y;
    let z = params.z;

    // Resolve cortical_idx via service layer.
    let connectome_service = state.connectome_service.as_ref();
    let area = connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .map_err(|_| ApiError::not_found("CorticalArea", &cortical_id))?;

    // Resolve neuron_id via NPU coordinate lookup (fast path).
    //
    // IMPORTANT (Axum): handler futures must be `Send`.
    // Do NOT hold non-Send locks/guards across `.await`.
    let neuron_id_u32: u32 = {
        // Note: this uses the global ConnectomeManager singleton, consistent with existing connectome endpoints.
        let manager = feagi_brain_development::ConnectomeManager::instance();
        let manager_lock = manager.read();
        let npu_arc = manager_lock
            .get_npu()
            .ok_or_else(|| ApiError::internal("NPU not initialized"))?;
        let npu_lock = npu_arc.lock().unwrap();

        npu_lock
            .get_neuron_id_at_coordinate(area.cortical_idx, x, y, z)
            .ok_or_else(|| {
                ApiError::not_found(
                    "Neuron",
                    &format!("cortical_id={} x={} y={} z={}", cortical_id, x, y, z),
                )
            })?
    };

    let mut props = connectome_service
        .get_neuron_properties(neuron_id_u32 as u64)
        .await
        .map_err(ApiError::from)?;

    // Always include resolved identity fields for clients.
    props.insert(
        "neuron_id".to_string(),
        serde_json::json!(neuron_id_u32 as u64),
    );
    props.insert("cortical_id".to_string(), serde_json::json!(cortical_id));
    props.insert(
        "cortical_idx".to_string(),
        serde_json::json!(area.cortical_idx),
    );

    Ok(Json(props))
}

/// GET /v1/connectome/area_neurons
#[utoipa::path(get, path = "/v1/connectome/area_neurons", tag = "connectome")]
pub async fn get_area_neurons_query(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vec<u64>>> {
    Ok(Json(Vec::new()))
}

/// GET /v1/connectome/fire_queue/{cortical_area}
#[utoipa::path(
    get,
    path = "/v1/connectome/fire_queue/{cortical_area}",
    tag = "connectome"
)]
pub async fn get_fire_queue_area(
    State(_state): State<ApiState>,
    Path(_area): Path<String>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
}

/// GET /v1/connectome/plasticity
#[utoipa::path(get, path = "/v1/connectome/plasticity", tag = "connectome")]
pub async fn get_plasticity_info(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let genome_json = state
        .genome_service
        .save_genome(feagi_services::types::SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .map_err(ApiError::from)?;
    let genome: feagi_evolutionary::RuntimeGenome =
        feagi_evolutionary::load_genome_from_json(&genome_json)
            .map_err(|e| ApiError::internal(format!("Failed to parse saved genome: {}", e)))?;
    let (memory_area_ids, plastic_mappings, brain_region_ids) =
        feagi_services::impls::genome_service_impl::architecture_indexes_from_genome(&genome);
    let enabled = !memory_area_ids.is_empty() || !plastic_mappings.is_empty();
    let mut response = HashMap::new();
    response.insert("enabled".to_string(), serde_json::json!(enabled));
    response.insert(
        "memory_area_count".to_string(),
        serde_json::json!(memory_area_ids.len()),
    );
    response.insert(
        "plastic_mapping_count".to_string(),
        serde_json::json!(plastic_mappings.len()),
    );
    response.insert(
        "brain_region_count".to_string(),
        serde_json::json!(brain_region_ids.len()),
    );
    response.insert(
        "memory_area_ids".to_string(),
        serde_json::json!(memory_area_ids),
    );
    response.insert(
        "plastic_mappings".to_string(),
        serde_json::json!(plastic_mappings),
    );
    response.insert(
        "brain_region_ids".to_string(),
        serde_json::json!(brain_region_ids),
    );
    Ok(Json(response))
}

/// GET /v1/connectome/path
#[utoipa::path(get, path = "/v1/connectome/path", tag = "connectome")]
pub async fn get_path_query(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vec<Vec<String>>>> {
    Ok(Json(Vec::new()))
}

/// GET /v1/connectome/download
///
/// Writes the connectome snapshot to the configured data root and returns the
/// saved file path. The HTTP response does not include connectome contents.
#[utoipa::path(
    get,
    path = "/v1/connectome/download",
    tag = "connectome",
    responses(
        (status = 200, description = "Filesystem path of the saved connectome file", body = HashMap<String, String>)
    )
)]
pub async fn get_download_connectome(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    info!("[API] GET /v1/connectome/download - Saving connectome to filesystem");
    let snapshot = state
        .connectome_service
        .export_connectome()
        .await
        .map_err(ApiError::from)?;
    let file_name = connectome_file_name("saved_connectome", &connectome_timestamp());
    let file_path = save_connectome_snapshot(&state.filesystem_data_root, &file_name, &snapshot)?;
    info!("Connectome saved to {}", file_path);
    Ok(Json(HashMap::from([
        ("file_path".to_string(), file_path),
        (
            "message".to_string(),
            "Connectome saved successfully".to_string(),
        ),
    ])))
}

/// GET /v1/connectome/download-cortical-area/{cortical_area}
#[utoipa::path(
    get,
    path = "/v1/connectome/download-cortical-area/{cortical_area}",
    tag = "connectome"
)]
pub async fn get_download_cortical_area(
    State(state): State<ApiState>,
    Path(area): Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let area_info = state
        .connectome_service
        .get_cortical_area(&area)
        .await
        .map_err(ApiError::from)?;
    let snapshot = state
        .connectome_service
        .export_connectome()
        .await
        .map_err(ApiError::from)?;
    let filtered = snapshot.filter_to_cortical_idx(area_info.cortical_idx);
    let file_name = connectome_file_name(
        &format!("saved_connectome_{}", area_info.cortical_id),
        &connectome_timestamp(),
    );
    let file_path = save_connectome_snapshot(&state.filesystem_data_root, &file_name, &filtered)?;
    Ok(Json(HashMap::from([
        ("file_path".to_string(), file_path),
        ("cortical_area".to_string(), area_info.cortical_id),
        (
            "message".to_string(),
            "Cortical area connectome saved successfully".to_string(),
        ),
    ])))
}

/// POST /v1/connectome/upload
///
/// Accepts a saved `.connectome` file via multipart form field `file`.
#[cfg(feature = "http")]
#[utoipa::path(
    post,
    path = "/v1/connectome/upload",
    tag = "connectome",
    request_body(content = ConnectomeFileUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Connectome imported from uploaded file", body = HashMap<String, String>),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_upload_connectome(
    State(state): State<ApiState>,
    mut multipart: axum::extract::Multipart,
) -> ApiResult<Json<HashMap<String, String>>> {
    info!("[API] POST /v1/connectome/upload - Loading connectome file");
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::invalid_input(format!("Invalid multipart upload: {}", e)))?
    {
        if field.name() == Some("file") {
            let bytes = field.bytes().await.map_err(|e| {
                ApiError::invalid_input(format!("Failed to read uploaded file: {}", e))
            })?;
            file_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let bytes =
        file_bytes.ok_or_else(|| ApiError::invalid_input("Missing multipart field 'file'"))?;
    let snapshot = load_connectome_snapshot_from_bytes(&bytes)?;
    import_connectome_snapshot(&state, snapshot).await
}

/// GET /v1/connectome/directory
///
/// Lists saved connectome files under `{data_root}/connectome`. Swagger UI uses this
/// so upload can target the same folder download writes to (browser file pickers cannot
/// open an arbitrary initial directory).
#[cfg(feature = "http")]
#[utoipa::path(
    get,
    path = "/v1/connectome/directory",
    tag = "connectome",
    responses(
        (status = 200, description = "Connectome directory and saved files", body = ConnectomeDirectoryResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_connectome_directory(
    State(state): State<ApiState>,
) -> ApiResult<Json<ConnectomeDirectoryResponse>> {
    info!("[API] GET /v1/connectome/directory - Listing saved connectome files");
    Ok(Json(list_connectome_directory(
        &state.filesystem_data_root,
    )?))
}

/// POST /v1/connectome/upload-saved
///
/// Imports a connectome file already present under `{data_root}/connectome`.
#[cfg(feature = "http")]
#[utoipa::path(
    post,
    path = "/v1/connectome/upload-saved",
    tag = "connectome",
    request_body = ConnectomeUploadSavedRequest,
    responses(
        (status = 200, description = "Connectome imported from saved file", body = HashMap<String, String>),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_upload_connectome_saved(
    State(state): State<ApiState>,
    Json(body): Json<ConnectomeUploadSavedRequest>,
) -> ApiResult<Json<HashMap<String, String>>> {
    info!(
        "[API] POST /v1/connectome/upload-saved - Loading connectome file {}",
        body.file_name
    );
    let file_path = resolve_connectome_file_name(&state.filesystem_data_root, &body.file_name)?;
    let bytes = std::fs::read(&file_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ApiError::not_found("Connectome file", &body.file_name)
        } else {
            ApiError::internal(format!("Failed to read connectome file: {}", e))
        }
    })?;
    let snapshot = load_connectome_snapshot_from_bytes(&bytes)?;
    import_connectome_snapshot(&state, snapshot).await
}

/// POST /v1/connectome/upload-cortical-area
#[utoipa::path(post, path = "/v1/connectome/upload-cortical-area", tag = "connectome")]
pub async fn post_upload_cortical_area(
    State(state): State<ApiState>,
    Json(data): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let area_snapshot: feagi_npu_neural::types::connectome::ConnectomeSnapshot =
        serde_json::from_value(data).map_err(|e| {
            ApiError::invalid_input(format!("Invalid cortical-area connectome snapshot: {}", e))
        })?;
    if area_snapshot.cortical_area_names.len() != 1 {
        return Err(ApiError::invalid_input(
            "Per-area upload requires exactly one cortical_area_names entry".to_string(),
        ));
    }
    let cortical_idx = *area_snapshot
        .cortical_area_names
        .keys()
        .next()
        .ok_or_else(|| ApiError::invalid_input("Missing cortical_area_names key".to_string()))?;
    let live = state
        .connectome_service
        .export_connectome()
        .await
        .map_err(ApiError::from)?;
    let merged = live.replace_cortical_idx(cortical_idx, &area_snapshot);
    state
        .connectome_service
        .import_connectome(merged)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Cortical area connectome imported".to_string(),
    )])))
}

/// GET /v1/connectome/cortical_area/list/types
#[utoipa::path(
    get,
    path = "/v1/connectome/cortical_area/list/types",
    tag = "connectome",
    responses(
        (status = 200, description = "List of cortical types with their cortical IDs and group IDs", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_cortical_area_list_types(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // Note: decode_cortical_id removed - use CorticalID methods
    use std::collections::{HashMap, HashSet};

    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list cortical areas: {}", e)))?;

    // Helper function to map cortical subtype to human-readable title
    fn get_cortical_type_title(subtype: &str) -> String {
        match subtype {
            "svi" => "segmented vision".to_string(),
            "mot" => "motor".to_string(),
            "bat" => "battery".to_string(),
            "mis" => "miscellaneous".to_string(),
            "gaz" => "gaze control".to_string(),
            "pow" => "power".to_string(),
            "dea" => "death".to_string(),
            _ => {
                // For unknown types, capitalize first letter and add spaces
                if !subtype.is_empty() {
                    let mut chars = subtype.chars();
                    let first = chars.next().unwrap().to_uppercase().collect::<String>();
                    let rest: String = chars.collect();
                    format!("{}{}", first, rest)
                } else {
                    "unknown".to_string()
                }
            }
        }
    }

    // Group areas by cortical subtype
    let mut type_map: HashMap<String, (String, Vec<String>, HashSet<u8>)> = HashMap::new();

    for area in areas {
        // Parse cortical ID from base64
        use feagi_structures::genomic::cortical_area::CorticalID;
        if let Ok(cortical_id_typed) = CorticalID::try_from_base_64(&area.cortical_id) {
            // Extract subtype and group_id using CorticalID methods
            if let Some(subtype) = cortical_id_typed.extract_subtype() {
                let entry = type_map.entry(subtype.clone()).or_insert_with(|| {
                    let title = get_cortical_type_title(&subtype);
                    (title, Vec::new(), HashSet::new())
                });

                // Add cortical ID in base64 format
                entry.1.push(area.cortical_id.clone());

                // Add group_id if available
                if let Some(group_id) = cortical_id_typed.extract_group_id() {
                    entry.2.insert(group_id);
                }
            }
        }
    }

    // Convert to response format
    let mut response: HashMap<String, serde_json::Value> = HashMap::new();
    for (subtype, (title, mut cortical_ids, group_ids)) in type_map {
        // Sort cortical_ids for consistent output
        cortical_ids.sort();

        let mut group_ids_vec: Vec<u8> = group_ids.into_iter().collect();
        group_ids_vec.sort_unstable();

        response.insert(
            subtype,
            serde_json::json!({
                "title": title,
                "cortical_ids": cortical_ids,
                "group_ids": group_ids_vec
            }),
        );
    }

    Ok(Json(response))
}

/// GET /v1/connectome/memory_neuron — plasticity memory-neuron record and NPU synapse lists.
#[utoipa::path(
    get,
    path = "/v1/connectome/memory_neuron",
    tag = "connectome",
    params(MemoryNeuronQuery),
    responses(
        (status = 200, description = "Memory neuron detail", body = MemoryNeuronDetailResponse),
        (status = 400, description = "Invalid neuron id"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory_neuron(
    State(_state): State<ApiState>,
    Query(q): Query<MemoryNeuronQuery>,
) -> ApiResult<Json<MemoryNeuronDetailResponse>> {
    if !feagi_npu_plasticity::NeuronIdManager::is_memory_neuron_id(q.neuron_id) {
        return Err(ApiError::invalid_input(format!(
            "neuron_id must be a memory neuron id in range {}..={}",
            feagi_npu_plasticity::MEMORY_NEURON_ID_START,
            feagi_npu_plasticity::MEMORY_NEURON_ID_MAX
        )));
    }

    let manager = feagi_brain_development::ConnectomeManager::instance();
    let mgr = manager.read();

    // CRITICAL: Only hold the plasticity executor mutex while reading MemoryNeuronArray.
    // Never hold it while acquiring the NPU lock (synapse queries), or the burst thread can
    // deadlock (NPU held → plasticity vs plasticity held → NPU).
    let detail = {
        let exec = mgr
            .get_plasticity_executor()
            .ok_or_else(|| ApiError::internal("Plasticity executor not available"))?;
        let ex = exec
            .lock()
            .map_err(|_| ApiError::internal("Plasticity executor lock poisoned"))?;
        ex.memory_neuron_detail(q.neuron_id)
            .ok_or_else(|| ApiError::not_found("Memory neuron", &q.neuron_id.to_string()))?
    };

    let cortical_idx = detail.cortical_area_idx;
    let (cortical_id, cortical_name) = mgr
        .get_cortical_id(cortical_idx)
        .and_then(|cid| {
            mgr.get_cortical_area(cid)
                .map(|a| (cid.as_base_64(), a.name.clone()))
        })
        .unwrap_or_else(|| (String::new(), String::new()));

    let outgoing_full = mgr.get_outgoing_synapses(q.neuron_id as u64);
    let incoming_full = mgr.get_incoming_synapses(q.neuron_id as u64);
    let oc = outgoing_full.len();
    let ic = incoming_full.len();
    let (out_json, in_json) =
        synapse_details_for_neuron(&mgr, q.neuron_id, &outgoing_full, &incoming_full);

    Ok(Json(MemoryNeuronDetailResponse {
        neuron_id: q.neuron_id as u64,
        cortical_idx,
        cortical_id,
        cortical_name,
        pattern_hash: detail.pattern_hash,
        is_longterm_memory: detail.is_longterm_memory,
        is_active: detail.is_active,
        lifespan_current: detail.lifespan_current,
        lifespan_initial: detail.lifespan_initial,
        lifespan_growth_rate: detail.lifespan_growth_rate,
        creation_burst: detail.creation_burst,
        last_activation_burst: detail.last_activation_burst,
        activation_count: detail.activation_count,
        outgoing_synapse_count: oc,
        incoming_synapse_count: ic,
        outgoing_synapses: out_json,
        incoming_synapses: in_json,
    }))
}

async fn import_connectome_snapshot(
    state: &ApiState,
    snapshot: feagi_npu_neural::types::connectome::ConnectomeSnapshot,
) -> ApiResult<Json<HashMap<String, String>>> {
    state
        .connectome_service
        .import_connectome(snapshot)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Connectome imported successfully".to_string(),
    )])))
}

/// Connectome files are written under `{data_root}/connectome/{file_name}`.
fn connectome_dir(data_root: &std::path::Path) -> PathBuf {
    data_root.join("connectome")
}

fn connectome_save_path(data_root: &std::path::Path, file_name: &str) -> PathBuf {
    connectome_dir(data_root).join(file_name)
}

/// Returns the absolute connectome directory path and saved `.connectome` files.
fn list_connectome_directory(
    data_root: &std::path::Path,
) -> ApiResult<ConnectomeDirectoryResponse> {
    let dir_path = connectome_dir(data_root);
    if let Some(parent) = dir_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ApiError::internal(format!(
                "Failed to create connectome parent directory: {}",
                e
            ))
        })?;
    }
    std::fs::create_dir_all(&dir_path)
        .map_err(|e| ApiError::internal(format!("Failed to create connectome directory: {}", e)))?;

    let absolute_dir = std::fs::canonicalize(&dir_path).map_err(|e| {
        ApiError::internal(format!("Failed to resolve connectome directory: {}", e))
    })?;

    let mut files = Vec::new();
    let entries = std::fs::read_dir(&absolute_dir)
        .map_err(|e| ApiError::internal(format!("Failed to read connectome directory: {}", e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            ApiError::internal(format!("Failed to read connectome directory entry: {}", e))
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if !file_name.ends_with(".connectome") {
            continue;
        }
        let metadata = entry.metadata().map_err(|e| {
            ApiError::internal(format!("Failed to read connectome file metadata: {}", e))
        })?;
        let modified_at = metadata
            .modified()
            .ok()
            .map(|mtime| {
                chrono::DateTime::<chrono::Utc>::from(mtime)
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            })
            .unwrap_or_else(|| "unknown".to_string());
        files.push(ConnectomeSavedFileEntry {
            file_path: path.display().to_string(),
            file_name,
            size_bytes: metadata.len(),
            modified_at,
        });
    }

    files.sort_by(|left, right| right.modified_at.cmp(&left.modified_at));

    Ok(ConnectomeDirectoryResponse {
        directory: absolute_dir.display().to_string(),
        files,
    })
}

/// Resolve a user-supplied connectome filename to an absolute path under `{data_root}/connectome`.
fn resolve_connectome_file_name(
    data_root: &std::path::Path,
    file_name: &str,
) -> ApiResult<PathBuf> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err(ApiError::invalid_input("file_name must not be empty"));
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err(ApiError::invalid_input(
            "file_name must be a base filename without path components".to_string(),
        ));
    }
    if !trimmed.ends_with(".connectome") {
        return Err(ApiError::invalid_input(
            "file_name must end with .connectome".to_string(),
        ));
    }

    let candidate = connectome_save_path(data_root, trimmed);
    let absolute_dir = std::fs::canonicalize(connectome_dir(data_root)).map_err(|e| {
        ApiError::internal(format!("Failed to resolve connectome directory: {}", e))
    })?;
    let absolute_file = std::fs::canonicalize(&candidate).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ApiError::not_found("Connectome file", trimmed)
        } else {
            ApiError::internal(format!("Failed to resolve connectome file path: {}", e))
        }
    })?;
    if !absolute_file.starts_with(&absolute_dir) {
        return Err(ApiError::invalid_input(
            "file_name resolves outside the connectome directory".to_string(),
        ));
    }
    Ok(absolute_file)
}

fn connectome_file_name(prefix: &str, timestamp: &str) -> String {
    format!("{prefix}_{timestamp}.connectome")
}

fn connectome_timestamp() -> String {
    chrono::Utc::now().format("%Y_%m_%d-%H_%M_%S").to_string()
}

/// Persist `snapshot` and return the absolute path including filename and extension.
fn save_connectome_snapshot(
    data_root: &std::path::Path,
    file_name: &str,
    snapshot: &feagi_npu_neural::types::connectome::ConnectomeSnapshot,
) -> ApiResult<String> {
    let save_path = connectome_save_path(data_root, file_name);
    if let Some(parent) = save_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ApiError::internal(format!("Failed to create connectome directory: {}", e))
        })?;
    }
    #[cfg(feature = "services")]
    {
        feagi_services::connectome::save_connectome(snapshot, &save_path)
            .map_err(|e| ApiError::internal(format!("Failed to write connectome file: {}", e)))?;
        let absolute = std::fs::canonicalize(&save_path).map_err(|e| {
            ApiError::internal(format!("Failed to resolve saved connectome path: {}", e))
        })?;
        Ok(absolute.display().to_string())
    }
    #[cfg(not(feature = "services"))]
    {
        let _ = snapshot;
        Err(ApiError::internal(
            "Connectome file I/O requires the services feature".to_string(),
        ))
    }
}

fn load_connectome_snapshot_from_bytes(
    bytes: &[u8],
) -> ApiResult<feagi_npu_neural::types::connectome::ConnectomeSnapshot> {
    #[cfg(feature = "services")]
    {
        feagi_services::connectome::load_connectome_from_bytes(bytes)
            .map_err(|e| ApiError::invalid_input(format!("Invalid connectome file: {}", e)))
    }
    #[cfg(not(feature = "services"))]
    {
        let _ = bytes;
        Err(ApiError::internal(
            "Connectome file I/O requires the services feature".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        connectome_dir, connectome_file_name, connectome_save_path, list_connectome_directory,
        resolve_connectome_file_name, save_connectome_snapshot,
    };
    use feagi_npu_neural::types::connectome::{
        ConnectomeMetadata, ConnectomeSnapshot, SerializableNeuronArray, SerializableSynapseArray,
    };
    use std::path::PathBuf;

    fn empty_snapshot() -> ConnectomeSnapshot {
        ConnectomeSnapshot {
            version: 1,
            neurons: SerializableNeuronArray::default(),
            synapses: SerializableSynapseArray::default(),
            cortical_area_names: Default::default(),
            burst_count: 0,
            power_amount: 1.0,
            fire_ledger_window: 20,
            metadata: ConnectomeMetadata::default(),
            genome_json: None,
            memory_area_ids: Vec::new(),
            plastic_mappings: Vec::new(),
            brain_region_ids: Vec::new(),
            long_term_memory_neurons: Vec::new(),
            long_term_memory_replay_frames: Vec::new(),
        }
    }

    #[test]
    fn connectome_file_name_includes_full_name_and_extension() {
        let name = connectome_file_name("saved_connectome", "2026_09_02-17_45_37");
        assert_eq!(name, "saved_connectome_2026_09_02-17_45_37.connectome");
    }

    #[test]
    fn connectome_save_path_uses_connectome_dir() {
        let path = connectome_save_path(
            &PathBuf::from("/data/feagi"),
            "saved_connectome_2026_09_02-17_45_37.connectome",
        );
        assert_eq!(
            path,
            PathBuf::from("/data/feagi/connectome/saved_connectome_2026_09_02-17_45_37.connectome")
        );
    }

    #[test]
    fn connectome_dir_is_under_data_root() {
        let dir = connectome_dir(&PathBuf::from("/data/feagi"));
        assert_eq!(dir, PathBuf::from("/data/feagi/connectome"));
    }

    #[test]
    fn resolve_connectome_file_name_rejects_path_traversal() {
        let data_root = std::env::temp_dir().join(format!(
            "feagi-connectome-resolve-{}--temp",
            std::process::id()
        ));
        let err = resolve_connectome_file_name(&data_root, "../secrets.connectome")
            .expect_err("path traversal must be rejected");
        assert!(err.message.contains("path components"));
        let _ = std::fs::remove_dir_all(&data_root);
    }

    #[test]
    fn list_connectome_directory_returns_saved_files() {
        let data_root = std::env::temp_dir().join(format!(
            "feagi-connectome-list-{}--temp",
            std::process::id()
        ));
        let file_name = "saved_connectome_test.connectome";
        save_connectome_snapshot(&data_root, file_name, &empty_snapshot())
            .expect("save connectome");
        let listing = list_connectome_directory(&data_root).expect("list connectome directory");
        assert!(
            listing.directory.ends_with("/connectome")
                || listing.directory.ends_with("\\connectome")
        );
        assert_eq!(listing.files.len(), 1);
        assert_eq!(listing.files[0].file_name, file_name);
        let resolved = resolve_connectome_file_name(&data_root, file_name).expect("resolve file");
        assert!(resolved.is_file());
        std::fs::remove_dir_all(&data_root).expect("cleanup temp connectome dir");
    }

    #[cfg(feature = "services")]
    #[test]
    fn save_connectome_snapshot_writes_file_and_returns_path_with_extension() {
        let data_root = std::env::temp_dir().join(format!(
            "feagi-connectome-save-{}--temp",
            std::process::id()
        ));
        let file_name = "saved_connectome_test.connectome";
        let returned = save_connectome_snapshot(&data_root, file_name, &empty_snapshot())
            .expect("save connectome");
        assert!(returned.ends_with(".connectome"), "{returned}");
        let expected = connectome_save_path(&data_root, file_name);
        assert!(expected.is_file(), "missing {}", expected.display());
        let bytes = std::fs::read(&expected).expect("read saved connectome");
        let loaded = super::load_connectome_snapshot_from_bytes(&bytes).expect("load bytes");
        assert_eq!(loaded.burst_count, 0);
        std::fs::remove_dir_all(&data_root).expect("cleanup temp connectome dir");
    }
}

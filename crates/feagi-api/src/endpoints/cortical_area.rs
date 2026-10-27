// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Cortical Area API Endpoints - Exact port from Python `/v1/cortical_area/*`
//!
//! Reference: feagi-py/feagi/api/v1/cortical_area.py

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, Query, State};
use feagi_evolutionary::extract_memory_properties;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalSubUnitIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::IOCorticalAreaConfigurationFlag;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use utoipa::{IntoParams, ToSchema};

// ============================================================================
// REQUEST/RESPONSE MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaIdListResponse {
    pub cortical_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaNameListResponse {
    pub cortical_area_name_list: Vec<String>,
}

/// Body for `PUT /v1/cortical_area/reset` (matches Brain Visualizer `area_list` payload).
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaResetRequest {
    pub area_list: Vec<String>,
}

/// Per-area outcome for cortical reset.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaResetItem {
    pub cortical_idx: u32,
    pub neurons_reset: usize,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaResetResponse {
    pub message: String,
    pub results: Vec<CorticalAreaResetItem>,
}

/// Per-area outcome for incoming-weight reset.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaResetWeightsItem {
    pub cortical_id: String,
    pub mappings_reset: usize,
    pub synapses_reset: usize,
}

/// Response for `PUT /v1/cortical_area/reset_weights`.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalAreaResetWeightsResponse {
    pub results: Vec<CorticalAreaResetWeightsItem>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UnitTopologyData {
    pub relative_position: [i32; 3],
    pub dimensions: [u32; 3],
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorticalTypeMetadata {
    pub description: String,
    pub encodings: Vec<String>,
    pub formats: Vec<String>,
    pub units: u32,
    pub resolution: Vec<i32>,
    pub structure: String,
    pub unit_default_topology: HashMap<usize, UnitTopologyData>,
}

/// Maximum outgoing **or** incoming synapse detail objects per request page (100 synapse records max per neuron: 50 + 50).
pub const VOXEL_NEURON_SYNAPSES_PER_DIRECTION_PER_PAGE: usize = 50;

/// Returns `(range_start, range_end, has_more)` for a 0-based `page` over `total` items (`page_size` per page).
fn synapse_page_window(total: usize, page: u32) -> (usize, usize, bool) {
    let page = page as usize;
    let page_size = VOXEL_NEURON_SYNAPSES_PER_DIRECTION_PER_PAGE;
    let start = page.saturating_mul(page_size);
    if start >= total {
        return (0, 0, false);
    }
    let end = (start + page_size).min(total);
    let has_more = total > end;
    (start, end, has_more)
}

/// Query parameters for [`get_voxel_neurons`] (same coordinate space as `/v1/connectome/neuron_properties_at`).
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct VoxelNeuronsQuery {
    /// Cortical area ID (base64-encoded string, e.g. from genome).
    pub cortical_id: String,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    /// 0-based page for synapse detail lists: at most [`VOXEL_NEURON_SYNAPSES_PER_DIRECTION_PER_PAGE`] outgoing and the same count incoming per page.
    #[serde(default)]
    pub synapse_page: u32,
}

/// JSON body for [`post_voxel_neurons`] (same fields as [`VoxelNeuronsQuery`]).
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct VoxelNeuronsBody {
    pub cortical_id: String,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    #[serde(default)]
    pub synapse_page: u32,
}

/// Default page size for [`MemoryCorticalAreaQuery::page_size`].
pub const MEMORY_CORTICAL_NEURON_IDS_PAGE_SIZE_DEFAULT: u32 = 50;
/// Maximum allowed page size for memory neuron id list pagination.
pub const MEMORY_CORTICAL_NEURON_IDS_PAGE_SIZE_MAX: u32 = 500;

fn default_memory_cortical_page_size() -> u32 {
    MEMORY_CORTICAL_NEURON_IDS_PAGE_SIZE_DEFAULT
}

/// Query parameters for [`get_memory_cortical_area`].
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MemoryCorticalAreaQuery {
    /// Base64 cortical area id for a memory area.
    pub cortical_id: String,
    /// 0-based page index for `memory_neuron_ids`.
    #[serde(default)]
    pub page: u32,
    /// Page size for `memory_neuron_ids` (clamped to [`MEMORY_CORTICAL_NEURON_IDS_PAGE_SIZE_MAX`]).
    #[serde(default = "default_memory_cortical_page_size")]
    pub page_size: u32,
}

/// Lifecycle values the area record already resolved, plus the keys that stay in
/// `properties` after duplicate top-level fields are removed.
fn memory_parameters_from_reported_area(
    area: &feagi_services::types::CorticalAreaInfo,
    extracted: &feagi_evolutionary::MemoryAreaProperties,
) -> MemoryCorticalAreaParamsResponse {
    MemoryCorticalAreaParamsResponse {
        temporal_depth: area
            .temporal_depth
            .unwrap_or(extracted.temporal_depth)
            .max(1),
        longterm_mem_threshold: area.longterm_mem_threshold,
        lifespan_growth_rate: area.lifespan_growth_rate as f32,
        init_lifespan: area.init_lifespan,
        mp_learning_enabled: area
            .mp_learning_enabled
            .unwrap_or(extracted.mp_learning_enabled),
        mp_change_mode: area
            .mp_change_mode
            .clone()
            .unwrap_or_else(|| extracted.mp_change_mode.as_str().to_string()),
        mp_delta_quantization: area
            .mp_delta_quantization
            .unwrap_or(extracted.mp_delta_quantization),
        mp_ratio_quantization: area
            .mp_ratio_quantization
            .unwrap_or(extracted.mp_ratio_quantization),
        min_window_activity: extracted.min_window_activity,
        scan_skip_density: extracted.scan_skip_density,
        memory_neuron_cap: area
            .memory_neuron_cap
            .unwrap_or(extracted.memory_neuron_cap),
        ltm_eviction: area
            .ltm_eviction
            .clone()
            .unwrap_or_else(|| extracted.ltm_eviction.as_str().to_string()),
    }
}

/// Genome memory parameters for the cortical area (from `extract_memory_properties`).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MemoryCorticalAreaParamsResponse {
    pub temporal_depth: u32,
    pub longterm_mem_threshold: u32,
    pub lifespan_growth_rate: f32,
    pub init_lifespan: u32,
    pub mp_learning_enabled: bool,
    /// Change-based MP encoding: "none", "mp_differential", or "mp_ratio".
    pub mp_change_mode: String,
    /// Differential rounding step in MP units.
    pub mp_delta_quantization: f32,
    /// Ratio rounding step in percent per compounding bucket.
    pub mp_ratio_quantization: f32,
    pub min_window_activity: u32,
    pub scan_skip_density: f32,
    /// Most memory neurons (short-term plus long-term) this area may hold. `0` means no
    /// per-area cap; only the shared pool limits the area.
    pub memory_neuron_cap: u32,
    /// What the area does when its cap or the shared pool is full: "lru" evicts one of its
    /// neurons to make room, "none" lets the creation fail.
    pub ltm_eviction: String,
}

/// Occupancy of the pool shared by all memory areas.
#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize, ToSchema)]
pub struct MemoryPoolResponse {
    /// Slots in the pool.
    pub pool_capacity: usize,
    /// Active memory neurons across all areas.
    pub pool_in_use: usize,
    /// Active long-term memory neurons across all areas.
    pub pool_ltm: usize,
}

/// Response for [`get_memory_cortical_area`].
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MemoryCorticalAreaResponse {
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub cortical_name: String,
    pub short_term_neuron_count: usize,
    pub long_term_neuron_count: usize,
    pub memory_parameters: MemoryCorticalAreaParamsResponse,
    /// Neurons evicted from this area to make room for new ones since the area was created.
    pub evicted_total: usize,
    /// New-pattern creations in this area that failed because no slot could be freed.
    pub creation_failed_total: usize,
    /// New-pattern creations skipped because a subscribed `memory.formation` modulator is
    /// suppressing formation. Not failures.
    pub formation_suppressed_total: usize,
    /// Occupancy of the pool shared by all memory areas.
    pub memory_pool: MemoryPoolResponse,
    /// Mode the plasticity layer actually runs: "pattern_only", "mp_learning",
    /// "mp_differential", or "mp_ratio". Differs from `memory_parameters` when a change
    /// mode was auto-disabled (temporal_depth < 2). `None` if the area is not registered.
    pub effective_mp_mode: Option<String>,
    /// Upstream cortical indices that feed pattern detection for this memory area.
    pub upstream_cortical_area_indices: Vec<u32>,
    pub upstream_cortical_area_count: usize,
    /// Distinct temporal patterns cached in the pattern detector for this area.
    pub upstream_pattern_cache_size: usize,
    pub incoming_synapse_count: usize,
    pub outgoing_synapse_count: usize,
    pub total_memory_neuron_ids: usize,
    pub page: u32,
    pub page_size: u32,
    pub memory_neuron_ids: Vec<u64>,
    pub has_more: bool,
}

/// Per-peer cortical id (base64 string or null), genome cortical **name**, cortical index, and voxel `(x,y,z)` for the given neuron id.
pub(crate) fn peer_cortical_voxel_fields(
    mgr: &feagi_brain_development::ConnectomeManager,
    peer_id: u64,
    prefix: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut m = serde_json::Map::new();
    let peer_cortical = mgr.get_neuron_cortical_id(peer_id);
    let cortical_id = peer_cortical.map(|c| c.as_base_64());
    m.insert(
        format!("{prefix}_cortical_id"),
        cortical_id.map_or(serde_json::Value::Null, serde_json::Value::String),
    );
    let cortical_name = peer_cortical
        .and_then(|cid| mgr.get_cortical_area(&cid))
        .map(|a| a.name.clone());
    m.insert(
        format!("{prefix}_cortical_name"),
        cortical_name.map_or(serde_json::Value::Null, serde_json::Value::String),
    );
    m.insert(
        format!("{prefix}_cortical_idx"),
        mgr.get_neuron_cortical_idx_opt(peer_id)
            .map_or(serde_json::Value::Null, |v| serde_json::json!(v)),
    );
    let (vx, vy, vz) = mgr.get_neuron_coordinates(peer_id);
    m.insert(format!("{prefix}_x"), serde_json::json!(vx));
    m.insert(format!("{prefix}_y"), serde_json::json!(vy));
    m.insert(format!("{prefix}_z"), serde_json::json!(vz));
    m
}

/// Build JSON arrays for NPU synapse tuples, aligned with `/v1/connectome/{cortical_area_id}/synapses`,
/// plus peer cortical id, `target_cortical_name` / `source_cortical_name` (genome name), and voxel for the **target** (outgoing) or **source** (incoming) neuron.
pub(crate) fn synapse_details_for_neuron(
    mgr: &feagi_brain_development::ConnectomeManager,
    neuron_id: u32,
    outgoing: &[(u32, f32, f32, u8)],
    incoming: &[(u32, f32, f32, u8)],
) -> (serde_json::Value, serde_json::Value) {
    let outgoing_json: Vec<serde_json::Value> = outgoing
        .iter()
        .map(|&(target_id, weight, psp, synapse_type)| {
            let mut obj = serde_json::Map::new();
            obj.insert("source_neuron_id".to_string(), serde_json::json!(neuron_id));
            obj.insert("target_neuron_id".to_string(), serde_json::json!(target_id));
            obj.insert("weight".to_string(), serde_json::json!(weight));
            obj.insert("postsynaptic_potential".to_string(), serde_json::json!(psp));
            obj.insert("synapse_type".to_string(), serde_json::json!(synapse_type));
            obj.extend(peer_cortical_voxel_fields(mgr, target_id as u64, "target"));
            serde_json::Value::Object(obj)
        })
        .collect();
    let incoming_json: Vec<serde_json::Value> = incoming
        .iter()
        .map(|&(source_id, weight, psp, synapse_type)| {
            let mut obj = serde_json::Map::new();
            obj.insert("source_neuron_id".to_string(), serde_json::json!(source_id));
            obj.insert("target_neuron_id".to_string(), serde_json::json!(neuron_id));
            obj.insert("weight".to_string(), serde_json::json!(weight));
            obj.insert("postsynaptic_potential".to_string(), serde_json::json!(psp));
            obj.insert("synapse_type".to_string(), serde_json::json!(synapse_type));
            obj.extend(peer_cortical_voxel_fields(mgr, source_id as u64, "source"));
            serde_json::Value::Object(obj)
        })
        .collect();
    (
        serde_json::Value::Array(outgoing_json),
        serde_json::Value::Array(incoming_json),
    )
}

/// All neurons whose 3D coordinate within the cortical area matches the requested voxel, with live properties.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VoxelNeuronsResponse {
    pub cortical_id: String,
    /// Genome / connectome human-readable name for this cortical area (same as `cortical_name` elsewhere).
    pub cortical_name: String,
    pub cortical_idx: u32,
    /// Queried voxel within the cortical volume (same values as `x`, `y`, `z`).
    pub voxel_coordinate: [u32; 3],
    pub x: u32,
    pub y: u32,
    pub z: u32,
    /// Echo of the requested synapse detail page (0-based).
    pub synapse_page: u32,
    pub neuron_count: usize,
    pub neurons: Vec<serde_json::Value>,
}

/// Resolve every neuron at `(x, y, z)` inside `cortical_id` and attach `cortical_id` / `cortical_idx`,
/// plus paginated `outgoing_synapses` / `incoming_synapses` (at most 50 each per `synapse_page`),
/// full `outgoing_synapse_count` / `incoming_synapse_count`, and `*_synapses_has_more` flags.
async fn resolve_voxel_neurons(
    state: &ApiState,
    cortical_id: String,
    x: u32,
    y: u32,
    z: u32,
    synapse_page: u32,
) -> ApiResult<VoxelNeuronsResponse> {
    let connectome_service = state.connectome_service.as_ref();
    let area = connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .map_err(ApiError::from)?;

    let cortical_idx = area.cortical_idx;
    let cortical_name = area.name.clone();

    let matching_ids: Vec<u32> = {
        let manager = feagi_brain_development::ConnectomeManager::instance();
        let manager_lock = manager.read();
        let npu_arc = manager_lock
            .get_npu()
            .ok_or_else(|| ApiError::internal("NPU not initialized"))?;
        let npu_lock = npu_arc.lock().map_err(|_| {
            ApiError::internal("NPU mutex poisoned; restart FEAGI or wait for burst recovery")
        })?;

        let mut ids: Vec<u32> = npu_lock
            .get_neurons_in_cortical_area(cortical_idx)
            .into_iter()
            .filter(|&nid| {
                npu_lock
                    .get_neuron_coordinates(nid)
                    .map(|(nx, ny, nz)| nx == x && ny == y && nz == z)
                    .unwrap_or(false)
            })
            .collect();
        ids.sort_unstable();
        ids
    };

    let mut neurons: Vec<serde_json::Value> = Vec::with_capacity(matching_ids.len());
    for neuron_id in &matching_ids {
        let nid = *neuron_id;
        let mut props = connectome_service
            .get_neuron_properties(nid as u64)
            .await
            .map_err(ApiError::from)?;
        props.insert(
            "cortical_id".to_string(),
            serde_json::Value::String(cortical_id.clone()),
        );
        props.insert("cortical_idx".to_string(), serde_json::json!(cortical_idx));

        let (
            outgoing_synapse_count,
            incoming_synapse_count,
            out_json,
            in_json,
            outgoing_synapses_has_more,
            incoming_synapses_has_more,
        ) = {
            let manager = feagi_brain_development::ConnectomeManager::instance();
            let mgr = manager.read();
            let outgoing_full = mgr.get_outgoing_synapses(nid as u64);
            let incoming_full = mgr.get_incoming_synapses(nid as u64);
            let oc = outgoing_full.len();
            let ic = incoming_full.len();
            let (o_start, o_end, out_has_more) = synapse_page_window(oc, synapse_page);
            let (i_start, i_end, in_has_more) = synapse_page_window(ic, synapse_page);
            let out_slice = &outgoing_full[o_start..o_end];
            let in_slice = &incoming_full[i_start..i_end];
            let (out_json, in_json) = synapse_details_for_neuron(&mgr, nid, out_slice, in_slice);
            (oc, ic, out_json, in_json, out_has_more, in_has_more)
        };
        props.insert("synapse_page".to_string(), serde_json::json!(synapse_page));
        props.insert(
            "outgoing_synapse_count".to_string(),
            serde_json::json!(outgoing_synapse_count),
        );
        props.insert(
            "incoming_synapse_count".to_string(),
            serde_json::json!(incoming_synapse_count),
        );
        props.insert(
            "outgoing_synapses_has_more".to_string(),
            serde_json::json!(outgoing_synapses_has_more),
        );
        props.insert(
            "incoming_synapses_has_more".to_string(),
            serde_json::json!(incoming_synapses_has_more),
        );
        props.insert("outgoing_synapses".to_string(), out_json);
        props.insert("incoming_synapses".to_string(), in_json);

        neurons.push(serde_json::to_value(&props).map_err(|e| {
            ApiError::internal(format!("Failed to serialize neuron properties: {}", e))
        })?);
    }

    Ok(VoxelNeuronsResponse {
        cortical_id,
        cortical_name,
        cortical_idx,
        voxel_coordinate: [x, y, z],
        x,
        y,
        z,
        synapse_page,
        neuron_count: neurons.len(),
        neurons,
    })
}

// ============================================================================
// ENDPOINTS
// ============================================================================

/// List all IPU (Input Processing Unit) cortical area IDs. Returns IDs of all sensory cortical areas.
#[utoipa::path(get, path = "/v1/cortical_area/ipu", tag = "cortical_area")]
pub async fn get_ipu(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let ipu_areas: Vec<String> = areas
                .into_iter()
                .filter(|a| a.area_type == "sensory" || a.area_type == "IPU")
                .map(|a| a.cortical_id)
                .collect();
            Ok(Json(ipu_areas))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get IPU areas: {}",
            e
        ))),
    }
}

/// List all OPU (Output Processing Unit) cortical area IDs. Returns IDs of all motor cortical areas.
#[utoipa::path(get, path = "/v1/cortical_area/opu", tag = "cortical_area")]
pub async fn get_opu(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let opu_areas: Vec<String> = areas
                .into_iter()
                .filter(|a| a.area_type == "motor" || a.area_type == "OPU")
                .map(|a| a.cortical_id)
                .collect();
            Ok(Json(opu_areas))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get OPU areas: {}",
            e
        ))),
    }
}

/// Get a list of all cortical area IDs across the entire genome (IPU, OPU, custom, memory, and core areas).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area_id_list",
    tag = "cortical_area",
    responses(
        (status = 200, description = "Cortical area IDs retrieved successfully", body = CorticalAreaIdListResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_cortical_area_id_list(
    State(state): State<ApiState>,
) -> ApiResult<Json<CorticalAreaIdListResponse>> {
    tracing::debug!(target: "feagi-api", "🔍 GET /v1/cortical_area/cortical_area_id_list - handler called");
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.get_cortical_area_ids().await {
        Ok(ids) => {
            tracing::info!(target: "feagi-api", "✅ GET /v1/cortical_area/cortical_area_id_list - success, returning {} IDs", ids.len());
            tracing::debug!(target: "feagi-api", "📋 Cortical area IDs: {:?}", ids.iter().take(20).collect::<Vec<_>>());
            let response = CorticalAreaIdListResponse {
                cortical_ids: ids.clone(),
            };
            match serde_json::to_string(&response) {
                Ok(json_str) => {
                    tracing::debug!(target: "feagi-api", "📤 Response JSON: {}", json_str);
                }
                Err(e) => {
                    tracing::warn!(target: "feagi-api", "⚠️ Failed to serialize response: {}", e);
                }
            }
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!(target: "feagi-api", "❌ GET /v1/cortical_area/cortical_area_id_list - error: {}", e);
            Err(ApiError::internal(format!(
                "Failed to get cortical IDs: {}",
                e
            )))
        }
    }
}

/// Get a list of all cortical area names (human-readable labels for all cortical areas).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area_name_list",
    tag = "cortical_area",
    responses(
        (status = 200, description = "Cortical area names retrieved successfully", body = CorticalAreaNameListResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_cortical_area_name_list(
    State(state): State<ApiState>,
) -> ApiResult<Json<CorticalAreaNameListResponse>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let names: Vec<String> = areas.into_iter().map(|a| a.name).collect();
            Ok(Json(CorticalAreaNameListResponse {
                cortical_area_name_list: names,
            }))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get cortical names: {}",
            e
        ))),
    }
}

/// Get a map of cortical area IDs to their human-readable names. Returns {cortical_id: name} pairs.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_id_name_mapping",
    tag = "cortical_area"
)]
pub async fn get_cortical_id_name_mapping(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    let ids = connectome_service
        .get_cortical_area_ids()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get IDs: {}", e)))?;

    let mut mapping = HashMap::new();
    for id in ids {
        if let Ok(area) = connectome_service.get_cortical_area(&id).await {
            mapping.insert(id, area.name);
        }
    }
    Ok(Json(mapping))
}

/// Get available cortical area types: sensory, motor, memory, and custom.
#[utoipa::path(get, path = "/v1/cortical_area/cortical_types", tag = "cortical_area")]
pub async fn get_cortical_types(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![
        "sensory".to_string(),
        "motor".to_string(),
        "memory".to_string(),
        "custom".to_string(),
    ]))
}

/// Get detailed cortical connectivity mappings showing source-to-destination connections with mapping rules.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_map_detailed",
    tag = "cortical_area",
    responses(
        (status = 200, description = "Detailed cortical area mapping data", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_cortical_map_detailed(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let mut map: HashMap<String, serde_json::Value> = HashMap::new();

            for area in areas {
                // Extract cortical_mapping_dst from area properties
                if let Some(cortical_mapping_dst) = area.properties.get("cortical_mapping_dst") {
                    if !cortical_mapping_dst.is_null()
                        && cortical_mapping_dst
                            .as_object()
                            .is_some_and(|obj| !obj.is_empty())
                    {
                        map.insert(area.cortical_id.clone(), cortical_mapping_dst.clone());
                    }
                }
            }

            Ok(Json(map))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get detailed map: {}",
            e
        ))),
    }
}

/// Get 2D positions of all cortical areas for visualization. Returns {cortical_id: (x, y)} coordinates.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_locations_2d",
    tag = "cortical_area"
)]
pub async fn get_cortical_locations_2d(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, (i32, i32)>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let locations: HashMap<String, (i32, i32)> = areas
                .into_iter()
                .map(|area| (area.cortical_id, (area.position.0, area.position.1)))
                .collect();
            Ok(Json(locations))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get 2D locations: {}",
            e
        ))),
    }
}

/// Get complete cortical area data including geometry, neural parameters, and metadata. Used by Brain Visualizer.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area/geometry",
    tag = "cortical_area"
)]
pub async fn get_cortical_area_geometry(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let geometry: HashMap<String, serde_json::Value> = areas.into_iter()
                .map(|area| {
                    // Return FULL cortical area data (matching Python format)
                    // This is what Brain Visualizer expects for genome loading
                    let coordinate_2d = area
                        .properties
                        .get("coordinate_2d")
                        .or_else(|| area.properties.get("coordinates_2d"))
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([0, 0]));
                    let data = serde_json::json!({
                        "cortical_id": area.cortical_id,
                        "cortical_name": area.name,
                        "cortical_group": area.cortical_group,
                        "cortical_type": area.cortical_type,  // NEW: Explicitly include cortical_type for BV
                        "cortical_sub_group": area.sub_group.as_ref().unwrap_or(&String::new()),  // Return empty string instead of null
                        "coordinates_3d": [area.position.0, area.position.1, area.position.2],
                        "coordinates_2d": coordinate_2d,
                        "cortical_dimensions": [area.dimensions.0, area.dimensions.1, area.dimensions.2],
                        "cortical_neuron_per_vox_count": area.neurons_per_voxel,
                        "visualization": area.visible,
                        "visible": area.visible,
                        // Also include dictionary-style for backward compatibility
                        "dimensions": {
                            "x": area.dimensions.0,
                            "y": area.dimensions.1,
                            "z": area.dimensions.2
                        },
                        "position": {
                            "x": area.position.0,
                            "y": area.position.1,
                            "z": area.position.2
                        },
                        // Neural parameters
                        "neuron_post_synaptic_potential": area.postsynaptic_current,
                        // BV expects firing threshold and threshold limit as separate fields.
                        "neuron_fire_threshold": area.firing_threshold,
                        "neuron_firing_threshold_limit": area.firing_threshold_limit,
                        "plasticity_constant": area.plasticity_constant,
                        "degeneration": area.degeneration,
                        "leak_coefficient": area.leak_coefficient,
                        "refractory_period": area.refractory_period,
                        "snooze_period": area.snooze_period,
                        // Parent region ID (required by Brain Visualizer)
                        "parent_region_id": area.parent_region_id,
                        // Visualization voxel granularity for large-area rendering (optional)
                        "visualization_voxel_granularity": area.visualization_voxel_granularity.map(|(x, y, z)| serde_json::json!([x, y, z])),
                        // Sensor cadence hold (input areas only; effective value incl. template default)
                        "sensory_hold_last_frame": area.sensory_hold_last_frame,
                        "scan_twin": geometry_scan_twin_flag(&area.properties),
                        "classifier_role": geometry_classifier_role(&area.properties),
                        "temporal_depth": area.temporal_depth,
                    });
                    (area.cortical_id.clone(), data)
                })
                .collect();
            Ok(Json(geometry))
        }
        Err(e) => Err(ApiError::internal(format!("Failed to get geometry: {}", e))),
    }
}

/// Get visibility status of all cortical areas. Returns {cortical_id: visibility_flag}.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_visibility",
    tag = "cortical_area"
)]
pub async fn get_cortical_visibility(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, bool>>> {
    let connectome_service = state.connectome_service.as_ref();
    match connectome_service.list_cortical_areas().await {
        Ok(areas) => {
            let visibility: HashMap<String, bool> = areas
                .into_iter()
                .map(|area| (area.cortical_id, area.visible))
                .collect();
            Ok(Json(visibility))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get visibility: {}",
            e
        ))),
    }
}

/// Get the 2D location of a cortical area by its name. Request: {cortical_name: string}.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/cortical_name_location",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn post_cortical_name_location(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, (i32, i32)>>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_name = request
        .get("cortical_name")
        .ok_or_else(|| ApiError::invalid_input("cortical_name required"))?;

    match connectome_service.get_cortical_area(cortical_name).await {
        Ok(area) => Ok(Json(HashMap::from([(
            area.cortical_id,
            (area.position.0, area.position.1),
        )]))),
        Err(e) => Err(ApiError::internal(format!("Failed to get location: {}", e))),
    }
}

/// Get detailed properties of a single cortical area by ID. Request: {cortical_id: string}.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/cortical_area_properties",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn post_cortical_area_properties(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_id = request
        .get("cortical_id")
        .ok_or_else(|| ApiError::invalid_input("cortical_id required"))?;

    match connectome_service.get_cortical_area(cortical_id).await {
        Ok(area_info) => {
            tracing::debug!(target: "feagi-api", "Cortical area properties for {}: cortical_group={}, area_type={}, cortical_type={}", 
                cortical_id, area_info.cortical_group, area_info.area_type, area_info.cortical_type);
            tracing::info!(target: "feagi-api", "[API-RESPONSE] Returning mp_driven_psp={} for area {}", area_info.mp_driven_psp, cortical_id);
            let json_value = serde_json::to_value(&area_info).unwrap_or_default();
            tracing::debug!(target: "feagi-api", "Serialized JSON keys: {:?}", json_value.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            tracing::debug!(target: "feagi-api", "Serialized cortical_type value: {:?}", json_value.get("cortical_type"));
            Ok(Json(json_value))
        }
        Err(e) => Err(ApiError::internal(format!(
            "Failed to get properties: {}",
            e
        ))),
    }
}

/// Get properties for multiple cortical areas. Accepts array [\"id1\", \"id2\"] or object {cortical_id_list: [...]}.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/multi/cortical_area_properties",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn post_multi_cortical_area_properties(
    State(state): State<ApiState>,
    Json(request): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let connectome_service = state.connectome_service.as_ref();
    let mut result = HashMap::new();

    // Support both formats for backward compatibility
    let cortical_ids: Vec<String> = if request.is_array() {
        // Format 1: Direct array ["id1", "id2"] (Python SDK)
        request
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    } else if request.is_object() {
        // Format 2: Object with cortical_id_list {"cortical_id_list": ["id1", "id2"]} (Brain Visualizer)
        request
            .get("cortical_id_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ApiError::invalid_input("cortical_id_list required in object format"))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    } else {
        return Err(ApiError::invalid_input(
            "Request must be an array of IDs or object with cortical_id_list",
        ));
    };

    for cortical_id in cortical_ids {
        if let Ok(area_info) = connectome_service.get_cortical_area(&cortical_id).await {
            tracing::trace!(target: "feagi-api",
                "[MULTI] Area {}: cortical_type={}, cortical_group={}, is_mem_type={:?}",
                cortical_id, area_info.cortical_type, area_info.cortical_group,
                area_info.properties.get("is_mem_type")
            );
            let json_value = serde_json::to_value(&area_info).unwrap_or_default();
            tracing::trace!(target: "feagi-api",
                "[MULTI] Serialized has cortical_type: {}",
                json_value.get("cortical_type").is_some()
            );
            result.insert(cortical_id, json_value);
        }
    }
    Ok(Json(result))
}

fn parse_dimension_triplet(value: &serde_json::Value) -> Option<(usize, usize, usize)> {
    let values = value.as_array()?;
    if values.len() != 3 {
        return None;
    }
    let width = values.first().and_then(serde_json::Value::as_u64)? as usize;
    let height = values.get(1).and_then(serde_json::Value::as_u64)? as usize;
    let depth = values.get(2).and_then(serde_json::Value::as_u64)? as usize;
    if width == 0 || height == 0 || depth == 0 {
        return None;
    }
    Some((width, height, depth))
}

/// Optional per-subunit volumes. Absent means every subunit uses the shared override or template.
fn parse_per_device_dimensions_by_subunit(
    request: &HashMap<String, serde_json::Value>,
) -> Result<HashMap<u8, (usize, usize, usize)>, ApiError> {
    let Some(raw) = request.get("per_device_dimensions_by_subunit") else {
        return Ok(HashMap::new());
    };
    let object = raw.as_object().ok_or_else(|| {
        ApiError::invalid_input("per_device_dimensions_by_subunit must be an object")
    })?;
    let mut parsed = HashMap::new();
    for (key, value) in object {
        let subunit_index = key.parse::<u8>().map_err(|_| {
            ApiError::invalid_input("per_device_dimensions_by_subunit keys must be subunit indexes")
        })?;
        let dimensions = parse_dimension_triplet(value).ok_or_else(|| {
            ApiError::invalid_input(format!(
                "per_device_dimensions_by_subunit[{key}] must be [width, height, depth]"
            ))
        })?;
        parsed.insert(subunit_index, dimensions);
    }
    Ok(parsed)
}

fn property_dev_count(properties: &Option<HashMap<String, serde_json::Value>>) -> Option<usize> {
    properties
        .as_ref()
        .and_then(|properties| properties.get("dev_count"))
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
}

fn property_per_device_dimensions(
    properties: &Option<HashMap<String, serde_json::Value>>,
) -> Option<(usize, usize, usize)> {
    properties
        .as_ref()
        .and_then(|properties| properties.get("cortical_dimensions_per_device"))
        .and_then(parse_dimension_triplet)
}

/// True when the connectome area already has the volume this request would write.
fn io_area_matches_requested_geometry(
    current_dimensions: (usize, usize, usize),
    current_dev_count: Option<usize>,
    current_per_device: Option<(usize, usize, usize)>,
    requested_dimensions: (usize, usize, usize),
    requested_dev_count: Option<usize>,
    requested_per_device: Option<(usize, usize, usize)>,
) -> bool {
    current_dimensions == requested_dimensions
        && current_dev_count == requested_dev_count
        && current_per_device == requested_per_device
}

fn dimension_update_changes(
    dimensions: (usize, usize, usize),
    per_device: Option<(usize, usize, usize)>,
    dev_count: Option<usize>,
) -> HashMap<String, serde_json::Value> {
    let mut changes = HashMap::new();
    changes.insert(
        "dimensions".to_string(),
        serde_json::json!([dimensions.0, dimensions.1, dimensions.2]),
    );
    if let Some(per_device) = per_device {
        changes.insert(
            "cortical_dimensions_per_device".to_string(),
            serde_json::json!([per_device.0, per_device.1, per_device.2]),
        );
    }
    if let Some(dev_count) = dev_count {
        changes.insert("dev_count".to_string(), serde_json::json!(dev_count));
    }
    changes
}

/// Create IPU (sensory) or OPU (motor) cortical areas with proper topology and multi-unit support.
#[utoipa::path(post, path = "/v1/cortical_area/cortical_area", tag = "cortical_area")]
#[allow(unused_variables)] // In development - parameters will be used when implemented
pub async fn post_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<serde_json::Value>> {
    use feagi_services::types::CreateCorticalAreaParams;
    use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};

    // ARCHITECTURE: Use genome_service (proper entry point) instead of connectome_service
    let genome_service = state.genome_service.as_ref();

    // Extract required fields
    let cortical_type_key = request
        .get("cortical_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("cortical_id required"))?;

    let mut group_id: u16 = request
        .get("group_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .try_into()
        .map_err(|_| ApiError::invalid_input("group_id out of range"))?;

    let device_count = request
        .get("device_count")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ApiError::invalid_input("device_count required"))?
        as usize;

    let coordinates_3d: Vec<i32> = request
        .get("coordinates_3d")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 3 {
                Some(vec![
                    arr[0].as_i64()? as i32,
                    arr[1].as_i64()? as i32,
                    arr[2].as_i64()? as i32,
                ])
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::invalid_input("coordinates_3d must be [x, y, z]"))?;

    let cortical_type_str = request
        .get("cortical_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("cortical_type required"))?;

    let unit_id: Option<u16> = request
        .get("unit_id")
        .and_then(|v| v.as_u64())
        .map(|value| {
            value
                .try_into()
                .map_err(|_| ApiError::invalid_input("unit_id out of range"))
        })
        .transpose()?;
    if let Some(unit_id) = unit_id {
        group_id = unit_id;
    }

    // Extract neurons_per_voxel from request (default to 1 if not provided)
    let neurons_per_voxel = request
        .get("neurons_per_voxel")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    // Optional: Override per-device dimensions (especially Z for angle resolution)
    // Format: [x, y, z] where x=joints per device, y=1, z=angle_resolution
    // Example: [1, 1, 32] for single-joint servo with 32-angle resolution
    let per_device_dimensions_override: Option<(usize, usize, usize)> = request
        .get("per_device_dimensions")
        .and_then(parse_dimension_triplet);
    let per_device_dimensions_by_subunit = parse_per_device_dimensions_by_subunit(&request)?;

    // BREAKING CHANGE (unreleased API):
    // `data_type_config` is now per-subunit, because some cortical units have heterogeneous
    // subunits (e.g. Gaze: Percentage2D + Percentage).
    //
    // Request must provide:
    //   data_type_configs_by_subunit: { "0": <u16>, "1": <u16>, ... }
    let raw_configs = request
        .get("data_type_configs_by_subunit")
        .and_then(|v| v.as_object())
        .ok_or_else(|| ApiError::invalid_input("data_type_configs_by_subunit (object) required"))?;

    let mut data_type_configs_by_subunit: HashMap<u8, u16> = HashMap::new();

    for (k, v) in raw_configs {
        let subunit_idx_u64 = k.parse::<u64>().map_err(|_| {
            ApiError::invalid_input("data_type_configs_by_subunit keys must be integers")
        })?;
        let subunit_idx: u8 = subunit_idx_u64.try_into().map_err(|_| {
            ApiError::invalid_input("data_type_configs_by_subunit key out of range")
        })?;

        let parsed_u64 = if let Some(u) = v.as_u64() {
            Some(u)
        } else if let Some(i) = v.as_i64() {
            if i >= 0 {
                Some(i as u64)
            } else {
                None
            }
        } else if let Some(f) = v.as_f64() {
            if f >= 0.0 {
                Some(f.round() as u64)
            } else {
                None
            }
        } else if let Some(s) = v.as_str() {
            s.parse::<u64>().ok()
        } else {
            None
        }
        .ok_or_else(|| {
            ApiError::invalid_input("data_type_configs_by_subunit values must be numeric")
        })?;

        if parsed_u64 > u16::MAX as u64 {
            return Err(ApiError::invalid_input(
                "data_type_configs_by_subunit value exceeds u16::MAX",
            ));
        }

        data_type_configs_by_subunit.insert(subunit_idx, parsed_u64 as u16);
    }

    tracing::info!(
        target: "feagi-api",
        "Creating cortical areas for {} with neurons_per_voxel={}, data_type_configs_by_subunit={:?}",
        cortical_type_key,
        neurons_per_voxel,
        data_type_configs_by_subunit
    );

    // Determine number of units and get topology. `resolved_sensory_unit` is carried into the
    // per-subunit loop below so IPU creation can apply the unit's template-defined neural
    // tunables (firing threshold / increment) instead of generic zeros.
    let (num_units, unit_topology, resolved_sensory_unit) = if cortical_type_str == "IPU" {
        // Find the matching sensory cortical unit
        let unit = *SensoryCorticalUnit::list_all()
            .iter()
            .find(|u| {
                let id_ref = u.get_cortical_id_unit_reference();
                let key = format!("i{}", std::str::from_utf8(&id_ref).unwrap_or(""));
                key == cortical_type_key
            })
            .ok_or_else(|| {
                ApiError::invalid_input(format!("Unknown IPU type: {}", cortical_type_key))
            })?;

        (
            unit.get_number_cortical_areas(),
            unit.get_unit_default_topology(),
            Some(unit),
        )
    } else if cortical_type_str == "OPU" {
        // Find the matching motor cortical unit
        let unit = MotorCorticalUnit::list_all()
            .iter()
            .find(|u| {
                let id_ref = u.get_cortical_id_unit_reference();
                let key = format!("o{}", std::str::from_utf8(&id_ref).unwrap_or(""));
                key == cortical_type_key
            })
            .ok_or_else(|| {
                ApiError::invalid_input(format!("Unknown OPU type: {}", cortical_type_key))
            })?;

        (
            unit.get_number_cortical_areas(),
            unit.get_unit_default_topology(),
            None,
        )
    } else {
        return Err(ApiError::invalid_input("cortical_type must be IPU or OPU"));
    };

    tracing::info!(
        "Creating {} units for cortical type: {}",
        num_units,
        cortical_type_key
    );

    // Build creation parameters for all units
    let mut creation_params = Vec::new();
    for unit_idx in 0..num_units {
        let data_type_config = data_type_configs_by_subunit
            .get(&(unit_idx as u8))
            .copied()
            .ok_or_else(|| {
                ApiError::invalid_input(format!(
                    "data_type_configs_by_subunit missing entry for subunit {}",
                    unit_idx
                ))
            })?;

        let io_flag = IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(
            data_type_config,
        )
        .map_err(|e| {
            ApiError::invalid_input(format!(
                "Invalid data_type_config for subunit {}: {}",
                unit_idx, e
            ))
        })?;

        // Get per-device dimensions from topology, then scale X by device_count:
        // total_x = device_count * per_device_x
        // If per_device_dimensions_override is provided, use it instead of topology defaults
        let (per_device_dimensions, dimensions) = if let Some(override_dims) =
            per_device_dimensions_by_subunit
                .get(&(unit_idx as u8))
                .copied()
                .or(per_device_dimensions_override)
        {
            // Use custom per-device dimensions, scale X by device_count
            let total_x = override_dims.0.saturating_mul(device_count);
            (override_dims, (total_x, override_dims.1, override_dims.2))
        } else if let Some(topo) = unit_topology.get(&CorticalSubUnitIndex::from(unit_idx as u8)) {
            // Use topology defaults, scale X by device_count
            let dims = topo.channel_dimensions_default;
            let per_device = (dims[0] as usize, dims[1] as usize, dims[2] as usize);
            let total_x = per_device.0.saturating_mul(device_count);
            (per_device, (total_x, per_device.1, per_device.2))
        } else {
            ((1, 1, 1), (device_count.max(1), 1, 1)) // Fallback
        };

        // Calculate position for this unit
        let position =
            if let Some(topo) = unit_topology.get(&CorticalSubUnitIndex::from(unit_idx as u8)) {
                let rel_pos = topo.relative_position;
                (
                    coordinates_3d[0] + rel_pos[0],
                    coordinates_3d[1] + rel_pos[1],
                    coordinates_3d[2] + rel_pos[2],
                )
            } else {
                (coordinates_3d[0], coordinates_3d[1], coordinates_3d[2])
            };

        // Construct proper 8-byte cortical ID
        // Byte structure: [type(i/o), subtype[0], subtype[1], subtype[2], encoding_type, encoding_format, unit_idx, group_id]
        // Extract the 3-character subtype from cortical_type_key (e.g., "isvi" -> "svi")
        let subtype_bytes = if cortical_type_key.len() >= 4 {
            let subtype_str = &cortical_type_key[1..4]; // Skip the 'i' or 'o' prefix
            let mut bytes = [0u8; 3];
            for (i, c) in subtype_str.chars().take(3).enumerate() {
                bytes[i] = c as u8;
            }
            bytes
        } else {
            return Err(ApiError::invalid_input("Invalid cortical_type_key"));
        };

        // Construct the 8-byte cortical ID: flags include subunit in bits 4-7,
        // unit index is little-endian u16 in bytes 6-7.
        let cortical_id_obj = io_flag.as_io_cortical_id(
            cortical_type_str == "IPU",
            subtype_bytes,
            CorticalUnitIndex::from(group_id),
            CorticalSubUnitIndex::from(unit_idx as u8),
        );
        let cortical_id = cortical_id_obj.as_base_64();

        tracing::debug!(target: "feagi-api",
            "  Unit {}: dims={}x{}x{}, neurons_per_voxel={}, total_neurons={}",
            unit_idx, dimensions.0, dimensions.1, dimensions.2, neurons_per_voxel,
            dimensions.0 * dimensions.1 * dimensions.2 * neurons_per_voxel as usize
        );

        // Store device_count and per-device dimensions in properties for BV compatibility
        let mut properties = HashMap::new();
        properties.insert(
            "dev_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(device_count)),
        );
        properties.insert(
            "cortical_dimensions_per_device".to_string(),
            serde_json::json!([
                per_device_dimensions.0,
                per_device_dimensions.1,
                per_device_dimensions.2
            ]),
        );

        // Apply per-unit neural tunables. `CreateCorticalAreaParams.properties` fully replaces
        // `area.properties` for non-Memory area types (see GenomeServiceImpl::create_cortical_areas),
        // so these must be set here rather than via the (unused-for-IPU/OPU) typed
        // firing_threshold_increment/limit fields below.
        if let Some(sensory_unit) = resolved_sensory_unit {
            if let Some(default_firing_threshold) = sensory_unit.get_default_firing_threshold() {
                properties.insert(
                    "firing_threshold".to_string(),
                    serde_json::json!(default_firing_threshold),
                );
            }
            if let Some(default_increment) = sensory_unit.get_default_firing_threshold_increment() {
                properties.insert(
                    "firing_threshold_increment_x".to_string(),
                    serde_json::json!(default_increment[0]),
                );
                properties.insert(
                    "firing_threshold_increment_y".to_string(),
                    serde_json::json!(default_increment[1]),
                );
                properties.insert(
                    "firing_threshold_increment_z".to_string(),
                    serde_json::json!(default_increment[2]),
                );
            }
            if let Some(default_mp_charge_accumulation) =
                sensory_unit.get_default_mp_charge_accumulation()
            {
                properties.insert(
                    "mp_charge_accumulation".to_string(),
                    serde_json::json!(default_mp_charge_accumulation),
                );
            }
            // Sensor cadence hold: scalar sensors hold their last frame for one burst.
            properties.insert(
                feagi_structures::genomic::SENSORY_HOLD_LAST_FRAME_PROPERTY.to_string(),
                serde_json::json!(sensory_unit.get_default_sensory_hold_last_frame()),
            );
        }

        let area_name = request
            .get("cortical_name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string())
            .unwrap_or_else(|| {
                feagi_services::impls::connectome_service_impl::derive_friendly_cortical_name(
                    &cortical_id_obj,
                )
                .unwrap_or_else(|| format!("{} Unit {}", cortical_type_key, group_id))
            });

        properties.remove("modulator_instance_id");
        let params = CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: area_name,
            dimensions,
            position,
            area_type: cortical_type_str.to_string(),
            visible: Some(true),
            sub_group: None,
            neurons_per_voxel: Some(neurons_per_voxel),
            postsynaptic_current: Some(0.0),
            plasticity_constant: Some(0.0),
            degeneration: Some(0.0),
            psp_uniform_distribution: Some(false),
            firing_threshold_increment: Some(0.0),
            firing_threshold_limit: Some(0.0),
            consecutive_fire_count: Some(0),
            snooze_period: Some(0),
            refractory_period: Some(0),
            leak_coefficient: Some(0.0),
            leak_variability: Some(0.0),
            burst_engine_active: Some(true),
            properties: Some(properties),
        };

        creation_params.push(params);
    }

    // Same group_id addresses the same cortical IDs. An existing subunit is resized to the
    // requested volume. Only subunits that are absent are created, so a partial segmented
    // vision group is completed at the agent's registered size.
    let connectome_service = state.connectome_service.as_ref();
    let mut to_create = Vec::new();
    let mut resized_ids: Vec<String> = Vec::new();
    for params in creation_params {
        let exists = connectome_service
            .cortical_area_exists(&params.cortical_id)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to check cortical area {}: {}",
                    params.cortical_id, e
                ))
            })?;
        if !exists {
            to_create.push(params);
            continue;
        }
        let current = connectome_service
            .get_cortical_area(&params.cortical_id)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to read cortical area {}: {}",
                    params.cortical_id, e
                ))
            })?;
        let requested_per_device = property_per_device_dimensions(&params.properties);
        let requested_dev_count = property_dev_count(&params.properties);
        if io_area_matches_requested_geometry(
            current.dimensions,
            current.dev_count,
            current.cortical_dimensions_per_device,
            params.dimensions,
            requested_dev_count,
            requested_per_device,
        ) {
            continue;
        }
        let changes =
            dimension_update_changes(params.dimensions, requested_per_device, requested_dev_count);
        genome_service
            .update_cortical_area(&params.cortical_id, changes)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to update cortical area {} to the registered size: {}",
                    params.cortical_id, e
                ))
            })?;
        tracing::info!(
            "Updated existing cortical area {} to {:?} for group {}",
            params.cortical_id,
            params.dimensions,
            group_id
        );
        resized_ids.push(params.cortical_id);
    }

    tracing::info!(
        "Calling GenomeService to create {} cortical areas (resized {})",
        to_create.len(),
        resized_ids.len()
    );

    // ARCHITECTURE: Call genome_service.create_cortical_areas (proper flow)
    // This will: 1) Update runtime genome, 2) Call neuroembryogenesis, 3) Create neurons/synapses
    let areas_details = if to_create.is_empty() {
        Vec::new()
    } else {
        genome_service
            .create_cortical_areas(to_create)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to create cortical areas: {}", e)))?
    };

    tracing::info!(
        "✅ Successfully created {} cortical areas via GenomeService",
        areas_details.len()
    );

    // Serialize as JSON
    let areas_json = serde_json::to_value(&areas_details).unwrap_or_default();

    // Extract cortical IDs from created areas. Resized IDs stay in the response so a partial
    // group reports every subunit this request aligned.
    let mut created_ids: Vec<String> = areas_details
        .iter()
        .map(|a| a.cortical_id.clone())
        .collect();
    created_ids.extend(resized_ids.iter().cloned());

    // Return comprehensive response
    let first_id = created_ids.first().cloned().unwrap_or_default();
    let message = if resized_ids.is_empty() {
        format!("Created {} cortical areas", areas_details.len())
    } else {
        format!(
            "Created {} cortical areas, resized {}",
            areas_details.len(),
            resized_ids.len()
        )
    };
    let mut response = serde_json::Map::new();
    response.insert("message".to_string(), serde_json::Value::String(message));
    response.insert(
        "cortical_id".to_string(),
        serde_json::Value::String(first_id),
    ); // For backward compatibility
    response.insert(
        "cortical_ids".to_string(),
        serde_json::Value::String(created_ids.join(", ")),
    );
    response.insert(
        "unit_count".to_string(),
        serde_json::Value::Number(created_ids.len().into()),
    );
    response.insert("areas".to_string(), areas_json); // Full details for all areas

    Ok(Json(serde_json::Value::Object(response)))
}

/// Update properties of an existing cortical area (position, dimensions, neural parameters, etc.).
#[utoipa::path(put, path = "/v1/cortical_area/cortical_area", tag = "cortical_area")]
pub async fn put_cortical_area(
    State(state): State<ApiState>,
    Json(mut request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let genome_service = state.genome_service.as_ref();

    // Extract cortical_id
    let cortical_id = request
        .get("cortical_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("cortical_id required"))?
        .to_string();

    tracing::debug!(
        target: "feagi-api",
        "PUT /v1/cortical_area/cortical_area - received update for area: {} (keys: {:?})",
        cortical_id,
        request.keys().collect::<Vec<_>>()
    );

    // Remove cortical_id from changes (it's not a property to update)
    request.remove("cortical_id");

    // Call GenomeService with raw changes (it handles classification and routing)
    match genome_service
        .update_cortical_area(&cortical_id, request)
        .await
    {
        Ok(area_info) => {
            let updated_id = area_info.cortical_id.clone();
            tracing::debug!(
                target: "feagi-api",
                "PUT /v1/cortical_area/cortical_area - success for {} (updated_id={})",
                cortical_id,
                updated_id
            );
            Ok(Json(HashMap::from([
                ("message".to_string(), "Cortical area updated".to_string()),
                ("cortical_id".to_string(), updated_id),
                ("previous_cortical_id".to_string(), cortical_id),
            ])))
        }
        Err(e) => {
            tracing::error!(target: "feagi-api", "PUT /v1/cortical_area/cortical_area - failed for {}: {}", cortical_id, e);
            Err(ApiError::internal(format!("Failed to update: {}", e)))
        }
    }
}

/// Delete a cortical area by ID. Removes the area and all associated neurons and synapses.
#[utoipa::path(
    delete,
    path = "/v1/cortical_area/cortical_area",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development - parameters will be used when implemented
pub async fn delete_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let connectome_service = state.connectome_service.as_ref();
    let cortical_id = request
        .get("cortical_id")
        .ok_or_else(|| ApiError::invalid_input("cortical_id required"))?;

    match connectome_service.delete_cortical_area(cortical_id).await {
        Ok(_) => Ok(Json(HashMap::from([(
            "message".to_string(),
            "Cortical area deleted".to_string(),
        )]))),
        Err(e) => Err(ApiError::internal(format!("Failed to delete: {}", e))),
    }
}

/// Create a custom cortical area for internal processing with specified dimensions and position.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/custom_cortical_area",
    tag = "cortical_area"
)]
pub async fn post_custom_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    use feagi_services::types::CreateCorticalAreaParams;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Helper: check whether BV is requesting a MEMORY cortical area (still routed through this endpoint).
    //
    // Brain Visualizer sends:
    //   sub_group_id: "MEMORY"
    //   cortical_group: "CUSTOM"
    //
    // In feagi-core, the authoritative cortical type is derived from the CorticalID prefix byte:
    // - b'c' => Custom
    // - b'm' => Memory
    //
    // So if sub_group_id indicates MEMORY, we must generate an 'm' prefixed CorticalID.
    let is_memory_area_requested = request
        .get("sub_group_id")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("MEMORY"))
        .unwrap_or(false);

    // Extract required fields from request
    let cortical_name = request
        .get("cortical_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("cortical_name required"))?;

    let cortical_dimensions: Vec<u32> = request
        .get("cortical_dimensions")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 3 {
                Some(vec![
                    arr[0].as_u64()? as u32,
                    arr[1].as_u64()? as u32,
                    arr[2].as_u64()? as u32,
                ])
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::invalid_input("cortical_dimensions must be [x, y, z]"))?;

    let coordinates_3d: Vec<i32> = request
        .get("coordinates_3d")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 3 {
                Some(vec![
                    arr[0].as_i64()? as i32,
                    arr[1].as_i64()? as i32,
                    arr[2].as_i64()? as i32,
                ])
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::invalid_input("coordinates_3d must be [x, y, z]"))?;

    // Custom and memory areas must belong to a brain region (circuit / sub-region), not be
    // created without regional membership (root is reserved for core, IPU, and OPU).
    let brain_region_id = request
        .get("brain_region_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            ApiError::invalid_input(
                "brain_region_id is required for custom and memory cortical areas",
            )
        })?;

    let cortical_sub_group = request
        .get("cortical_sub_group")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    tracing::info!(target: "feagi-api",
        "Creating {} cortical area '{}' with dimensions: {}x{}x{}, position: ({}, {}, {})",
        if is_memory_area_requested { "memory" } else { "custom" },
        cortical_name, cortical_dimensions[0], cortical_dimensions[1], cortical_dimensions[2],
        coordinates_3d[0], coordinates_3d[1], coordinates_3d[2]
    );

    // Generate unique cortical ID for custom cortical area
    // Format: [b'c', 6 random alphanumeric bytes, group_counter]
    // Use timestamp + counter to ensure uniqueness
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Create 8-byte cortical ID for custom/memory area
    // Byte 0: 'c' for custom OR 'm' for memory (authoritative type discriminator)
    // Bytes 1-6: Derived from name (first 6 chars, padded with underscores)
    // Byte 7: Counter based on timestamp lower bits
    let mut cortical_id_bytes = [0u8; 8];
    cortical_id_bytes[0] = if is_memory_area_requested { b'm' } else { b'c' };

    // Use the cortical name for bytes 1-6 (truncate or pad as needed)
    let name_bytes = cortical_name.as_bytes();
    for i in 1..7 {
        cortical_id_bytes[i] = if i - 1 < name_bytes.len() {
            // Use alphanumeric ASCII only
            let c = name_bytes[i - 1];
            if c.is_ascii_alphanumeric() || c == b'_' {
                c
            } else {
                b'_'
            }
        } else {
            b'_' // Padding
        };
    }

    // Byte 7: Use timestamp lower byte for uniqueness
    cortical_id_bytes[7] = (timestamp & 0xFF) as u8;

    // Encode to base64 for use as cortical_id string
    let cortical_id = general_purpose::STANDARD.encode(cortical_id_bytes);

    tracing::debug!(target: "feagi-api",
        "Generated cortical_id: {} (raw bytes: {:?})",
        cortical_id, cortical_id_bytes
    );

    // parent_region_id is required (validated above) so the area is registered in the hierarchy.
    let mut properties = HashMap::new();
    properties.insert(
        "parent_region_id".to_string(),
        serde_json::Value::String(brain_region_id.clone()),
    );

    // Create cortical area parameters
    let params = CreateCorticalAreaParams {
        cortical_id: cortical_id.clone(),
        name: cortical_name.to_string(),
        dimensions: (
            cortical_dimensions[0] as usize,
            cortical_dimensions[1] as usize,
            cortical_dimensions[2] as usize,
        ),
        position: (coordinates_3d[0], coordinates_3d[1], coordinates_3d[2]),
        area_type: if is_memory_area_requested {
            "Memory".to_string()
        } else {
            "Custom".to_string()
        },
        visible: Some(true),
        sub_group: cortical_sub_group,
        neurons_per_voxel: Some(1),
        postsynaptic_current: None,
        plasticity_constant: Some(0.0),
        degeneration: Some(0.0),
        psp_uniform_distribution: Some(false),
        firing_threshold_increment: Some(0.0),
        firing_threshold_limit: Some(0.0),
        consecutive_fire_count: Some(0),
        snooze_period: Some(0),
        refractory_period: Some(0),
        leak_coefficient: Some(0.0),
        leak_variability: Some(0.0),
        burst_engine_active: Some(true),
        properties: Some(properties),
    };

    let genome_service = state.genome_service.as_ref();

    tracing::info!(target: "feagi-api", "Calling GenomeService to create custom cortical area");

    // Create the cortical area via GenomeService
    let areas_details = genome_service
        .create_cortical_areas(vec![params])
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create custom cortical area: {}", e)))?;

    let created_area = areas_details
        .first()
        .ok_or_else(|| ApiError::internal("No cortical area was created"))?;

    tracing::info!(target: "feagi-api",
        "✅ Successfully created custom cortical area '{}' with ID: {}",
        cortical_name, created_area.cortical_id
    );

    // Return response
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        "Custom cortical area created successfully".to_string(),
    );
    response.insert("cortical_id".to_string(), created_area.cortical_id.clone());
    response.insert("cortical_name".to_string(), cortical_name.to_string());

    Ok(Json(response))
}

fn classifier_memory_params(
    cortical_id: String,
    name: String,
    position: (i32, i32, i32),
    parent_region_id: &str,
    extra_properties: HashMap<String, serde_json::Value>,
) -> feagi_services::types::CreateCorticalAreaParams {
    let mut properties = extra_properties;
    properties.insert(
        "parent_region_id".to_string(),
        serde_json::Value::String(parent_region_id.to_string()),
    );
    properties.insert("classifier_assembly".to_string(), serde_json::json!(true));
    feagi_services::types::CreateCorticalAreaParams {
        cortical_id,
        name,
        dimensions: (1, 1, 1),
        position,
        area_type: "Memory".to_string(),
        visible: Some(false),
        sub_group: Some("MEMORY".to_string()),
        neurons_per_voxel: Some(1),
        postsynaptic_current: None,
        plasticity_constant: Some(0.0),
        degeneration: Some(0.0),
        psp_uniform_distribution: Some(false),
        firing_threshold_increment: Some(0.0),
        firing_threshold_limit: Some(0.0),
        consecutive_fire_count: Some(0),
        snooze_period: Some(0),
        refractory_period: Some(0),
        leak_coefficient: Some(0.0),
        leak_variability: Some(0.0),
        burst_engine_active: Some(true),
        properties: Some(properties),
    }
}

#[cfg(test)]
fn classifier_enable_burst_changes() -> HashMap<String, serde_json::Value> {
    let mut changes = HashMap::new();
    changes.insert("burst_engine_active".to_string(), serde_json::json!(true));
    changes
}

/// Place the class-map twin above the classifier. Y clearance is the twin's own
/// height so the volume does not overlap the stamp at the classifier origin.
fn classifier_twin_position(
    classifier_position: (i32, i32, i32),
    twin_dimensions: (usize, usize, usize),
    x_offset: i32,
) -> (i32, i32, i32) {
    let clearance = twin_dimensions.1.max(1) as i32;
    (
        classifier_position.0 + x_offset,
        classifier_position.1 + clearance,
        classifier_position.2,
    )
}

/// Class output copies the class input's dimensions.
#[cfg(test)]
fn classifier_output_dimensions(
    class_dimensions: (usize, usize, usize),
) -> ApiResult<(usize, usize, usize)> {
    if class_dimensions.0 == 0 || class_dimensions.1 == 0 || class_dimensions.2 == 0 {
        return Err(ApiError::invalid_input(
            "class area dimensions must be greater than zero",
        ));
    }
    Ok(class_dimensions)
}

#[allow(dead_code)]
fn classifier_output_changes(
    classifier: &feagi_structures::genomic::classifiers::Classifier,
    class_area_name: &str,
    twin_dimensions: (usize, usize, usize),
    x_offset: i32,
) -> HashMap<String, serde_json::Value> {
    let mut changes = HashMap::new();
    changes.insert(
        "cortical_name".to_string(),
        serde_json::json!(
            feagi_structures::genomic::classifiers::class_output_area_name(
                &classifier.name,
                class_area_name,
            )
        ),
    );
    changes.insert(
        "coordinates_3d".to_string(),
        serde_json::json!(classifier_twin_position(
            (
                classifier.coordinates_3d[0],
                classifier.coordinates_3d[1],
                classifier.coordinates_3d[2],
            ),
            twin_dimensions,
            x_offset,
        )),
    );
    changes.insert(
        "cortical_dimensions".to_string(),
        serde_json::json!([twin_dimensions.0, twin_dimensions.1, twin_dimensions.2]),
    );
    changes.insert(
        "parent_region_id".to_string(),
        serde_json::json!(classifier.parent_region_id),
    );
    changes.insert("burst_engine_active".to_string(), serde_json::json!(true));
    changes.insert("mp_driven_psp".to_string(), serde_json::json!(false));
    changes.insert("leak_coefficient".to_string(), serde_json::json!(1.0));
    changes
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
fn classifier_scan_twin_params(
    cortical_id: String,
    name: String,
    dimensions: (usize, usize, usize),
    position: (i32, i32, i32),
    parent_region_id: &str,
    field_area_id: &str,
    class_memory_id: &str,
) -> feagi_services::types::CreateCorticalAreaParams {
    let mut properties = HashMap::new();
    properties.insert(
        "parent_region_id".to_string(),
        serde_json::Value::String(parent_region_id.to_string()),
    );
    properties.insert("classifier_assembly".to_string(), serde_json::json!(true));
    properties.insert(
        "classifier_role".to_string(),
        serde_json::json!("class_output"),
    );
    properties.insert("burst_engine_active".to_string(), serde_json::json!(true));
    properties.insert("is_mem_type".to_string(), serde_json::json!(false));
    properties.insert("mp_driven_psp".to_string(), serde_json::json!(false));
    properties.insert(
        "memory_twin_of".to_string(),
        serde_json::json!(field_area_id),
    );
    properties.insert(
        "memory_twin_for".to_string(),
        serde_json::json!(class_memory_id),
    );
    feagi_services::types::CreateCorticalAreaParams {
        cortical_id,
        name,
        dimensions,
        position,
        area_type: "Custom".to_string(),
        visible: Some(true),
        sub_group: None,
        neurons_per_voxel: Some(1),
        postsynaptic_current: None,
        plasticity_constant: Some(0.0),
        degeneration: Some(0.0),
        psp_uniform_distribution: Some(false),
        firing_threshold_increment: Some(0.0),
        firing_threshold_limit: Some(0.0),
        consecutive_fire_count: Some(0),
        snooze_period: Some(0),
        refractory_period: Some(0),
        leak_coefficient: Some(1.0),
        leak_variability: Some(0.0),
        burst_engine_active: Some(true),
        properties: Some(properties),
    }
}

fn geometry_scan_twin_flag(properties: &HashMap<String, serde_json::Value>) -> bool {
    properties.get("scan_twin").and_then(|v| v.as_bool()) == Some(true)
}

fn geometry_classifier_role(properties: &HashMap<String, serde_json::Value>) -> serde_json::Value {
    match properties.get("classifier_role").and_then(|v| v.as_str()) {
        Some(role) if !role.is_empty() => serde_json::Value::String(role.to_string()),
        _ => serde_json::Value::Null,
    }
}

use feagi_structures::genomic::classifiers::classifier_mapping_rule;

#[allow(dead_code)]
fn mapping_rule_morphology_id(rule: &serde_json::Value) -> Option<&str> {
    rule.as_object()
        .and_then(|rule_object| rule_object.get("morphology_id"))
        .and_then(|value| value.as_str())
}

#[allow(dead_code)]
fn mapping_rules_for_destination(
    properties: &HashMap<String, serde_json::Value>,
    dst_area_id: &str,
) -> Vec<serde_json::Value> {
    properties
        .get("cortical_mapping_dst")
        .and_then(|value| value.as_object())
        .and_then(|mappings| mappings.get(dst_area_id))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Encode and scan share one edge when the kernel area is also the field.
///
/// `update_cortical_mapping` replaces the destination's rule list. Writing only
/// `episodic_scan` deletes `episodic_memory`, so plasticity registers zero
/// episodic upstream while Brain Visualizer still draws the remaining yellow edge.
fn classifier_edge_rules_with_morphology(
    _existing_rules: &[serde_json::Value],
    morphology_id: &str,
    associative_window: u32,
) -> Vec<serde_json::Value> {
    vec![classifier_mapping_rule(morphology_id, associative_window)]
}

#[allow(dead_code)]
fn classifier_edge_rules_without_morphology(
    existing_rules: &[serde_json::Value],
    morphology_id: &str,
) -> Vec<serde_json::Value> {
    existing_rules
        .iter()
        .filter(|rule| mapping_rule_morphology_id(rule) != Some(morphology_id))
        .cloned()
        .collect()
}

/// Associative rule whose Reward instances are applied per scanning instance.
fn classifier_reward_mapping_rule(
    associative_window: u32,
    existing: &serde_json::Value,
    reward_modulator_ids: &[String],
    eligibility_decay_bursts: u32,
) -> serde_json::Value {
    let mut rule = if existing.is_object() {
        existing.clone()
    } else {
        classifier_mapping_rule("associative_memory", associative_window)
    };
    if let Some(object) = rule.as_object_mut() {
        object.insert(
            "morphology_id".to_string(),
            serde_json::json!("associative_memory"),
        );
        object.insert("plasticity_flag".to_string(), serde_json::json!(true));
        object.insert("plasticity_mode".to_string(), serde_json::json!("rstdp"));
        object.remove("reward_source_area");
        object.remove("punishment_source_area");
        if reward_modulator_ids.is_empty() {
            object.remove("modulators");
        } else {
            object.insert(
                "modulators".to_string(),
                serde_json::json!(reward_modulator_ids),
            );
        }
        object.remove("classifier_instance_reward");
        object.insert(
            "eligibility_decay_bursts".to_string(),
            serde_json::json!(eligibility_decay_bursts),
        );
        object
            .entry("plasticity_constant".to_string())
            .or_insert(serde_json::json!(1));
        object
            .entry("ltp_multiplier".to_string())
            .or_insert(serde_json::json!(1));
        object
            .entry("ltd_multiplier".to_string())
            .or_insert(serde_json::json!(1));
        object
            .entry("plasticity_window".to_string())
            .or_insert(serde_json::json!(associative_window));
    }
    rule
}

fn classifier_plain_associative_rule(
    associative_window: u32,
    existing: &serde_json::Value,
) -> serde_json::Value {
    let mut rule = if existing.is_object() {
        existing.clone()
    } else {
        classifier_mapping_rule("associative_memory", associative_window)
    };
    if let Some(object) = rule.as_object_mut() {
        object.remove("plasticity_mode");
        object.remove("reward_source_area");
        object.remove("punishment_source_area");
        object.remove("modulators");
        object.remove("classifier_instance_reward");
        object.remove("eligibility_decay_bursts");
        object.insert("plasticity_flag".to_string(), serde_json::json!(true));
        object.insert(
            "morphology_id".to_string(),
            serde_json::json!("associative_memory"),
        );
    }
    rule
}

fn existing_associative_rule(
    properties: &HashMap<String, serde_json::Value>,
    class_memory_id: &str,
) -> serde_json::Value {
    let Some(mapping) = properties
        .get("cortical_mapping_dst")
        .and_then(|value| value.as_object())
    else {
        return serde_json::Value::Null;
    };
    let Some(rules) = mapping
        .get(class_memory_id)
        .and_then(|value| value.as_array())
    else {
        return serde_json::Value::Null;
    };
    rules
        .iter()
        .find(|rule| {
            rule.get("morphology_id").and_then(|value| value.as_str()) == Some("associative_memory")
        })
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}

#[allow(dead_code)]
fn area_dimensions_u32(dimensions: (usize, usize, usize), label: &str) -> ApiResult<[u32; 3]> {
    let convert = |axis: usize| -> ApiResult<u32> {
        u32::try_from(axis).map_err(|_| {
            ApiError::invalid_input(format!("{label} dimension does not fit a 32-bit size"))
        })
    };
    Ok([
        convert(dimensions.0)?,
        convert(dimensions.1)?,
        convert(dimensions.2)?,
    ])
}

/// Mint an 8-byte cortical id for a classifier-owned area.
///
/// `prefix` must be a legal cortical type byte. Sample and class memory use `m`.
/// The error area is a custom area and uses `c`.
fn classifier_owned_area_id(
    prefix: u8,
    name: &str,
    timestamp: u64,
    suffix: u8,
) -> Result<String, ApiError> {
    let mut bytes = [0u8; CorticalID::CORTICAL_ID_LENGTH];
    bytes[0] = prefix;
    let name_bytes = name.as_bytes();
    for i in 1..7 {
        bytes[i] = if i - 1 < name_bytes.len() {
            let c = name_bytes[i - 1];
            if c.is_ascii_alphanumeric() || c == b'_' {
                c
            } else {
                b'_'
            }
        } else {
            b'_'
        };
    }
    bytes[7] = ((timestamp.wrapping_add(u64::from(suffix))) & 0xFF) as u8;
    let id = CorticalID::try_from_bytes(&bytes)
        .map_err(|error| ApiError::invalid_input(format!("Invalid cortical ID: {error}")))?;
    Ok(id.as_base_64())
}

/// Create the classifier's own circuit region.
///
/// The requested parent may be root. Root accepts circuits. It does not accept
/// the memory and custom areas inside the classifier, so those areas are
/// parented to this new region.
async fn create_classifier_circuit(
    state: &ApiState,
    name: &str,
    parent_region_id: &str,
    coordinates_3d: [i32; 3],
) -> ApiResult<String> {
    use feagi_services::types::CreateBrainRegionParams;
    use feagi_structures::genomic::brain_regions::RegionID;

    let region_id = RegionID::new().to_string();
    let mut properties = HashMap::new();
    properties.insert(
        "coordinate_2d".to_string(),
        serde_json::json!([coordinates_3d[0], coordinates_3d[2]]),
    );
    properties.insert(
        "coordinate_3d".to_string(),
        serde_json::json!(coordinates_3d),
    );
    properties.insert(
        "parent_region_id".to_string(),
        serde_json::json!(parent_region_id),
    );
    properties.insert("classifier_circuit".to_string(), serde_json::json!(true));
    state
        .connectome_service
        .create_brain_region(CreateBrainRegionParams {
            region_id: region_id.clone(),
            name: name.to_string(),
            region_type: "Custom".to_string(),
            parent_id: Some(parent_region_id.to_string()),
            properties: Some(properties),
        })
        .await
        .map_err(|error| {
            ApiError::internal(format!("Failed to create classifier circuit: {error}"))
        })?;
    Ok(region_id)
}

/// Assemble a classifier as its own circuit under an existing brain region.
///
/// The parent may be root. Sample memory, class memory, and the error area
/// are created inside the new circuit. Sample and class inputs are added
/// later by connectivity rule.
#[utoipa::path(post, path = "/v1/cortical_area/classifier", tag = "cortical_area")]
pub async fn post_classifier(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let name = request
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("name required"))?;
    let brain_region_id = request
        .get("brain_region_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::invalid_input("brain_region_id required"))?;
    let coordinates_3d: Vec<i32> = request
        .get("coordinates_3d")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 3 {
                Some(vec![
                    arr[0].as_i64()? as i32,
                    arr[1].as_i64()? as i32,
                    arr[2].as_i64()? as i32,
                ])
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::invalid_input("coordinates_3d must be [x, y, z]"))?;
    let coordinates = [coordinates_3d[0], coordinates_3d[1], coordinates_3d[2]];
    let circuit_region_id =
        create_classifier_circuit(&state, name, brain_region_id, coordinates).await?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let sample_mem_id = classifier_owned_area_id(b'm', name, timestamp, 1)?;
    let class_mem_id = classifier_owned_area_id(b'm', name, timestamp, 2)?;
    let error_area_id = classifier_owned_area_id(b'c', name, timestamp, 3)?;
    let correct_area_id = classifier_owned_area_id(b'c', name, timestamp, 4)?;

    let mut sample_mem_props = HashMap::new();
    sample_mem_props.insert(
        "classifier_role".to_string(),
        serde_json::json!("sample_memory"),
    );
    sample_mem_props.insert(
        "classifier_class_memory_id".to_string(),
        serde_json::json!(class_mem_id),
    );
    sample_mem_props.insert("memory_twin_areas".to_string(), serde_json::json!({}));

    let mut class_mem_props = HashMap::new();
    class_mem_props.insert(
        "classifier_role".to_string(),
        serde_json::json!("class_memory"),
    );
    class_mem_props.insert(
        "classifier_sample_memory_id".to_string(),
        serde_json::json!(sample_mem_id),
    );
    class_mem_props.insert(
        "classifier_error_area_id".to_string(),
        serde_json::json!(error_area_id),
    );
    class_mem_props.insert("memory_twin_areas".to_string(), serde_json::json!({}));

    let mut sample_mem_params = classifier_memory_params(
        sample_mem_id.clone(),
        format!("{name}_sample_mem"),
        (coordinates_3d[0], coordinates_3d[1], coordinates_3d[2]),
        &circuit_region_id,
        sample_mem_props,
    );
    sample_mem_params.visible = Some(true);
    let class_mem_params = classifier_memory_params(
        class_mem_id.clone(),
        format!("{name}_class_mem"),
        (coordinates_3d[0], coordinates_3d[1], coordinates_3d[2]),
        &circuit_region_id,
        class_mem_props,
    );
    let error_params = classifier_answer_area_params(
        error_area_id.clone(),
        format!("{name} error"),
        (coordinates_3d[0], coordinates_3d[1] + 2, coordinates_3d[2]),
        &circuit_region_id,
        "error",
    );
    let correct_params = classifier_answer_area_params(
        correct_area_id.clone(),
        format!("{name} correct"),
        (coordinates_3d[0], coordinates_3d[1] + 4, coordinates_3d[2]),
        &circuit_region_id,
        "correct",
    );

    let genome_service = state.genome_service.as_ref();
    let created_areas = match genome_service
        .create_cortical_areas(vec![
            sample_mem_params,
            class_mem_params,
            error_params,
            correct_params,
        ])
        .await
    {
        Ok(created_areas) => created_areas,
        Err(error) => {
            let _ = state
                .connectome_service
                .delete_brain_region(&circuit_region_id)
                .await;
            return Err(ApiError::internal(format!(
                "Failed to create classifier assembly areas: {error}"
            )));
        }
    };
    let associative_window = created_areas
        .iter()
        .find(|area| area.cortical_id == sample_mem_id)
        .and_then(|area| area.temporal_depth)
        .ok_or_else(|| {
            ApiError::internal(
                "Classifier sample memory is missing temporal_depth; associative mapping cannot be created".to_string(),
            )
        })?;

    let connectome_service = state.connectome_service.as_ref();
    connectome_service
        .update_cortical_mapping(
            sample_mem_id.clone(),
            class_mem_id.clone(),
            vec![classifier_mapping_rule(
                "associative_memory",
                associative_window,
            )],
        )
        .await
        .map_err(|e| {
            ApiError::internal(format!(
                "Failed to map sample memory -> class memory: {}",
                e
            ))
        })?;

    let classifier_id = feagi_structures::genomic::brain_regions::RegionID::new().to_string();
    let classifier = feagi_structures::genomic::classifiers::Classifier {
        classifier_id: classifier_id.clone(),
        name: name.to_string(),
        parent_region_id: circuit_region_id.clone(),
        coordinates_3d: coordinates,
        sample_area_ids: Vec::new(),
        class_bindings: Vec::new(),
        sample_memory_id: sample_mem_id.clone(),
        class_memory_id: class_mem_id.clone(),
        error_area_id: error_area_id.clone(),
        reward_training: false,
        answer_feedback_area_id: None,
        correct_area_id: correct_area_id.clone(),
        reward_modulator_ids: Vec::new(),
        answer_latency_bursts: 0,
        learn_area_id: None,
        confidence_area_id: None,
        properties: HashMap::new(),
    };
    let mut classifier = classifier;
    ensure_classifier_answer_modulators(&state, &mut classifier).await?;
    connectome_service
        .upsert_classifier(classifier)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to register classifier: {}", e)))?;

    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        "Classifier assembly created".to_string(),
    );
    response.insert("classifier_id".to_string(), classifier_id);
    response.insert("sample_memory_id".to_string(), sample_mem_id);
    response.insert("class_memory_id".to_string(), class_mem_id);
    response.insert("error_area_id".to_string(), error_area_id);
    response.insert("correct_area_id".to_string(), correct_area_id);
    response.insert("circuit_region_id".to_string(), circuit_region_id);
    response.insert("name".to_string(), name.to_string());
    Ok(Json(response))
}

pub async fn list_classifiers(
    State(state): State<ApiState>,
) -> ApiResult<Json<Vec<feagi_services::types::ClassifierInfo>>> {
    let classifiers = state
        .connectome_service
        .list_classifiers()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list classifiers: {}", e)))?;
    Ok(Json(classifiers))
}

#[utoipa::path(
    get,
    path = "/v1/cortical_area/classifier/{classifier_id}",
    tag = "cortical_area"
)]
pub async fn get_classifier(
    State(state): State<ApiState>,
    Path(classifier_id): Path<String>,
) -> ApiResult<Json<feagi_services::types::ClassifierInfo>> {
    state
        .connectome_service
        .get_classifier(&classifier_id)
        .await
        .map(Json)
        .map_err(|e| match e {
            feagi_services::types::errors::ServiceError::NotFound { resource, id } => {
                ApiError::not_found(&resource, &id)
            }
            other => ApiError::internal(format!("Failed to get classifier: {}", other)),
        })
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateClassifierRequest {
    pub name: Option<String>,
    pub coordinates_3d: Option<[i32; 3]>,
    pub parent_region_id: Option<String>,
    #[serde(default)]
    pub reward_training: Option<bool>,
    /// Empty string clears the correct-answer area. Absent leaves it unchanged.
    #[serde(default)]
    pub answer_feedback_area_id: Option<String>,
    #[serde(default)]
    pub answer_latency_bursts: Option<u32>,
    /// Empty string clears the learn line. Absent leaves it unchanged.
    #[serde(default)]
    pub learn_area_id: Option<String>,
    /// Empty string clears the confidence OPU. Absent leaves it unchanged.
    #[serde(default)]
    pub confidence_area_id: Option<String>,
    /// `synaptic.reward` modulators the associative mapping learns from. Replaces the list.
    /// Absent leaves it unchanged; turning reward training on with an empty list selects the
    /// classifier's correct-answer and error modulators.
    #[serde(default)]
    pub reward_modulator_ids: Option<Vec<String>>,
}

/// Rejects ids that are not `synaptic.reward` modulators.
async fn validated_reward_modulator_ids(
    state: &ApiState,
    ids: Vec<String>,
) -> ApiResult<Vec<String>> {
    let mut accepted: Vec<String> = Vec::new();
    for id in ids {
        let id = id.trim().to_string();
        if id.is_empty() || accepted.contains(&id) {
            continue;
        }
        let body = state
            .genome_service
            .get_modulator(&id)
            .await
            .map_err(|e| ApiError::invalid_input(format!("reward modulator not found: {e}")))?;
        if body.get("type").and_then(|value| value.as_str()) != Some("synaptic.reward") {
            return Err(ApiError::invalid_input(format!(
                "modulator '{id}' is not a Reward modulator"
            )));
        }
        accepted.push(id);
    }
    Ok(accepted)
}

/// Settings shared by the classifier's 1x1x1 error and correct areas.
fn classifier_answer_area_params(
    cortical_id: String,
    name: String,
    position: (i32, i32, i32),
    circuit_region_id: &str,
    role: &str,
) -> feagi_services::types::CreateCorticalAreaParams {
    let mut props = HashMap::new();
    props.insert("classifier_role".to_string(), serde_json::json!(role));
    props.insert(
        "parent_region_id".to_string(),
        serde_json::Value::String(circuit_region_id.to_string()),
    );
    props.insert("classifier_assembly".to_string(), serde_json::json!(true));
    feagi_services::types::CreateCorticalAreaParams {
        cortical_id,
        name,
        dimensions: (1, 1, 1),
        position,
        area_type: "Custom".to_string(),
        visible: Some(true),
        sub_group: None,
        neurons_per_voxel: Some(1),
        postsynaptic_current: None,
        plasticity_constant: Some(0.0),
        degeneration: Some(0.0),
        psp_uniform_distribution: Some(false),
        firing_threshold_increment: Some(0.0),
        firing_threshold_limit: Some(0.0),
        consecutive_fire_count: Some(0),
        snooze_period: Some(0),
        refractory_period: Some(0),
        leak_coefficient: Some(1.0),
        leak_variability: Some(0.0),
        burst_engine_active: Some(true),
        properties: Some(props),
    }
}

/// Creates the error or correct area when the classifier has none: made before the correct
/// area existed, or the area was deleted.
async fn ensure_classifier_answer_areas(
    state: &ApiState,
    classifier: &mut feagi_structures::genomic::classifiers::Classifier,
) -> ApiResult<bool> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let [x, y, z] = classifier.coordinates_3d;
    let mut created = false;
    for (role, y_offset) in [("error", 2), ("correct", 4)] {
        let current = if role == "error" {
            &classifier.error_area_id
        } else {
            &classifier.correct_area_id
        };
        if !current.is_empty() {
            continue;
        }
        // The id is name-derived; step the suffix past any id already in use.
        let mut area_id = String::new();
        for suffix in 3u8..=255 {
            let candidate = classifier_owned_area_id(b'c', &classifier.name, timestamp, suffix)?;
            if state
                .connectome_service
                .get_cortical_area(&candidate)
                .await
                .is_err()
            {
                area_id = candidate;
                break;
            }
        }
        if area_id.is_empty() {
            return Err(ApiError::internal(
                "No free id for a classifier answer area".to_string(),
            ));
        }
        state
            .genome_service
            .create_cortical_areas(vec![classifier_answer_area_params(
                area_id.clone(),
                format!("{} {role}", classifier.name),
                (x, y + y_offset, z),
                &classifier.parent_region_id,
                role,
            )])
            .await
            .map_err(|e| {
                ApiError::internal(format!("Failed to create classifier {role} area: {e}"))
            })?;
        if role == "error" {
            classifier.error_area_id = area_id;
        } else {
            classifier.correct_area_id = area_id;
        }
        created = true;
    }
    Ok(created)
}

/// Reward modulator adopting `area_id`, created as `name` when none exists yet.
async fn ensure_answer_modulator(
    state: &ApiState,
    area_id: &str,
    name: String,
    magnitude_percent: f32,
) -> ApiResult<String> {
    let modulators = state
        .genome_service
        .list_modulators()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list modulators: {e}")))?;
    if let Some(object) = modulators.as_object() {
        for (id, body) in object {
            if body.get("driver_cortical_id").and_then(|v| v.as_str()) != Some(area_id) {
                continue;
            }
            // One modulator per driver: reuse it only if it can serve as this answer signal.
            let is_reward = body.get("type").and_then(|v| v.as_str()) == Some("synaptic.reward");
            let magnitude = body
                .get("magnitude_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            if is_reward && magnitude.signum() == magnitude_percent.signum() && magnitude != 0.0 {
                return Ok(id.clone());
            }
            return Err(ApiError::invalid_input(format!(
                "modulator '{id}' already uses this classifier answer area but is not a {} Reward modulator",
                if magnitude_percent < 0.0 { "negative" } else { "positive" }
            )));
        }
    }
    let write = feagi_evolutionary::ModulatorWrite {
        kind: "synaptic.reward".to_string(),
        magnitude_percent,
        effect_duration_bursts: 1,
        rest_bursts: 0,
        graded: false,
        full_scale_potential: None,
        driver_cortical_id: Some(area_id.to_string()),
    };
    let mut id = name.clone();
    let mut attempt = 1;
    loop {
        match state
            .genome_service
            .create_modulator(id.clone(), write.clone())
            .await
        {
            Ok(_) => return Ok(id),
            Err(feagi_services::types::errors::ServiceError::Conflict(_)) if attempt < 100 => {
                attempt += 1;
                id = format!("{name} {attempt}");
            }
            Err(err) => {
                return Err(ApiError::internal(format!(
                    "Failed to create answer modulator: {err}"
                )))
            }
        }
    }
}

/// The classifier's correct-answer (+) and error (-) Reward modulators, created if missing.
async fn ensure_classifier_answer_modulators(
    state: &ApiState,
    classifier: &mut feagi_structures::genomic::classifiers::Classifier,
) -> ApiResult<[String; 2]> {
    ensure_classifier_answer_areas(state, classifier).await?;
    let right = ensure_answer_modulator(
        state,
        &classifier.correct_area_id,
        format!("{} correct answer", classifier.name),
        100.0,
    )
    .await?;
    let wrong = ensure_answer_modulator(
        state,
        &classifier.error_area_id,
        format!("{} error", classifier.name),
        -100.0,
    )
    .await?;
    Ok([right, wrong])
}

/// Bursts from a class input to its class output, from the class input -> class memory rules.
async fn classifier_answer_delay_bursts(
    state: &ApiState,
    classifier: &feagi_structures::genomic::classifiers::Classifier,
) -> u32 {
    let mut input_delay: Option<u64> = None;
    for binding in &classifier.class_bindings {
        let Ok(area) = state
            .connectome_service
            .get_cortical_area(&binding.class_area_id)
            .await
        else {
            continue;
        };
        let delay = area
            .properties
            .get("cortical_mapping_dst")
            .and_then(|dst| dst.get(&classifier.class_memory_id))
            .and_then(|rules| rules.as_array())
            .and_then(|rules| {
                rules
                    .iter()
                    .filter_map(|rule| rule.get("synaptic_delay_bursts")?.as_u64())
                    .max()
            });
        input_delay = input_delay.max(delay);
    }
    feagi_structures::genomic::classifiers::classifier_answer_delay_bursts(
        input_delay.unwrap_or(1) as u32
    )
}

fn classifier_from_info(
    existing: feagi_services::types::ClassifierInfo,
) -> feagi_structures::genomic::classifiers::Classifier {
    feagi_structures::genomic::classifiers::Classifier {
        classifier_id: existing.classifier_id,
        name: existing.name,
        parent_region_id: existing.parent_region_id,
        coordinates_3d: existing.coordinates_3d,
        sample_area_ids: existing.sample_area_ids,
        class_bindings: existing.class_bindings,
        sample_memory_id: existing.sample_memory_id,
        class_memory_id: existing.class_memory_id,
        error_area_id: existing.error_area_id,
        reward_training: existing.reward_training,
        answer_feedback_area_id: existing.answer_feedback_area_id,
        correct_area_id: existing.correct_area_id,
        reward_modulator_ids: existing.reward_modulator_ids,
        answer_latency_bursts: existing.answer_latency_bursts,
        learn_area_id: existing.learn_area_id,
        confidence_area_id: existing.confidence_area_id,
        properties: existing.properties,
    }
}

#[allow(dead_code)]
async fn remap_classifier_input(
    connectome_service: &Arc<dyn feagi_services::ConnectomeService + Send + Sync>,
    old_src: Option<&str>,
    new_src: Option<&str>,
    dst: &str,
    morphology: &str,
    associative_window: u32,
    kernel_memory_id: &str,
) -> ApiResult<()> {
    if old_src == new_src {
        let _ = kernel_memory_id;
        return Ok(());
    }
    let _ = kernel_memory_id;
    if let Some(old_src) = old_src {
        let old_area = connectome_service
            .get_cortical_area(old_src)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to read classifier mapping {} -> {}: {}",
                    old_src, dst, e
                ))
            })?;
        let remaining = classifier_edge_rules_without_morphology(
            &mapping_rules_for_destination(&old_area.properties, dst),
            morphology,
        );
        connectome_service
            .update_cortical_mapping(old_src.to_string(), dst.to_string(), remaining)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to clear classifier mapping {} -> {}: {}",
                    old_src, dst, e
                ))
            })?;
    }
    if let Some(new_src) = new_src {
        let new_area = connectome_service
            .get_cortical_area(new_src)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to read classifier mapping {} -> {}: {}",
                    new_src, dst, e
                ))
            })?;
        let rules = classifier_edge_rules_with_morphology(
            &mapping_rules_for_destination(&new_area.properties, dst),
            morphology,
            associative_window,
        );
        connectome_service
            .update_cortical_mapping(new_src.to_string(), dst.to_string(), rules)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to set classifier mapping {} -> {}: {}",
                    new_src, dst, e
                ))
            })?;
    }
    Ok(())
}

async fn sync_classifier_reward_mapping(
    state: &ApiState,
    classifier: &feagi_structures::genomic::classifiers::Classifier,
    associative_window: u32,
) -> ApiResult<()> {
    let kernel_area = state
        .connectome_service
        .get_cortical_area(&classifier.sample_memory_id)
        .await
        .map_err(|e| ApiError::internal(format!("Classifier kernel memory is missing: {}", e)))?;
    let existing = existing_associative_rule(&kernel_area.properties, &classifier.class_memory_id);
    let rule = if classifier.reward_training {
        let answer_delay = classifier_answer_delay_bursts(state, classifier).await;
        classifier_reward_mapping_rule(
            associative_window,
            &existing,
            &classifier.reward_modulator_ids,
            feagi_structures::genomic::classifiers::classifier_reward_eligibility_bursts(
                answer_delay,
            ),
        )
    } else {
        classifier_plain_associative_rule(associative_window, &existing)
    };
    state
        .connectome_service
        .update_cortical_mapping(
            classifier.sample_memory_id.clone(),
            classifier.class_memory_id.clone(),
            vec![rule],
        )
        .await
        .map_err(|e| match e {
            feagi_services::types::errors::ServiceError::InvalidInput(message) => {
                ApiError::invalid_input(format!(
                    "Classifier associative mapping rejected: {message}"
                ))
            }
            other => ApiError::internal(format!(
                "Failed to update classifier associative mapping: {}",
                other
            )),
        })?;
    Ok(())
}

/// Creates the classifier's correct area and its "correct answer" (+) and "error" (-) Reward
/// modulators when missing, and returns their ids. Idempotent; upgrades classifiers made
/// before these existed.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/classifier/{classifier_id}/answer_modulators",
    tag = "cortical_area"
)]
pub async fn post_classifier_answer_modulators(
    State(state): State<ApiState>,
    Path(classifier_id): Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let existing = state
        .connectome_service
        .get_classifier(&classifier_id)
        .await
        .map_err(|e| match e {
            feagi_services::types::errors::ServiceError::NotFound { resource, id } => {
                ApiError::not_found(&resource, &id)
            }
            other => ApiError::internal(format!("Failed to get classifier: {}", other)),
        })?;
    let mut classifier = classifier_from_info(existing);
    let had_answer_areas =
        !classifier.correct_area_id.is_empty() && !classifier.error_area_id.is_empty();
    let [correct, error] = ensure_classifier_answer_modulators(&state, &mut classifier).await?;
    if !had_answer_areas {
        state
            .connectome_service
            .upsert_classifier(classifier.clone())
            .await
            .map_err(|e| ApiError::internal(format!("Failed to update classifier: {}", e)))?;
    }
    Ok(Json(HashMap::from([
        ("correct_answer".to_string(), correct),
        ("error".to_string(), error),
        ("correct_area_id".to_string(), classifier.correct_area_id),
    ])))
}

#[utoipa::path(
    put,
    path = "/v1/cortical_area/classifier/{classifier_id}",
    tag = "cortical_area"
)]
pub async fn update_classifier(
    State(state): State<ApiState>,
    Path(classifier_id): Path<String>,
    Json(request): Json<UpdateClassifierRequest>,
) -> ApiResult<Json<feagi_services::types::ClassifierInfo>> {
    let existing = state
        .connectome_service
        .get_classifier(&classifier_id)
        .await
        .map_err(|e| match e {
            feagi_services::types::errors::ServiceError::NotFound { resource, id } => {
                ApiError::not_found(&resource, &id)
            }
            other => ApiError::internal(format!("Failed to get classifier: {}", other)),
        })?;
    let mut classifier = classifier_from_info(existing);
    let circuit_hosted = state
        .connectome_service
        .get_brain_region(&classifier.parent_region_id)
        .await
        .ok()
        .and_then(|region| {
            region
                .properties
                .get("classifier_circuit")
                .and_then(|value| value.as_bool())
        })
        == Some(true);
    // The circuit stays the parent of the areas. Moving those areas onto root
    // is rejected. A parent change from the editor is ignored for a circuit.
    let parent_region_id = if circuit_hosted {
        None
    } else {
        request.parent_region_id
    };
    classifier
        .apply_assembly_update(request.name, request.coordinates_3d, parent_region_id)
        .map_err(ApiError::invalid_input)?;
    if circuit_hosted {
        let mut region_changes = HashMap::new();
        region_changes.insert("name".to_string(), serde_json::json!(classifier.name));
        region_changes.insert("title".to_string(), serde_json::json!(classifier.name));
        region_changes.insert(
            "coordinate_3d".to_string(),
            serde_json::json!(classifier.coordinates_3d),
        );
        region_changes.insert(
            "coordinate_2d".to_string(),
            serde_json::json!([classifier.coordinates_3d[0], classifier.coordinates_3d[2]]),
        );
        state
            .connectome_service
            .update_brain_region(&classifier.parent_region_id, region_changes)
            .await
            .map_err(|error| {
                ApiError::internal(format!("Failed to update classifier circuit: {error}"))
            })?;
    }
    if let Some(enabled) = request.reward_training {
        classifier.reward_training = enabled;
    }
    if let Some(feedback_area_id) = request.answer_feedback_area_id.clone() {
        if feedback_area_id.trim().is_empty() {
            classifier.answer_feedback_area_id = None;
        } else {
            state
                .connectome_service
                .get_cortical_area(feedback_area_id.trim())
                .await
                .map_err(|e| {
                    ApiError::invalid_input(format!("answer_feedback_area_id not found: {}", e))
                })?;
            classifier.answer_feedback_area_id = Some(feedback_area_id.trim().to_string());
        }
    }
    if let Some(latency) = request.answer_latency_bursts {
        classifier.answer_latency_bursts = latency;
    }
    if let Some(learn_area_id) = request.learn_area_id.clone() {
        if learn_area_id.trim().is_empty() {
            classifier.learn_area_id = None;
        } else {
            state
                .connectome_service
                .get_cortical_area(learn_area_id.trim())
                .await
                .map_err(|e| ApiError::invalid_input(format!("learn_area_id not found: {}", e)))?;
            classifier.learn_area_id = Some(learn_area_id.trim().to_string());
        }
    }
    if let Some(confidence_area_id) = request.confidence_area_id.clone() {
        if confidence_area_id.trim().is_empty() {
            classifier.confidence_area_id = None;
        } else {
            state
                .connectome_service
                .get_cortical_area(confidence_area_id.trim())
                .await
                .map_err(|e| {
                    ApiError::invalid_input(format!("confidence_area_id not found: {}", e))
                })?;
            classifier.confidence_area_id = Some(confidence_area_id.trim().to_string());
        }
    }
    let sample_mem = state
        .connectome_service
        .get_cortical_area(&classifier.sample_memory_id)
        .await
        .map_err(|e| ApiError::internal(format!("Classifier sample memory is missing: {}", e)))?;
    let associative_window = sample_mem.temporal_depth.ok_or_else(|| {
        ApiError::internal(
            "Classifier sample memory is missing temporal_depth; mappings cannot be updated"
                .to_string(),
        )
    })?;
    // Validate before creating anything, so a rejected request leaves nothing behind.
    let requested_sources = match request.reward_modulator_ids.clone() {
        Some(ids) => Some(validated_reward_modulator_ids(&state, ids).await?),
        None => None,
    };
    let answer_areas_before = (
        classifier.error_area_id.clone(),
        classifier.correct_area_id.clone(),
    );
    let answer_modulators = ensure_classifier_answer_modulators(&state, &mut classifier).await?;
    if answer_areas_before
        != (
            classifier.error_area_id.clone(),
            classifier.correct_area_id.clone(),
        )
    {
        // Record the new owned areas now; a later failure must not orphan them.
        state
            .connectome_service
            .upsert_classifier(classifier.clone())
            .await
            .map_err(|e| ApiError::internal(format!("Failed to update classifier: {}", e)))?;
    }
    let mut sources_changed = requested_sources.is_some();
    if let Some(ids) = requested_sources {
        classifier.reward_modulator_ids = ids;
    }
    // R-STDP needs at least one reward source; an empty selection means the classifier's own.
    if classifier.reward_training && classifier.reward_modulator_ids.is_empty() {
        classifier.reward_modulator_ids = answer_modulators.to_vec();
        sources_changed = true;
    }
    if request.reward_training.is_some() || sources_changed {
        sync_classifier_reward_mapping(&state, &classifier, associative_window).await?;
    }
    state
        .connectome_service
        .upsert_classifier(classifier.clone())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update classifier: {}", e)))?;
    for owned_id in classifier.owned_area_ids() {
        let mut owned_changes = HashMap::new();
        owned_changes.insert(
            "parent_region_id".to_string(),
            serde_json::json!(classifier.parent_region_id),
        );
        if owned_id == classifier.sample_memory_id {
            owned_changes.insert(
                "coordinates_3d".to_string(),
                serde_json::json!(classifier.coordinates_3d),
            );
            owned_changes.insert("visible".to_string(), serde_json::json!(true));
        }
        state
            .genome_service
            .update_cortical_area(&owned_id, owned_changes)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to update classifier internal {}: {}",
                    owned_id, e
                ))
            })?;
    }
    Ok(Json(feagi_services::types::ClassifierInfo::from(
        classifier,
    )))
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ClassifierFieldRequest {
    pub field_area_id: String,
}

pub async fn post_classifier_field(
    State(_state): State<ApiState>,
    Path(_classifier_id): Path<String>,
    Json(_request): Json<ClassifierFieldRequest>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::invalid_input(
        "Classifier fields are gone. Map the area with classifier_sample or classifier_class.",
    ))
}

pub async fn delete_classifier_field(
    State(_state): State<ApiState>,
    Path(_classifier_id): Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::invalid_input(
        "Classifier fields are gone. Remove the classifier_sample or classifier_class mapping.",
    ))
}

pub async fn delete_classifier(
    State(state): State<ApiState>,
    Path(classifier_id): Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    state
        .connectome_service
        .delete_classifier(&classifier_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete classifier: {}", e)))?;
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        "Classifier assembly deleted".to_string(),
    );
    response.insert("classifier_id".to_string(), classifier_id);
    Ok(Json(response))
}

/// Identity keys added by `get_cortical_area_properties`. They describe the source
/// area itself and must not be stored on the clone.
const CLONE_IDENTITY_PROPERTY_KEYS: &[&str] = &[
    "cortical_id",
    "cortical_id_s",
    "cortical_idx",
    "name",
    "cortical_name",
    "area_type",
    "dimensions",
    "position",
    "block_boundaries",
    "cortical_dimensions",
];

/// Build the property map for a cloned cortical area.
///
/// `raw_properties` is the unfiltered map from `ConnectomeService::get_cortical_area_properties`.
/// The cortical-area read DTO removes neural parameters from `properties` because those
/// values are also returned as typed fields. Cloning that stripped map drops them:
/// `create_cortical_areas` replaces the new area's property bag with the request map
/// after the typed create fields are written.
///
/// Mappings are removed here and reapplied through `update_cortical_mapping` so
/// synapses are generated for the clone.
pub(crate) fn properties_for_cortical_clone(
    raw_properties: HashMap<String, serde_json::Value>,
    parent_region_id: &str,
    coordinates_2d: [i32; 2],
    coordinates_3d: [i32; 3],
) -> HashMap<String, serde_json::Value> {
    let mut cloned = raw_properties;
    for key in CLONE_IDENTITY_PROPERTY_KEYS {
        cloned.remove(*key);
    }
    cloned.remove("cortical_mapping_dst");

    cloned.insert(
        "parent_region_id".to_string(),
        serde_json::Value::String(parent_region_id.to_string()),
    );

    let coord_2d = serde_json::json!([coordinates_2d[0], coordinates_2d[1]]);
    cloned.insert("coordinate_2d".to_string(), coord_2d.clone());
    // Parsed genomes store `2d_coordinate`; some clients send `coordinates_2d`.
    // Rewrite whichever aliases the source actually has so they do not keep the old placement.
    for key in ["2d_coordinate", "coordinates_2d"] {
        if cloned.contains_key(key) {
            cloned.insert(key.to_string(), coord_2d.clone());
        }
    }

    let coord_3d = serde_json::json!([coordinates_3d[0], coordinates_3d[1], coordinates_3d[2]]);
    for key in ["coordinates_3d", "relative_coordinate"] {
        if cloned.contains_key(key) {
            cloned.insert(key.to_string(), coord_3d.clone());
        }
    }

    cloned
}

/// Clone an existing cortical area, including its stored properties and neural parameters.
#[utoipa::path(post, path = "/v1/cortical_area/clone", tag = "cortical_area")]
pub async fn post_clone(
    State(state): State<ApiState>,
    Json(request): Json<CloneCorticalAreaRequest>,
) -> ApiResult<Json<HashMap<String, String>>> {
    use base64::{engine::general_purpose, Engine as _};
    use feagi_services::types::CreateCorticalAreaParams;
    use feagi_structures::genomic::cortical_area::CorticalID;
    use std::time::{SystemTime, UNIX_EPOCH};

    let genome_service = state.genome_service.as_ref();
    let connectome_service = state.connectome_service.as_ref();

    // Resolve + validate source cortical ID.
    let source_id = request.source_area_id.clone();
    let source_typed = CorticalID::try_from_base_64(&source_id)
        .map_err(|e| ApiError::invalid_input(e.to_string()))?;
    let src_first_byte = source_typed.as_bytes()[0];
    if src_first_byte != b'c' && src_first_byte != b'm' {
        return Err(ApiError::invalid_input(format!(
            "Cloning is only supported for custom ('c') and memory ('m') cortical areas (got prefix byte: {})",
            src_first_byte
        )));
    }

    // Fetch full source info (dimensions, neural params, properties, mappings).
    let source_area = connectome_service
        .get_cortical_area(&source_id)
        .await
        .map_err(|e| ApiError::not_found("CorticalArea", &e.to_string()))?;

    // FEAGI is the source of truth for brain-region membership.
    //
    // Do NOT trust the client/UI to provide parent_region_id correctly, because FEAGI already
    // knows the source area’s parent. We use FEAGI’s view of parent_region_id for persistence.
    //
    // If the client provides parent_region_id and it disagrees, fail fast to prevent ambiguity.
    let source_parent_region_id = source_area
        .parent_region_id
        .clone()
        .or_else(|| {
            source_area
                .properties
                .get("parent_region_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| {
            ApiError::internal(format!(
                "Source cortical area {} is missing parent_region_id; cannot determine region membership for clone",
                source_id
            ))
        })?;

    if let Some(client_parent_region_id) = request.parent_region_id.as_ref() {
        if client_parent_region_id != &source_parent_region_id {
            return Err(ApiError::invalid_input(format!(
                "parent_region_id mismatch for clone request: client sent '{}', but FEAGI source area {} belongs to '{}'",
                client_parent_region_id, source_id, source_parent_region_id
            )));
        }
    }

    // The read DTO strips neural parameters out of `properties`. Clone from the
    // raw stored map so those parameters are replicated.
    let raw_properties = connectome_service
        .get_cortical_area_properties(&source_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to read source properties: {}", e)))?;

    // Extract outgoing mappings (we will apply them after creation, via update_cortical_mapping).
    let outgoing_mapping_dst = raw_properties
        .get("cortical_mapping_dst")
        .and_then(|v| v.as_object())
        .cloned();

    // Generate unique cortical ID for the clone.
    //
    // Rules:
    // - Byte 0 keeps the source type discriminator (b'c' or b'm')
    // - Bytes 1-6 derived from new_name (alphanumeric/_ only)
    // - Byte 7 timestamp lower byte for uniqueness
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| ApiError::internal(format!("System clock error: {}", e)))?
        .as_millis() as u64;

    let mut cortical_id_bytes = [0u8; 8];
    cortical_id_bytes[0] = src_first_byte;

    let name_bytes = request.new_name.as_bytes();
    for i in 1..7 {
        cortical_id_bytes[i] = if i - 1 < name_bytes.len() {
            let c = name_bytes[i - 1];
            if c.is_ascii_alphanumeric() || c == b'_' {
                c
            } else {
                b'_'
            }
        } else {
            b'_'
        };
    }
    cortical_id_bytes[7] = (timestamp & 0xFF) as u8;

    let new_area_id = general_purpose::STANDARD.encode(cortical_id_bytes);

    // Clone the full stored property bag. Mappings are omitted and reapplied below.
    let cloned_properties = properties_for_cortical_clone(
        raw_properties,
        &source_parent_region_id,
        request.coordinates_2d,
        request.coordinates_3d,
    );

    let params = CreateCorticalAreaParams {
        cortical_id: new_area_id.clone(),
        name: request.new_name.clone(),
        dimensions: source_area.dimensions,
        position: (
            request.coordinates_3d[0],
            request.coordinates_3d[1],
            request.coordinates_3d[2],
        ),
        area_type: source_area.area_type.clone(),
        visible: Some(source_area.visible),
        sub_group: source_area.sub_group.clone(),
        neurons_per_voxel: Some(source_area.neurons_per_voxel),
        postsynaptic_current: Some(source_area.postsynaptic_current),
        plasticity_constant: Some(source_area.plasticity_constant),
        degeneration: Some(source_area.degeneration),
        psp_uniform_distribution: Some(source_area.psp_uniform_distribution),
        // Note: FEAGI core currently accepts scalar firing_threshold_increment on create.
        // We preserve full source properties above; the service layer remains authoritative.
        firing_threshold_increment: None,
        firing_threshold_limit: Some(source_area.firing_threshold_limit),
        consecutive_fire_count: Some(source_area.consecutive_fire_count),
        snooze_period: Some(source_area.snooze_period),
        refractory_period: Some(source_area.refractory_period),
        leak_coefficient: Some(source_area.leak_coefficient),
        leak_variability: Some(source_area.leak_variability),
        burst_engine_active: Some(source_area.burst_engine_active),
        properties: Some(cloned_properties),
    };

    // Create the cloned area via GenomeService (proper flow: genome update → neuroembryogenesis → NPU).
    let created_areas = genome_service
        .create_cortical_areas(vec![params])
        .await
        .map_err(|e| ApiError::internal(format!("Failed to clone cortical area: {}", e)))?;

    // DIAGNOSTIC: Log what coordinates were returned after creation
    if let Some(created_area) = created_areas.first() {
        tracing::info!(target: "feagi-api",
            "Clone created area {} with position {:?} (requested {:?})",
            new_area_id, created_area.position, request.coordinates_3d
        );
    }

    // Optionally clone cortical mappings (AutoWiring).
    if request.clone_cortical_mapping {
        // 1) Outgoing mappings: source -> dst becomes new -> dst
        if let Some(dst_map) = outgoing_mapping_dst {
            for (dst_id, rules) in dst_map {
                let dst_effective = if dst_id == source_id {
                    // Self-loop on source should become self-loop on clone.
                    new_area_id.clone()
                } else {
                    dst_id.clone()
                };

                let Some(rules_array) = rules.as_array() else {
                    return Err(ApiError::invalid_input(format!(
                        "Invalid cortical_mapping_dst value for dst '{}': expected array, got {}",
                        dst_id, rules
                    )));
                };

                connectome_service
                    .update_cortical_mapping(
                        new_area_id.clone(),
                        dst_effective,
                        rules_array.clone(),
                    )
                    .await
                    .map_err(|e| {
                        ApiError::internal(format!(
                            "Failed to clone outgoing mapping from {}: {}",
                            source_id, e
                        ))
                    })?;
            }
        }

        // 2) Incoming mappings: any src -> source becomes src -> new
        // We discover these by scanning all areas' cortical_mapping_dst maps.
        let all_areas = connectome_service
            .list_cortical_areas()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to list cortical areas: {}", e)))?;

        for area in all_areas {
            // Skip the source area itself: source->* already handled by outgoing clone above.
            if area.cortical_id == source_id {
                continue;
            }

            let Some(dst_map) = area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
            else {
                continue;
            };

            let Some(rules) = dst_map.get(&source_id) else {
                continue;
            };

            let Some(rules_array) = rules.as_array() else {
                return Err(ApiError::invalid_input(format!(
                    "Invalid cortical_mapping_dst value for src '{}', dst '{}': expected array, got {}",
                    area.cortical_id, source_id, rules
                )));
            };

            connectome_service
                .update_cortical_mapping(
                    area.cortical_id.clone(),
                    new_area_id.clone(),
                    rules_array.clone(),
                )
                .await
                .map_err(|e| {
                    ApiError::internal(format!(
                        "Failed to clone incoming mapping into {} from {}: {}",
                        source_id, area.cortical_id, e
                    ))
                })?;
        }
    }

    Ok(Json(HashMap::from([
        ("message".to_string(), "Cortical area cloned".to_string()),
        ("new_area_id".to_string(), new_area_id),
    ])))
}

/// Request payload for POST /v1/cortical_area/clone
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct CloneCorticalAreaRequest {
    /// Base64 cortical area ID to clone.
    pub source_area_id: String,
    /// New cortical area name (display name).
    pub new_name: String,
    /// New 3D coordinates for placement.
    pub coordinates_3d: [i32; 3],
    /// New 2D coordinates for visualization placement.
    pub coordinates_2d: [i32; 2],
    /// Target parent brain region ID to attach the clone under.
    ///
    /// NOTE: FEAGI does NOT rely on the client for this value; it derives the parent from the
    /// source area’s membership. If provided and mismatched, FEAGI rejects the request.
    #[serde(default)]
    pub parent_region_id: Option<String>,
    /// If true, clones cortical mappings (incoming + outgoing) to reproduce wiring.
    pub clone_cortical_mapping: bool,
}

/// Update properties of multiple cortical areas in a single request. (Not yet implemented)
#[utoipa::path(
    put,
    path = "/v1/cortical_area/multi/cortical_area",
    tag = "cortical_area"
)]
pub async fn put_multi_cortical_area(
    State(state): State<ApiState>,
    Json(mut request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let genome_service = state.genome_service.as_ref();

    // Extract cortical_id_list
    let cortical_ids: Vec<String> = request
        .get("cortical_id_list")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ApiError::invalid_input("cortical_id_list required"))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    if cortical_ids.is_empty() {
        return Err(ApiError::invalid_input("cortical_id_list cannot be empty"));
    }

    tracing::debug!(
        target: "feagi-api",
        "PUT /v1/cortical_area/multi/cortical_area - received update for {} areas (keys: {:?})",
        cortical_ids.len(),
        request.keys().collect::<Vec<_>>()
    );

    // Remove cortical_id_list from changes (it's not a property to update)
    request.remove("cortical_id_list");

    // Build shared properties (applies to all unless overridden per-id)
    let mut shared_properties = request.clone();
    for cortical_id in &cortical_ids {
        shared_properties.remove(cortical_id);
    }

    // Update each cortical area, using per-id properties when provided
    for cortical_id in &cortical_ids {
        tracing::debug!(target: "feagi-api", "PUT /v1/cortical_area/multi/cortical_area - updating area: {}", cortical_id);
        let mut properties = shared_properties.clone();
        if let Some(serde_json::Value::Object(per_id_map)) = request.get(cortical_id) {
            for (key, value) in per_id_map {
                properties.insert(key.clone(), value.clone());
            }
        }
        match genome_service
            .update_cortical_area(cortical_id, properties)
            .await
        {
            Ok(_) => {
                tracing::debug!(target: "feagi-api", "PUT /v1/cortical_area/multi/cortical_area - success for {}", cortical_id);
            }
            Err(e) => {
                tracing::error!(target: "feagi-api", "PUT /v1/cortical_area/multi/cortical_area - failed for {}: {}", cortical_id, e);
                return Err(ApiError::internal(format!(
                    "Failed to update cortical area {}: {}",
                    cortical_id, e
                )));
            }
        }
    }

    Ok(Json(HashMap::from([
        (
            "message".to_string(),
            format!("Updated {} cortical areas", cortical_ids.len()),
        ),
        ("cortical_ids".to_string(), cortical_ids.join(", ")),
    ])))
}

/// Delete multiple cortical areas by their IDs.
#[utoipa::path(
    delete,
    path = "/v1/cortical_area/multi/cortical_area",
    tag = "cortical_area"
)]
pub async fn delete_multi_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<Vec<String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    if request.is_empty() {
        return Err(ApiError::invalid_input(
            "Request must contain at least one cortical ID",
        ));
    }

    let connectome_service = state.connectome_service.as_ref();

    tracing::debug!(
        target: "feagi-api",
        "DELETE /v1/cortical_area/multi/cortical_area - deleting {} areas",
        request.len()
    );

    for cortical_id in &request {
        match connectome_service.delete_cortical_area(cortical_id).await {
            Ok(_) => {
                tracing::debug!(
                    target: "feagi-api",
                    "DELETE /v1/cortical_area/multi/cortical_area - deleted {}",
                    cortical_id
                );
            }
            Err(e) => {
                tracing::error!(
                    target: "feagi-api",
                    "DELETE /v1/cortical_area/multi/cortical_area - failed for {}: {}",
                    cortical_id,
                    e
                );
                return Err(ApiError::internal(format!(
                    "Failed to delete cortical area {}: {}",
                    cortical_id, e
                )));
            }
        }
    }

    Ok(Json(HashMap::from([
        (
            "message".to_string(),
            format!("Deleted {} cortical areas", request.len()),
        ),
        ("cortical_ids".to_string(), request.join(", ")),
    ])))
}

/// Update the 2D visualization coordinates of a cortical area. (Not yet implemented)
#[utoipa::path(put, path = "/v1/cortical_area/coord_2d", tag = "cortical_area")]
#[allow(unused_variables)] // In development
pub async fn put_coord_2d(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Update 2D coordinates
    Err(ApiError::internal("Not yet implemented"))
}

/// Hide/show cortical areas in visualizations. (Not yet implemented)
#[utoipa::path(
    put,
    path = "/v1/cortical_area/suppress_cortical_visibility",
    tag = "cortical_area"
)]
#[allow(unused_variables)] // In development
pub async fn put_suppress_cortical_visibility(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Suppress cortical visibility
    Err(ApiError::internal("Not yet implemented"))
}

/// Reset runtime neural state for one or more cortical areas (membrane potential, refractory
/// counters, FCL candidates). Genome, connections, and parameters are unchanged.
#[utoipa::path(
    put,
    path = "/v1/cortical_area/reset",
    tag = "cortical_area",
    request_body = CorticalAreaResetRequest,
    responses(
        (status = 200, description = "Reset applied", body = CorticalAreaResetResponse),
    )
)]
pub async fn put_reset(
    State(state): State<ApiState>,
    Json(request): Json<CorticalAreaResetRequest>,
) -> ApiResult<Json<CorticalAreaResetResponse>> {
    use tracing::info;

    if request.area_list.is_empty() {
        return Err(ApiError::invalid_input("area_list cannot be empty"));
    }

    info!(
        target: "feagi-api",
        "[RESET] Received reset request for {} cortical areas: {:?}",
        request.area_list.len(),
        request.area_list
    );

    let connectome_service = state.connectome_service.as_ref();
    let mut cortical_indices: Vec<u32> = Vec::with_capacity(request.area_list.len());
    for id in &request.area_list {
        let area = connectome_service
            .get_cortical_area(id)
            .await
            .map_err(ApiError::from)?;
        cortical_indices.push(area.cortical_idx);
        info!(
            target: "feagi-api",
            "[RESET] Resolved cortical ID '{}' to index {}",
            id,
            area.cortical_idx
        );
    }

    info!(
        target: "feagi-api",
        "[RESET] Calling runtime service to reset indices: {:?}",
        cortical_indices
    );

    let reset_pairs = state
        .runtime_service
        .reset_cortical_area_states(&cortical_indices)
        .await
        .map_err(ApiError::from)?;

    let results: Vec<CorticalAreaResetItem> = reset_pairs
        .into_iter()
        .map(|(cortical_idx, neurons_reset)| {
            info!(
                target: "feagi-api",
                "[RESET] Cortical area {} reset: {} neurons cleared",
                cortical_idx,
                neurons_reset
            );
            CorticalAreaResetItem {
                cortical_idx,
                neurons_reset,
            }
        })
        .collect();

    info!(
        target: "feagi-api",
        "[RESET] Reset complete for {} areas",
        results.len()
    );

    Ok(Json(CorticalAreaResetResponse {
        message: "ok".to_string(),
        results,
    }))
}

/// Reset learned synapse weights of every incoming mapping of the listed cortical areas to
/// each mapping's initial weight, clearing eligibility traces. Genome and neuron state are
/// unchanged (see `PUT /v1/cortical_area/reset` for membrane state).
#[utoipa::path(
    put,
    path = "/v1/cortical_area/reset_weights",
    tag = "cortical_area",
    request_body = CorticalAreaResetRequest,
    responses(
        (status = 200, description = "Weights reset", body = CorticalAreaResetWeightsResponse),
        (status = 404, description = "Cortical area not found"),
    )
)]
pub async fn put_reset_weights(
    State(state): State<ApiState>,
    Json(request): Json<CorticalAreaResetRequest>,
) -> ApiResult<Json<CorticalAreaResetWeightsResponse>> {
    if request.area_list.is_empty() {
        return Err(ApiError::invalid_input("area_list cannot be empty"));
    }
    let mut results = Vec::with_capacity(request.area_list.len());
    for id in &request.area_list {
        let info = state
            .connectome_service
            .reset_area_incoming_weights(id)
            .await
            .map_err(ApiError::from)?;
        results.push(CorticalAreaResetWeightsItem {
            cortical_id: id.clone(),
            mappings_reset: info.mappings_reset,
            synapses_reset: info.synapses_reset,
        });
    }
    Ok(Json(CorticalAreaResetWeightsResponse { results }))
}

/// Check if visualization is enabled for the system.
#[utoipa::path(get, path = "/v1/cortical_area/visualization", tag = "cortical_area")]
pub async fn get_visualization(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, bool>>> {
    let mut response = HashMap::new();
    response.insert("enabled".to_string(), true);
    Ok(Json(response))
}

/// Execute multiple cortical area operations (create, update, delete) in a single batch.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/batch_operations",
    tag = "cortical_area"
)]
pub async fn post_batch_operations(
    State(_state): State<ApiState>,
    Json(_ops): Json<Vec<HashMap<String, serde_json::Value>>>,
) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("processed".to_string(), 0);
    Ok(Json(response))
}

/// Alias for /v1/cortical_area/ipu - list all IPU cortical area IDs.
#[utoipa::path(get, path = "/v1/cortical_area/ipu/list", tag = "cortical_area")]
pub async fn get_ipu_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    get_ipu(State(state)).await
}

/// Alias for /v1/cortical_area/opu - list all OPU cortical area IDs.
#[utoipa::path(get, path = "/v1/cortical_area/opu/list", tag = "cortical_area")]
pub async fn get_opu_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    get_opu(State(state)).await
}

/// Update the 3D position of a cortical area. (Not yet implemented)
#[utoipa::path(put, path = "/v1/cortical_area/coordinates_3d", tag = "cortical_area")]
pub async fn put_coordinates_3d(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Delete multiple cortical areas by their IDs in a single operation.
#[utoipa::path(delete, path = "/v1/cortical_area/bulk_delete", tag = "cortical_area")]
pub async fn delete_bulk(
    State(_state): State<ApiState>,
    Json(_ids): Json<Vec<String>>,
) -> ApiResult<Json<HashMap<String, i32>>> {
    let mut response = HashMap::new();
    response.insert("deleted_count".to_string(), 0);
    Ok(Json(response))
}

/// Resize a cortical area by changing its dimensions. (Not yet implemented)
#[utoipa::path(post, path = "/v1/cortical_area/resize", tag = "cortical_area")]
pub async fn post_resize(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Move a cortical area to a new position. (Not yet implemented)
#[utoipa::path(post, path = "/v1/cortical_area/reposition", tag = "cortical_area")]
pub async fn post_reposition(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// List all neurons at a voxel `(x, y, z)` within a cortical area, with the same live property snapshot as `/v1/connectome/neuron_properties`.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/voxel_neurons",
    tag = "cortical_area",
    params(VoxelNeuronsQuery),
    responses(
        (status = 200, description = "Neurons in voxel", body = VoxelNeuronsResponse),
        (status = 404, description = "Cortical area or neuron data not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_voxel_neurons(
    State(state): State<ApiState>,
    Query(params): Query<VoxelNeuronsQuery>,
) -> ApiResult<Json<VoxelNeuronsResponse>> {
    resolve_voxel_neurons(
        &state,
        params.cortical_id,
        params.x,
        params.y,
        params.z,
        params.synapse_page,
    )
    .await
    .map(Json)
}

/// Same as [`get_voxel_neurons`] but accepts a JSON body (for clients that cannot use query strings).
#[utoipa::path(
    post,
    path = "/v1/cortical_area/voxel_neurons",
    tag = "cortical_area",
    request_body = VoxelNeuronsBody,
    responses(
        (status = 200, description = "Neurons in voxel", body = VoxelNeuronsResponse),
        (status = 404, description = "Cortical area or neuron data not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_voxel_neurons(
    State(state): State<ApiState>,
    Json(body): Json<VoxelNeuronsBody>,
) -> ApiResult<Json<VoxelNeuronsResponse>> {
    resolve_voxel_neurons(
        &state,
        body.cortical_id,
        body.x,
        body.y,
        body.z,
        body.synapse_page,
    )
    .await
    .map(Json)
}

/// GET /v1/cortical_area/memory — plasticity runtime stats, genome memory parameters, upstream wiring, synapse counts, and paginated memory neuron ids.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/memory",
    tag = "cortical_area",
    params(MemoryCorticalAreaQuery),
    responses(
        (status = 200, description = "Memory cortical area details", body = MemoryCorticalAreaResponse),
        (status = 400, description = "Invalid cortical id or not a memory area"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory_cortical_area(
    State(state): State<ApiState>,
    Query(params): Query<MemoryCorticalAreaQuery>,
) -> ApiResult<Json<MemoryCorticalAreaResponse>> {
    let connectome_service = state.connectome_service.as_ref();
    let area = connectome_service
        .get_cortical_area(&params.cortical_id)
        .await
        .map_err(ApiError::from)?;

    let mem_props = extract_memory_properties(&area.properties).ok_or_else(|| {
        ApiError::invalid_input(
            "cortical area is not a memory area (expected is_mem_type memory properties)",
        )
    })?;

    let cortical_idx = area.cortical_idx;
    let cortical_name = area.name.clone();

    let cid = CorticalID::try_from_base_64(&params.cortical_id)
        .map_err(|e| ApiError::invalid_input(format!("Invalid cortical_id: {}", e)))?;

    let page_size_u32 = params
        .page_size
        .clamp(1, MEMORY_CORTICAL_NEURON_IDS_PAGE_SIZE_MAX);
    let page_size = page_size_u32 as usize;
    let offset = (params.page as usize).saturating_mul(page_size);

    let manager = feagi_brain_development::ConnectomeManager::instance();
    let mgr = manager.read();

    let upstream_cortical_area_indices = mgr.get_upstream_cortical_areas(&cid);
    let upstream_cortical_area_count = upstream_cortical_area_indices.len();

    let exec = mgr
        .get_plasticity_executor()
        .ok_or_else(|| ApiError::internal("Plasticity executor not available"))?;
    let ex = exec
        .lock()
        .map_err(|_| ApiError::internal("Plasticity executor lock poisoned"))?;

    let runtime = ex
        .memory_cortical_area_runtime_info(cortical_idx)
        .ok_or_else(|| ApiError::internal("Plasticity service not initialized"))?;

    let (memory_neuron_ids_u32, total_memory_neuron_ids) = ex
        .paginated_memory_neuron_ids_in_area(cortical_idx, offset, page_size)
        .unwrap_or((Vec::new(), 0));

    let has_more = offset.saturating_add(memory_neuron_ids_u32.len()) < total_memory_neuron_ids;

    let memory_neuron_ids: Vec<u64> = memory_neuron_ids_u32
        .into_iter()
        .map(|id| id as u64)
        .collect();

    Ok(Json(MemoryCorticalAreaResponse {
        cortical_id: params.cortical_id,
        cortical_idx,
        cortical_name,
        short_term_neuron_count: runtime.short_term_neuron_count,
        long_term_neuron_count: runtime.long_term_neuron_count,
        // CorticalAreaInfo.properties drops init_lifespan, lifespan_growth_rate, and
        // longterm_mem_threshold because those are already top-level fields. Extracting
        // again from that bag substitutes MemoryAreaProperties defaults (9, 1, 100)
        // and hides the values that were written to the genome.
        memory_parameters: memory_parameters_from_reported_area(&area, &mem_props),
        evicted_total: runtime.evicted_total,
        creation_failed_total: runtime.creation_failed_total,
        formation_suppressed_total: runtime.formation_suppressed_total,
        memory_pool: MemoryPoolResponse {
            pool_capacity: runtime.memory_pool.pool_capacity,
            pool_in_use: runtime.memory_pool.pool_in_use,
            pool_ltm: runtime.memory_pool.pool_ltm,
        },
        effective_mp_mode: runtime.effective_mp_mode,
        upstream_cortical_area_indices,
        upstream_cortical_area_count,
        upstream_pattern_cache_size: runtime.upstream_pattern_cache_size,
        incoming_synapse_count: area.incoming_synapse_count,
        outgoing_synapse_count: area.outgoing_synapse_count,
        total_memory_neuron_ids,
        page: params.page,
        page_size: page_size_u32,
        memory_neuron_ids,
        has_more,
    }))
}

/// Get metadata for all available IPU types (vision, infrared, etc.). Includes encodings, formats, units, and topology.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/ipu/types",
    tag = "cortical_area",
    responses(
        (status = 200, description = "IPU type metadata", body = HashMap<String, CorticalTypeMetadata>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_ipu_types(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, CorticalTypeMetadata>>> {
    let mut types = HashMap::new();

    // Dynamically generate metadata from feagi_data_structures templates
    for unit in SensoryCorticalUnit::list_all() {
        let id_ref = unit.get_cortical_id_unit_reference();
        let key = format!("i{}", std::str::from_utf8(&id_ref).unwrap_or("???"));

        // All IPU types support both absolute and incremental encodings
        let encodings = vec!["absolute".to_string(), "incremental".to_string()];

        // Determine if formats are supported based on snake_case_name
        // Vision and SegmentedVision use CartesianPlane (no formats)
        // MiscData uses Misc (no formats)
        // All others use Percentage-based types (have formats)
        let snake_name = unit.get_snake_case_name();
        let formats = if snake_name == "vision"
            || snake_name == "segmented_vision"
            || snake_name == "depth_map"
            || snake_name == "misc_data"
            || snake_name == "miscellaneous"
        {
            vec![]
        } else {
            vec!["linear".to_string(), "fractional".to_string()]
        };

        // Default resolution based on type
        let resolution = if snake_name == "vision" {
            vec![128, 128, 3] // Simple vision default matches the Vision template (128x128 RGB)
        } else if snake_name == "segmented_vision" {
            vec![32, 32, 1] // Segmented vision segments are smaller
        } else if snake_name == "depth_map" {
            vec![64, 64, 1] // Depth map: XY topology, normalized depth as potential
        } else {
            vec![1, 1, 1] // Most sensors are scalar (1x1x1)
        };

        // Most sensors are asymmetric
        let structure = "asymmetric".to_string();

        // Get unit default topology
        let topology_map = unit.get_unit_default_topology();
        let unit_default_topology: HashMap<usize, UnitTopologyData> = topology_map
            .into_iter()
            .map(|(idx, topo)| {
                (
                    *idx as usize,
                    UnitTopologyData {
                        relative_position: topo.relative_position,
                        dimensions: topo.channel_dimensions_default,
                    },
                )
            })
            .collect();

        types.insert(
            key,
            CorticalTypeMetadata {
                description: unit.get_friendly_name().to_string(),
                encodings,
                formats,
                units: unit.get_number_cortical_areas() as u32,
                resolution,
                structure,
                unit_default_topology,
            },
        );
    }

    Ok(Json(types))
}

/// Get metadata for all available OPU types (motors, servos, etc.). Includes encodings, formats, units, and topology.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/opu/types",
    tag = "cortical_area",
    responses(
        (status = 200, description = "OPU type metadata", body = HashMap<String, CorticalTypeMetadata>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_opu_types(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, CorticalTypeMetadata>>> {
    let mut types = HashMap::new();

    // Dynamically generate metadata from feagi_data_structures templates
    for unit in MotorCorticalUnit::list_all() {
        let id_ref = unit.get_cortical_id_unit_reference();
        let key = format!("o{}", std::str::from_utf8(&id_ref).unwrap_or("???"));

        // All OPU types support both absolute and incremental encodings
        let encodings = vec!["absolute".to_string(), "incremental".to_string()];

        // Determine if formats are supported based on snake_case_name
        // MiscData uses Misc (no formats)
        // All others use Percentage-based types (have formats)
        let snake_name = unit.get_snake_case_name();
        let formats = if snake_name == "miscellaneous" {
            vec![]
        } else {
            vec!["linear".to_string(), "fractional".to_string()]
        };

        // Default resolution - all motors/actuators are typically scalar
        let resolution = vec![1, 1, 1];

        // All actuators are asymmetric
        let structure = "asymmetric".to_string();

        // Get unit default topology
        let topology_map = unit.get_unit_default_topology();
        let unit_default_topology: HashMap<usize, UnitTopologyData> = topology_map
            .into_iter()
            .map(|(idx, topo)| {
                (
                    *idx as usize,
                    UnitTopologyData {
                        relative_position: topo.relative_position,
                        dimensions: topo.channel_dimensions_default,
                    },
                )
            })
            .collect();

        types.insert(
            key,
            CorticalTypeMetadata {
                description: unit.get_friendly_name().to_string(),
                encodings,
                formats,
                units: unit.get_number_cortical_areas() as u32,
                resolution,
                structure,
                unit_default_topology,
            },
        );
    }

    Ok(Json(types))
}

/// Get list of all cortical area indices (numerical indices used internally for indexing).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_area_index_list",
    tag = "cortical_area"
)]
pub async fn get_cortical_area_index_list(
    State(state): State<ApiState>,
) -> ApiResult<Json<Vec<u32>>> {
    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    // CRITICAL FIX: Return the actual cortical_idx values, not fabricated sequential indices
    let indices: Vec<u32> = areas.iter().map(|a| a.cortical_idx).collect();
    Ok(Json(indices))
}

/// Get mapping from cortical area IDs to their internal indices. Returns {cortical_id: index}.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/cortical_idx_mapping",
    tag = "cortical_area"
)]
pub async fn get_cortical_idx_mapping(
    State(state): State<ApiState>,
) -> ApiResult<Json<std::collections::BTreeMap<String, u32>>> {
    use std::collections::BTreeMap;

    let connectome_service = state.connectome_service.as_ref();
    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("{}", e)))?;
    // CRITICAL FIX: Use the actual cortical_idx from CorticalArea, NOT enumerate() which ignores reserved indices!
    // Use BTreeMap for consistent alphabetical ordering
    let mapping: BTreeMap<String, u32> = areas
        .iter()
        .map(|a| (a.cortical_id.clone(), a.cortical_idx))
        .collect();
    Ok(Json(mapping))
}

/// Get restrictions on which cortical areas can connect to which (connection validation rules).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/mapping_restrictions",
    tag = "cortical_area"
)]
pub async fn get_mapping_restrictions_query(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// Get memory usage of a specific cortical area in bytes (calculated from neuron count).
#[utoipa::path(
    get,
    path = "/v1/cortical_area/{cortical_id}/memory_usage",
    tag = "cortical_area"
)]
pub async fn get_memory_usage(
    State(state): State<ApiState>,
    Path(cortical_id): Path<String>,
) -> ApiResult<Json<HashMap<String, i64>>> {
    let connectome_service = state.connectome_service.as_ref();

    // CRITICAL FIX: Calculate actual memory usage based on neuron count instead of hardcoded 0
    let area_info = connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .map_err(|_| ApiError::not_found("CorticalArea", &cortical_id))?;

    // Calculate memory usage: neuron_count × bytes per neuron
    // Each neuron in NeuronArray uses ~48 bytes (membrane_potential, threshold, refractory, etc.)
    const BYTES_PER_NEURON: i64 = 48;
    let memory_bytes = (area_info.neuron_count as i64) * BYTES_PER_NEURON;

    let mut response = HashMap::new();
    response.insert("memory_bytes".to_string(), memory_bytes);
    Ok(Json(response))
}

/// Get the total number of neurons in a specific cortical area.
#[utoipa::path(
    get,
    path = "/v1/cortical_area/{cortical_id}/neuron_count",
    tag = "cortical_area"
)]
pub async fn get_area_neuron_count(
    State(state): State<ApiState>,
    Path(cortical_id): Path<String>,
) -> ApiResult<Json<i64>> {
    let connectome_service = state.connectome_service.as_ref();

    // CRITICAL FIX: Get actual neuron count from ConnectomeService instead of hardcoded 0
    let area_info = connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .map_err(|_| ApiError::not_found("CorticalArea", &cortical_id))?;

    Ok(Json(area_info.neuron_count as i64))
}

/// Get available cortical type options for UI selection: Sensory, Motor, Custom, Memory.
#[utoipa::path(
    post,
    path = "/v1/cortical_area/cortical_type_options",
    tag = "cortical_area"
)]
pub async fn post_cortical_type_options(
    State(_state): State<ApiState>,
) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![
        "Sensory".to_string(),
        "Motor".to_string(),
        "Custom".to_string(),
        "Memory".to_string(),
    ]))
}

/// Get mapping restrictions for specific cortical areas (POST version with request body).
#[utoipa::path(
    post,
    path = "/v1/cortical_area/mapping_restrictions",
    tag = "cortical_area"
)]
pub async fn post_mapping_restrictions(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// Get mapping restrictions between two specific cortical areas (connection validation).
#[utoipa::path(
    post,
    path = "/v1/cortical_area/mapping_restrictions_between_areas",
    tag = "cortical_area"
)]
pub async fn post_mapping_restrictions_between_areas(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, Vec<String>>>> {
    Ok(Json(HashMap::new()))
}

/// Update 3D coordinates of a cortical area (alternative endpoint). (Not yet implemented)
#[utoipa::path(put, path = "/v1/cortical_area/coord_3d", tag = "cortical_area")]
pub async fn put_coord_3d(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

#[cfg(test)]
mod voxel_neurons_dto_tests {
    use super::{
        synapse_details_for_neuron, synapse_page_window, VoxelNeuronsBody, VoxelNeuronsResponse,
    };

    #[test]
    fn synapse_page_window_paginates_fifty_per_direction() {
        let (s, e, more) = synapse_page_window(120, 0);
        assert_eq!((s, e, more), (0, 50, true));
        let (s, e, more) = synapse_page_window(120, 1);
        assert_eq!((s, e, more), (50, 100, true));
        let (s, e, more) = synapse_page_window(120, 2);
        assert_eq!((s, e, more), (100, 120, false));
        let (s, e, more) = synapse_page_window(120, 3);
        assert_eq!((s, e, more), (0, 0, false));
    }

    #[test]
    fn synapse_details_matches_connectome_shape() {
        let mgr = feagi_brain_development::ConnectomeManager::new_for_testing();
        let out_full = vec![(10, 2.0, 5.0, 1)];
        let inc_full = vec![(3, 4.0, 6.0, 0)];
        let (out, inc) = synapse_details_for_neuron(&mgr, 7, &out_full, &inc_full);
        let out_a = out.as_array().expect("outgoing array");
        assert_eq!(out_a[0]["source_neuron_id"], serde_json::json!(7));
        assert_eq!(out_a[0]["target_neuron_id"], serde_json::json!(10));
        assert_eq!(out_a[0]["weight"], serde_json::json!(2.0));
        assert_eq!(out_a[0]["postsynaptic_potential"], serde_json::json!(5.0));
        assert_eq!(out_a[0]["synapse_type"], serde_json::json!(1));
        assert!(out_a[0].get("target_cortical_id").is_some());
        assert!(out_a[0].get("target_cortical_name").is_some());
        assert!(out_a[0].get("target_x").is_some());
        let in_a = inc.as_array().expect("incoming array");
        assert_eq!(in_a[0]["source_neuron_id"], serde_json::json!(3));
        assert_eq!(in_a[0]["target_neuron_id"], serde_json::json!(7));
        assert!(in_a[0].get("source_cortical_id").is_some());
        assert!(in_a[0].get("source_cortical_name").is_some());
        assert!(in_a[0].get("source_x").is_some());
    }

    #[test]
    fn voxel_neurons_body_deserializes_from_json() {
        let j = r#"{"cortical_id":"X19fcG93ZXI=","x":0,"y":0,"z":0}"#;
        let b: VoxelNeuronsBody = serde_json::from_str(j).expect("deserialize body");
        assert_eq!(b.cortical_id, "X19fcG93ZXI=");
        assert_eq!((b.x, b.y, b.z), (0, 0, 0));
        assert_eq!(b.synapse_page, 0);
    }

    #[test]
    fn voxel_neurons_response_serializes() {
        let r = VoxelNeuronsResponse {
            cortical_id: "id".to_string(),
            cortical_name: "test_area".to_string(),
            cortical_idx: 2,
            voxel_coordinate: [1, 2, 3],
            x: 1,
            y: 2,
            z: 3,
            synapse_page: 0,
            neuron_count: 0,
            neurons: vec![],
        };
        let v = serde_json::to_value(&r).expect("serialize");
        assert_eq!(v["cortical_name"], serde_json::json!("test_area"));
        assert_eq!(v["voxel_coordinate"], serde_json::json!([1, 2, 3]));
        assert_eq!(v["neuron_count"], serde_json::json!(0));
        assert_eq!(v["synapse_page"], serde_json::json!(0));
        assert_eq!(v["neurons"], serde_json::json!([]));
    }
}

#[cfg(test)]
mod classifier_mapping_rule_tests {
    use super::classifier_mapping_rule;

    #[test]
    fn reward_rule_is_standard_rstdp_over_the_sources() {
        let existing = serde_json::json!({
            "morphology_id": "associative_memory",
            "plasticity_constant": 3,
            "modulators": ["stale"],
            "classifier_instance_reward": true,
            "reward_source_area": "legacy",
            "punishment_source_area": "legacy",
        });
        let sources = vec!["joy".to_string(), "ouch".to_string()];
        let rule = super::classifier_reward_mapping_rule(4, &existing, &sources, 6);
        assert_eq!(rule["plasticity_mode"], "rstdp");
        assert_eq!(rule["plasticity_flag"], true);
        assert_eq!(rule["modulators"], serde_json::json!(["joy", "ouch"]));
        assert_eq!(rule["eligibility_decay_bursts"], 6);
        assert_eq!(rule["plasticity_constant"], 3, "existing tuning is kept");
        for gone in [
            "classifier_instance_reward",
            "reward_source_area",
            "punishment_source_area",
        ] {
            assert!(rule.get(gone).is_none(), "{gone} must be dropped");
        }
        let plain = super::classifier_plain_associative_rule(4, &rule);
        assert!(plain.get("modulators").is_none());
        assert!(plain.get("plasticity_mode").is_none());
        assert!(plain.get("eligibility_decay_bursts").is_none());
    }

    #[test]
    fn classifier_associative_rule_uses_memory_temporal_depth() {
        let window = feagi_evolutionary::MemoryAreaProperties::default().temporal_depth;
        assert!(window > 0);
        let associative = classifier_mapping_rule("associative_memory", window);
        assert_eq!(associative["plasticity_flag"], serde_json::json!(true));
        assert_eq!(associative["plasticity_window"], serde_json::json!(window));
        assert_eq!(associative["plasticity_constant"], serde_json::json!(1));
        assert_eq!(associative["ltp_multiplier"], serde_json::json!(1));
        assert_eq!(associative["ltd_multiplier"], serde_json::json!(1));
        let scan = classifier_mapping_rule("episodic_scan", window);
        assert_eq!(scan["plasticity_flag"], serde_json::json!(false));
        assert_eq!(scan["plasticity_window"], serde_json::json!(0));
        assert_eq!(scan["plasticity_constant"], serde_json::json!(0));
    }

    #[test]
    fn classifier_edge_is_one_rule() {
        use super::classifier_edge_rules_with_morphology;

        let encode = classifier_mapping_rule("episodic_memory", 1);
        let sample = classifier_edge_rules_with_morphology(
            std::slice::from_ref(&encode),
            "classifier_sample",
            1,
        );
        assert_eq!(sample.len(), 1);
        assert_eq!(sample[0]["morphology_id"], "classifier_sample");
    }

    #[test]
    fn geometry_flags_read_scan_twin_and_classifier_role() {
        use super::{geometry_classifier_role, geometry_scan_twin_flag};
        use std::collections::HashMap;

        let mut properties = HashMap::new();
        assert!(!geometry_scan_twin_flag(&properties));
        assert_eq!(
            geometry_classifier_role(&properties),
            serde_json::Value::Null
        );

        properties.insert("scan_twin".to_string(), serde_json::json!(true));
        properties.insert(
            "classifier_role".to_string(),
            serde_json::json!("kernel_memory"),
        );
        assert!(geometry_scan_twin_flag(&properties));
        assert_eq!(
            geometry_classifier_role(&properties),
            serde_json::json!("kernel_memory")
        );
    }

    #[test]
    fn classifier_memory_areas_are_created_hidden() {
        use super::classifier_memory_params;
        use std::collections::HashMap;

        let params = classifier_memory_params(
            "memid001".to_string(),
            "demo_kernel_mem".to_string(),
            (0, 0, 0),
            "region",
            HashMap::new(),
        );
        assert_eq!(params.visible, Some(false));
        assert_eq!(params.name, "demo_kernel_mem");
    }

    #[test]
    fn classifier_output_shape_copies_the_class_input() {
        use super::classifier_output_dimensions;
        assert_eq!(
            classifier_output_dimensions((12, 8, 4)).unwrap(),
            (12, 8, 4)
        );
        assert!(classifier_output_dimensions((12, 0, 4)).is_err());
    }

    #[test]
    fn classifier_class_output_does_not_forward_potential() {
        use super::classifier_scan_twin_params;
        let params = classifier_scan_twin_params(
            "twin0001".to_string(),
            "demo class output".to_string(),
            (12, 8, 4),
            (0, 0, 0),
            "region",
            "field",
            "kmem",
        );
        assert_eq!(
            params
                .properties
                .as_ref()
                .and_then(|properties| properties.get("mp_driven_psp")),
            Some(&serde_json::json!(false))
        );
    }

    #[test]
    fn classifier_enable_burst_writes_burst_engine_active() {
        use super::classifier_enable_burst_changes;
        assert_eq!(
            classifier_enable_burst_changes().get("burst_engine_active"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn classifier_twin_sits_above_the_classifier() {
        use super::classifier_twin_position;
        assert_eq!(
            classifier_twin_position((-29, 57, 10), (10, 6, 6), 0),
            (-29, 63, 10)
        );
    }

    #[test]
    fn classifier_twins_offset_along_x() {
        use super::classifier_twin_position;
        assert_eq!(
            classifier_twin_position((-29, 57, 10), (10, 6, 6), 10),
            (-19, 63, 10)
        );
    }

    #[test]
    fn classifier_scan_twin_is_custom_visible_and_flagged() {
        use super::classifier_scan_twin_params;

        let params = classifier_scan_twin_params(
            "ctwin001".to_string(),
            "demo class output".to_string(),
            (1, 1, 10),
            (10, 20, 30),
            "region",
            "field001",
            "memid001",
        );
        assert_eq!(params.name, "demo class output");
        assert_eq!(params.area_type, "Custom");
        assert_eq!(params.visible, Some(true));
        assert_eq!(params.dimensions, (1, 1, 10));
        assert_eq!(params.position, (10, 20, 30));
        let properties = params.properties.expect("class output properties");
        assert!(!properties.contains_key("scan_twin"));
        assert_eq!(
            properties.get("burst_engine_active"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            properties.get("classifier_role"),
            Some(&serde_json::json!("class_output"))
        );
        assert_eq!(
            properties.get("mp_driven_psp"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(params.leak_coefficient, Some(1.0));
    }
}

#[cfg(test)]
mod memory_parameters_response_tests {
    use super::memory_parameters_from_reported_area;
    use feagi_evolutionary::extract_memory_properties;
    use feagi_services::types::CorticalAreaInfo;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn reported_lifecycle_wins_over_stripped_properties_bag() {
        let mut properties = HashMap::new();
        properties.insert("is_mem_type".to_string(), json!(true));
        properties.insert("temporal_depth".to_string(), json!(4));
        let extracted = extract_memory_properties(&properties).expect("memory area");
        assert_eq!(extracted.init_lifespan, 9);
        assert_eq!(extracted.longterm_threshold, 100);

        let area: CorticalAreaInfo = serde_json::from_value(json!({
            "cortical_id": "bU1OSVNUXx8=",
            "cortical_id_s": "mMNIST__",
            "cortical_idx": 13,
            "cortical_name": "MNIST classifier_kernel_mem",
            "cortical_dimensions": [1, 1, 1],
            "coordinates_3d": [0, 0, 0],
            "area_type": "Memory",
            "cortical_group": "MEMORY",
            "cortical_type": "memory",
            "neuron_count": 0,
            "synapse_count": 0,
            "incoming_synapse_count": 0,
            "outgoing_synapse_count": 0,
            "visible": false,
            "neurons_per_voxel": 1,
            "neuron_post_synaptic_potential": 0.0,
            "neuron_post_synaptic_potential_max": 0.0,
            "neuron_plasticity_constant": 0.0,
            "neuron_degeneracy_coefficient": 0.0,
            "neuron_psp_uniform_distribution": false,
            "neuron_mp_driven_psp": false,
            "neuron_fire_threshold": 0.1,
            "neuron_fire_threshold_increment": [0.0, 0.0, 0.0],
            "neuron_firing_threshold_limit": 0.0,
            "neuron_consecutive_fire_count": 0,
            "neuron_snooze_period": 0,
            "neuron_refractory_period": 0,
            "neuron_leak_coefficient": 0.0,
            "neuron_leak_variability": 0.0,
            "neuron_mp_charge_accumulation": false,
            "neuron_excitability": 1.0,
            "neuron_burst_engine_active": true,
            "neuron_init_lifespan": 1,
            "neuron_lifespan_growth_rate": 2.0,
            "neuron_longterm_mem_threshold": 3,
            "temporal_depth": 4,
            "mp_learning_enabled": false,
            "properties": {"is_mem_type": true, "temporal_depth": 4}
        }))
        .expect("area record");

        let reported = memory_parameters_from_reported_area(&area, &extracted);
        assert_eq!(reported.init_lifespan, 1);
        assert_eq!(reported.longterm_mem_threshold, 3);
        assert_eq!(reported.lifespan_growth_rate, 2.0);
        assert_eq!(reported.temporal_depth, 4);
    }
}

#[cfg(test)]
mod cortical_clone_property_tests {
    use super::properties_for_cortical_clone;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn clone_property_map_keeps_neural_parameters_and_rewrites_placement() {
        let mut raw = HashMap::new();
        raw.insert("cortical_id".to_string(), json!("c291cmNl"));
        raw.insert("name".to_string(), json!("source"));
        raw.insert(
            "dimensions".to_string(),
            json!({"width": 2, "height": 2, "depth": 1}),
        );
        raw.insert("position".to_string(), json!([1, 2, 3]));
        raw.insert(
            "cortical_mapping_dst".to_string(),
            json!({"other": [{"morphology_id": "block_to_block"}]}),
        );
        raw.insert("firing_threshold".to_string(), json!(2.5));
        raw.insert("firing_threshold_increment_x".to_string(), json!(0.1));
        raw.insert("firing_threshold_increment_y".to_string(), json!(0.2));
        raw.insert("firing_threshold_increment_z".to_string(), json!(0.3));
        raw.insert("postsynaptic_current_max".to_string(), json!(42.0));
        raw.insert("mp_driven_psp".to_string(), json!(true));
        raw.insert("neuron_excitability".to_string(), json!(7.0));
        raw.insert("init_lifespan".to_string(), json!(9));
        raw.insert("synapse_attractivity".to_string(), json!(77.0));
        raw.insert("2d_coordinate".to_string(), json!([10, 20]));
        raw.insert("relative_coordinate".to_string(), json!([1, 2, 3]));
        raw.insert("temporal_depth".to_string(), json!(6));

        let cloned = properties_for_cortical_clone(raw, "region-1", [40, 50], [8, 9, 10]);

        assert!(!cloned.contains_key("cortical_id"));
        assert!(!cloned.contains_key("name"));
        assert!(!cloned.contains_key("cortical_mapping_dst"));
        assert_eq!(cloned.get("firing_threshold"), Some(&json!(2.5)));
        assert_eq!(
            cloned.get("firing_threshold_increment_x"),
            Some(&json!(0.1))
        );
        assert_eq!(
            cloned.get("firing_threshold_increment_y"),
            Some(&json!(0.2))
        );
        assert_eq!(
            cloned.get("firing_threshold_increment_z"),
            Some(&json!(0.3))
        );
        assert_eq!(cloned.get("postsynaptic_current_max"), Some(&json!(42.0)));
        assert_eq!(cloned.get("mp_driven_psp"), Some(&json!(true)));
        assert_eq!(cloned.get("neuron_excitability"), Some(&json!(7.0)));
        assert_eq!(cloned.get("init_lifespan"), Some(&json!(9)));
        assert_eq!(cloned.get("synapse_attractivity"), Some(&json!(77.0)));
        assert_eq!(cloned.get("temporal_depth"), Some(&json!(6)));
        assert_eq!(
            cloned.get("parent_region_id").and_then(|v| v.as_str()),
            Some("region-1")
        );
        assert_eq!(cloned.get("coordinate_2d"), Some(&json!([40, 50])));
        assert_eq!(cloned.get("2d_coordinate"), Some(&json!([40, 50])));
        assert_eq!(cloned.get("relative_coordinate"), Some(&json!([8, 9, 10])));
    }
}

#[cfg(test)]
mod classifier_area_id_tests {
    use super::classifier_owned_area_id;
    use feagi_structures::genomic::cortical_area::{CorticalAreaType, CorticalID};

    #[test]
    fn error_area_id_parses_as_a_custom_area() {
        let sample = classifier_owned_area_id(b'm', "C1", 10, 1).expect("sample id");
        let class_memory = classifier_owned_area_id(b'm', "C1", 10, 2).expect("class memory id");
        let error = classifier_owned_area_id(b'c', "C1", 10, 3).expect("error id");
        assert_ne!(sample, class_memory);
        assert_ne!(error, sample);
        let error_id = CorticalID::try_from_base_64(&error).expect("error id parses");
        assert!(matches!(
            error_id.as_cortical_type().expect("error type"),
            CorticalAreaType::Custom(_)
        ));
        assert!(classifier_owned_area_id(b'e', "C1", 10, 3).is_err());
    }
}

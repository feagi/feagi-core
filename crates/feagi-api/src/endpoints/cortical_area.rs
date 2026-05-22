// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Cortical Area API Endpoints - Exact port from Python `/v1/cortical_area/*`
//!
//! Reference: feagi-py/feagi/api/v1/cortical_area.py

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, Query, State};
use feagi_evolutionary::extract_memory_properties;
use feagi_genome_definitions::::descriptors::CorticalSubUnitIndex;
use feagi_genome_definitions::::CorticalID;
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

/// Genome memory parameters for the cortical area (from `extract_memory_properties`).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MemoryCorticalAreaParamsResponse {
    pub temporal_depth: u32,
    pub longterm_mem_threshold: u32,
    pub lifespan_growth_rate: f32,
    pub init_lifespan: u32,
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

    let mut group_id = request
        .get("group_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u8;

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

    let unit_id: Option<u8> = request
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
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() == 3 {
                Some((
                    arr[0].as_u64()? as usize,
                    arr[1].as_u64()? as usize,
                    arr[2].as_u64()? as usize,
                ))
            } else {
                None
            }
        });

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

    // Determine number of units and get topology
    let (num_units, unit_topology) = if cortical_type_str == "IPU" {
        // Find the matching sensory cortical unit
        let unit = SensoryCorticalUnit::list_all()
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

        // Split per-subunit data_type_config into two bytes for cortical ID
        let config_byte_4 = (data_type_config & 0xFF) as u8; // Lower byte
        let config_byte_5 = ((data_type_config >> 8) & 0xFF) as u8; // Upper byte

        // Get per-device dimensions from topology, then scale X by device_count:
        // total_x = device_count * per_device_x
        // If per_device_dimensions_override is provided, use it instead of topology defaults
        let (per_device_dimensions, dimensions) = if let Some(override_dims) =
            per_device_dimensions_override
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

        // Construct the 8-byte cortical ID
        let cortical_id_bytes = [
            if cortical_type_str == "IPU" {
                b'i'
            } else {
                b'o'
            }, // Byte 0: type
            subtype_bytes[0], // Byte 1: subtype[0]
            subtype_bytes[1], // Byte 2: subtype[1]
            subtype_bytes[2], // Byte 3: subtype[2]
            config_byte_4,    // Byte 4: data type config (lower byte)
            config_byte_5,    // Byte 5: data type config (upper byte)
            unit_idx as u8,   // Byte 6: unit index
            group_id,         // Byte 7: group ID
        ];

        // Encode to base64 for use as cortical_id string
        let cortical_id = general_purpose::STANDARD.encode(cortical_id_bytes);

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

        let params = CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: format!("{} Unit {}", cortical_type_key, unit_idx),
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

    tracing::info!(
        "Calling GenomeService to create {} cortical areas",
        creation_params.len()
    );

    // ARCHITECTURE: Call genome_service.create_cortical_areas (proper flow)
    // This will: 1) Update runtime genome, 2) Call neuroembryogenesis, 3) Create neurons/synapses
    let areas_details = genome_service
        .create_cortical_areas(creation_params)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create cortical areas: {}", e)))?;

    tracing::info!(
        "✅ Successfully created {} cortical areas via GenomeService",
        areas_details.len()
    );

    // Serialize as JSON
    let areas_json = serde_json::to_value(&areas_details).unwrap_or_default();

    // Extract cortical IDs from created areas
    let created_ids: Vec<String> = areas_details
        .iter()
        .map(|a| a.cortical_id.clone())
        .collect();

    // Return comprehensive response
    let first_id = created_ids.first().cloned().unwrap_or_default();
    let mut response = serde_json::Map::new();
    response.insert(
        "message".to_string(),
        serde_json::Value::String(format!("Created {} cortical areas", created_ids.len())),
    );
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

/// Clone an existing cortical area with all its properties and structure. (Not yet implemented)
#[utoipa::path(post, path = "/v1/cortical_area/clone", tag = "cortical_area")]
pub async fn post_clone(
    State(state): State<ApiState>,
    Json(request): Json<CloneCorticalAreaRequest>,
) -> ApiResult<Json<HashMap<String, String>>> {
    use base64::{engine::general_purpose, Engine as _};
    use feagi_services::types::CreateCorticalAreaParams;
    use feagi_genome_definitions::::CorticalID;
    use serde_json::Value;
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

    // Extract outgoing mappings (we will apply them after creation, via update_cortical_mapping).
    let outgoing_mapping_dst = source_area
        .properties
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

    // Clone properties, but do NOT carry over cortical mapping properties directly.
    // Mappings must be created via update_cortical_mapping so synapses are regenerated.
    let mut cloned_properties = source_area.properties.clone();
    cloned_properties.remove("cortical_mapping_dst");

    // Set parent region + 2D coordinate explicitly for the clone.
    cloned_properties.insert(
        "parent_region_id".to_string(),
        Value::String(source_parent_region_id),
    );
    cloned_properties.insert(
        "coordinate_2d".to_string(),
        serde_json::json!([request.coordinates_2d[0], request.coordinates_2d[1]]),
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
        memory_parameters: MemoryCorticalAreaParamsResponse {
            temporal_depth: mem_props.temporal_depth,
            longterm_mem_threshold: mem_props.longterm_threshold,
            lifespan_growth_rate: mem_props.lifespan_growth_rate,
            init_lifespan: mem_props.init_lifespan,
        },
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
            || snake_name == "miscellaneous"
        {
            vec![]
        } else {
            vec!["linear".to_string(), "fractional".to_string()]
        };

        // Default resolution based on type
        let resolution = if snake_name == "vision" {
            vec![64, 64, 1] // Vision sensors typically 64x64
        } else if snake_name == "segmented_vision" {
            vec![32, 32, 1] // Segmented vision segments are smaller
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

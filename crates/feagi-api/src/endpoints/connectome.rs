// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Connectome API Endpoints - Exact port from Python `/v1/connectome/*`

// Removed - using crate::common::State instead
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Path, Query, State};
use std::collections::HashMap;

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
            let detailed: HashMap<String, serde_json::Value> = areas
                .into_iter()
                .map(|area| {
                    (
                        area.cortical_id.clone(),
                        serde_json::to_value(area).unwrap_or_default(),
                    )
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

    debug!(target: "feagi-api", "Getting synapses for area {} (idx={}): {} neurons", area_id, cortical_idx, neurons.len());

    // Collect all outgoing synapses from neurons in this area
    // Access NPU through ConnectomeManager singleton
    let manager = feagi_brain_development::ConnectomeManager::instance();
    let manager_lock = manager.read();
    let npu_arc = manager_lock
        .get_npu()
        .ok_or_else(|| ApiError::internal("NPU not initialized"))?;
    let npu_lock = npu_arc.lock().unwrap();

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
    State(_state): State<ApiState>,
    Path(_neuron_id): Path<u64>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
}

/// GET /v1/connectome/neuron_properties
#[utoipa::path(get, path = "/v1/connectome/neuron_properties", tag = "connectome")]
pub async fn get_neuron_properties_query(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
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
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, bool>>> {
    let mut response = HashMap::new();
    response.insert("enabled".to_string(), true);
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
#[utoipa::path(get, path = "/v1/connectome/download", tag = "connectome")]
pub async fn get_download_connectome(
    State(state): State<ApiState>,
) -> ApiResult<Json<serde_json::Value>> {
    // Export connectome via service layer (architecture-compliant)
    let snapshot = state
        .connectome_service
        .export_connectome()
        .await
        .map_err(|e| ApiError::from(e))?;

    // Serialize snapshot to JSON
    let json_value = serde_json::to_value(&snapshot)
        .map_err(|e| ApiError::internal(format!("Failed to serialize connectome: {}", e)))?;

    Ok(Json(json_value))
}

/// GET /v1/connectome/download-cortical-area/{cortical_area}
#[utoipa::path(
    get,
    path = "/v1/connectome/download-cortical-area/{cortical_area}",
    tag = "connectome"
)]
pub async fn get_download_cortical_area(
    State(_state): State<ApiState>,
    Path(_area): Path<String>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
}

/// POST /v1/connectome/upload
#[utoipa::path(post, path = "/v1/connectome/upload", tag = "connectome")]
pub async fn post_upload_connectome(
    State(state): State<ApiState>,
    Json(data): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // Deserialize snapshot from JSON
    let snapshot: feagi_npu_neural::types::connectome::ConnectomeSnapshot =
        serde_json::from_value(data).map_err(|e| {
            ApiError::invalid_input(format!("Invalid connectome snapshot format: {}", e))
        })?;

    // Import connectome via service layer (architecture-compliant)
    state
        .connectome_service
        .import_connectome(snapshot)
        .await
        .map_err(|e| ApiError::from(e))?;

    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Connectome imported successfully".to_string(),
    )])))
}

/// POST /v1/connectome/upload-cortical-area
#[utoipa::path(post, path = "/v1/connectome/upload-cortical-area", tag = "connectome")]
pub async fn post_upload_cortical_area(
    State(_state): State<ApiState>,
    Json(_data): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Upload not yet implemented".to_string(),
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
                if subtype.len() > 0 {
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
        use feagi_data_structures::genomic::cortical_area::CorticalID;
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

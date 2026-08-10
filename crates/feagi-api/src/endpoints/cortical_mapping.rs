// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Cortical Mapping API Endpoints - Exact port from Python `/v1/cortical_mapping/*`

// Removed - using crate::common::State instead
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Query, State};
use std::collections::HashMap;

/// LTP/LTD multipliers are stored in the NPU as `i8` (aligns with effective range after
/// `u8` clamping). Accept JSON integers or float-shaped whole numbers, then range-check.
fn i8_ltd_ltp_from_json_value(v: &serde_json::Value, field: &str) -> Result<i8, ApiError> {
    let n = v
        .as_i64()
        .or_else(|| v.as_f64().map(|f| f as i64))
        .ok_or_else(|| ApiError::invalid_input(format!("{field} must be a number (integer-like)")))?;
    i8::try_from(n).map_err(|_| ApiError::invalid_input(format!("{field} must be in i8 range {}..={} (got {})", i8::MIN, i8::MAX, n)))
}

/// POST /v1/cortical_mapping/afferents
#[utoipa::path(post, path = "/v1/cortical_mapping/afferents", tag = "cortical_mapping")]
pub async fn post_afferents(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<Vec<String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/cortical_mapping/efferents
#[utoipa::path(post, path = "/v1/cortical_mapping/efferents", tag = "cortical_mapping")]
pub async fn post_efferents(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<Vec<String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/cortical_mapping/mapping_properties
#[utoipa::path(
    post,
    path = "/v1/cortical_mapping/mapping_properties",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "Cortical mapping connections", body = Vec<serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_mapping_properties(
    State(state): State<ApiState>,
    Json(req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    use tracing::debug;

    let src_area = req
        .get("src_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing src_cortical_area"))?;

    let dst_area = req
        .get("dst_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing dst_cortical_area"))?;

    debug!(target: "feagi-api", "Getting mapping properties: {} -> {}", src_area, dst_area);

    let connectome_service = state.connectome_service.as_ref();

    // Get source cortical_area area
    let src_area_info = connectome_service
        .get_cortical_area(src_area)
        .await
        .map_err(|e| ApiError::not_found("Cortical area", &format!("Source area {}: {}", src_area, e)))?;

    // Look for cortical_mapping_dst in properties
    let mapping_dst = src_area_info.properties.get("cortical_mapping_dst").and_then(|v| v.as_object());

    if mapping_dst.is_none() {
        debug!(target: "feagi-api", "No cortical_mapping_dst found for {}", src_area);
        return Ok(Json(vec![]));
    }

    // Get connections for this destination
    let connections = mapping_dst.unwrap().get(dst_area).and_then(|v| v.as_array());

    if connections.is_none() {
        debug!(target: "feagi-api", "No connections found from {} to {}", src_area, dst_area);
        return Ok(Json(vec![]));
    }

    // Normalize connections to expected format
    let mut formatted = Vec::new();
    for conn in connections.unwrap() {
        if let Some(arr) = conn.as_array() {
            // Array format:
            // [morphology_id, morphology_scalar, psc_multiplier, plasticity_flag,
            //  plasticity_constant, ltp_multiplier, ltd_multiplier, plasticity_window,
            //  synaptic_delay_bursts]
            if arr.len() < 8 {
                return Err(ApiError::invalid_input(format!(
                    "Invalid dstmap rule array (expected 8 elements including plasticity_window), got {}: {:?}",
                    arr.len(),
                    arr
                )));
            }
            // Strict parsing (no implicit defaults).
            let morphology_id = arr[0].as_str().ok_or_else(|| ApiError::invalid_input("morphology_id must be a string"))?;
            let morphology_scalar = arr[1].clone();
            let psc_multiplier = arr[2]
                .as_i64()
                .ok_or_else(|| ApiError::invalid_input("postSynapticCurrent_multiplier must be an integer"))?;
            let plasticity_flag = arr[3]
                .as_bool()
                .ok_or_else(|| ApiError::invalid_input("plasticity_flag must be a boolean"))?;
            let plasticity_constant = arr[4]
                .as_i64()
                .ok_or_else(|| ApiError::invalid_input("plasticity_constant must be an integer"))?;
            let ltp_multiplier = i8_ltd_ltp_from_json_value(&arr[5], "ltp_multiplier")?;
            let ltd_multiplier = i8_ltd_ltp_from_json_value(&arr[6], "ltd_multiplier")?;
            let plasticity_window = arr[7]
                .as_i64()
                .ok_or_else(|| ApiError::invalid_input("plasticity_window must be an integer"))?;

            let synaptic_delay_bursts: u64 = if arr.len() >= 9 {
                arr[8]
                    .as_u64()
                    .or_else(|| arr[8].as_i64().map(|i| i as u64))
                    .ok_or_else(|| ApiError::invalid_input("synaptic_delay_bursts must be a non-negative integer"))?
            } else {
                1
            }
            .max(1);

            formatted.push(serde_json::json!({
                "morphology_id": morphology_id,
                "morphology_scalar": morphology_scalar,
                "postSynapticCurrent_multiplier": psc_multiplier,
                "plasticity_flag": plasticity_flag,
                "plasticity_constant": plasticity_constant,
                "ltp_multiplier": ltp_multiplier,
                "ltd_multiplier": ltd_multiplier,
                "plasticity_window": plasticity_window,
                "synaptic_delay_bursts": synaptic_delay_bursts,
            }));
        } else if let Some(obj) = conn.as_object() {
            // Dict format - strict schema (no implicit defaults)
            let morphology_id = obj
                .get("morphology_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::invalid_input("morphology_id must be a string"))?;
            let morphology_scalar = obj
                .get("morphology_scalar")
                .cloned()
                .ok_or_else(|| ApiError::invalid_input("morphology_scalar missing"))?;
            let psc_multiplier = obj
                .get("postSynapticCurrent_multiplier")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ApiError::invalid_input("postSynapticCurrent_multiplier must be an integer"))?;
            let plasticity_flag = obj
                .get("plasticity_flag")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| ApiError::invalid_input("plasticity_flag must be a boolean"))?;
            let plasticity_constant = obj
                .get("plasticity_constant")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ApiError::invalid_input("plasticity_constant must be an integer"))?;
            let ltp_multiplier = i8_ltd_ltp_from_json_value(
                obj.get("ltp_multiplier")
                    .ok_or_else(|| ApiError::invalid_input("ltp_multiplier missing"))?,
                "ltp_multiplier",
            )?;
            let ltd_multiplier = i8_ltd_ltp_from_json_value(
                obj.get("ltd_multiplier")
                    .ok_or_else(|| ApiError::invalid_input("ltd_multiplier missing"))?,
                "ltd_multiplier",
            )?;
            let plasticity_window = obj
                .get("plasticity_window")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ApiError::invalid_input("plasticity_window must be an integer"))?;

            let synaptic_delay_bursts: u64 = obj
                .get("synaptic_delay_bursts")
                .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                .unwrap_or(1)
                .max(1);

            // R-STDP optional fields. plasticity_mode is the canonical successor of
            // plasticity_flag; when absent, downstream defaults to Stdp / Off based on the flag.
            // The other three fields are only meaningful when plasticity_mode == "rstdp" and are
            // validated downstream by the BDU.
            let plasticity_mode = obj.get("plasticity_mode").and_then(|v| v.as_str()).map(str::to_string);
            let eligibility_decay_bursts = obj.get("eligibility_decay_bursts").and_then(|v| v.as_u64());
            let reward_source_area = obj.get("reward_source_area").and_then(|v| v.as_str()).map(str::to_string);
            let punishment_source_area = obj.get("punishment_source_area").and_then(|v| v.as_str()).map(str::to_string);
            // Optional R-STDP / STDP weight ceiling. Validated downstream by the BDU; we only
            // shape it here so the rule round-trips cleanly through GET.
            let max_weight = obj.get("max_weight").and_then(|v| if v.is_null() { None } else { v.as_f64() });
            let plasticity_eta = obj.get("plasticity_eta").and_then(|v| if v.is_null() { None } else { v.as_f64() });

            let mut rule = serde_json::json!({
                "morphology_id": morphology_id,
                "morphology_scalar": morphology_scalar,
                "postSynapticCurrent_multiplier": psc_multiplier,
                "plasticity_flag": plasticity_flag,
                "plasticity_constant": plasticity_constant,
                "ltp_multiplier": ltp_multiplier,
                "ltd_multiplier": ltd_multiplier,
                "plasticity_window": plasticity_window,
                "synaptic_delay_bursts": synaptic_delay_bursts,
            });
            let rule_obj = rule.as_object_mut().unwrap();
            if let Some(mode) = plasticity_mode {
                rule_obj.insert("plasticity_mode".to_string(), serde_json::json!(mode));
            }
            if let Some(decay) = eligibility_decay_bursts {
                rule_obj.insert("eligibility_decay_bursts".to_string(), serde_json::json!(decay));
            }
            if let Some(area) = reward_source_area {
                rule_obj.insert("reward_source_area".to_string(), serde_json::json!(area));
            }
            if let Some(area) = punishment_source_area {
                rule_obj.insert("punishment_source_area".to_string(), serde_json::json!(area));
            }
            if let Some(mw) = max_weight {
                rule_obj.insert("max_weight".to_string(), serde_json::json!(mw));
            }
            if let Some(eta) = plasticity_eta {
                rule_obj.insert("plasticity_eta".to_string(), serde_json::json!(eta));
            }
            formatted.push(rule);
        }
    }

    debug!(target: "feagi-api", "Returning {} mapping connections from {} to {}", formatted.len(), src_area, dst_area);
    Ok(Json(formatted))
}

/// PUT /v1/cortical_mapping/mapping_properties
#[utoipa::path(
    put,
    path = "/v1/cortical_mapping/mapping_properties",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "Cortical mapping updated successfully", body = HashMap<String, serde_json::Value>),
        (status = 404, description = "Cortical area not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_mapping_properties(
    State(state): State<ApiState>,
    Json(req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::{debug, info};

    let src_area = req
        .get("src_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing src_cortical_area"))?;

    let dst_area = req
        .get("dst_cortical_area")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing dst_cortical_area"))?;

    let mapping_string = req
        .get("mapping_string")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ApiError::invalid_input("Missing mapping_string"))?;

    info!(
        target: "feagi-api",
        "PUT cortical_area mapping: {} -> {} with {} connections",
        src_area,
        dst_area,
        mapping_string.len()
    );
    debug!(target: "feagi-api", "Mapping data: {:?}", mapping_string);

    let connectome_service = state.connectome_service.as_ref();

    // Update the cortical_area mapping (this modifies ConnectomeManager and regenerates synapses)
    let synapse_count = connectome_service
        .update_cortical_mapping(src_area.to_string(), dst_area.to_string(), mapping_string.clone())
        .await
        .map_err(|e| match e {
            feagi_services::types::ServiceError::InvalidInput(msg) => ApiError::invalid_input(msg),
            feagi_services::types::ServiceError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Failed to update cortical_area mapping: {}", e)),
        })?;

    info!(target: "feagi-api", "Cortical mapping updated successfully: {} synapses created", synapse_count);

    // Return success response matching Python format
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!(format!(
            "Cortical mapping properties updated successfully from {} to {}",
            src_area, dst_area
        )),
    );
    response.insert("synapse_count".to_string(), serde_json::json!(synapse_count));
    response.insert("src_region".to_string(), serde_json::json!(null)); // TODO: Add region context
    response.insert("dst_region".to_string(), serde_json::json!(null)); // TODO: Add region context

    Ok(Json(response))
}

/// GET /v1/cortical_mapping/mapping
/// Get specific cortical_area mapping between two areas
#[utoipa::path(
    get,
    path = "/v1/cortical_mapping/mapping",
    tag = "cortical_mapping",
    params(
        ("src_cortical_area" = String, Query, description = "Source cortical_area area ID"),
        ("dst_cortical_area" = String, Query, description = "Destination cortical_area area ID")
    ),
    responses(
        (status = 200, description = "Mapping properties", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_mapping(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let src_area = params
        .get("src_cortical_area")
        .ok_or_else(|| ApiError::invalid_input("src_cortical_area required"))?;
    let dst_area = params
        .get("dst_cortical_area")
        .ok_or_else(|| ApiError::invalid_input("dst_cortical_area required"))?;

    // Get mapping properties directly (avoid recursion)
    let connectome_service = state.connectome_service.as_ref();

    // Get source cortical_area area
    let src_area_info = connectome_service
        .get_cortical_area(src_area)
        .await
        .map_err(|e| ApiError::not_found("Cortical area", &format!("Source area {}: {}", src_area, e)))?;

    // Look for cortical_mapping_dst in properties
    let mapping_dst = src_area_info.properties.get("cortical_mapping_dst").and_then(|v| v.as_object());

    if mapping_dst.is_none() {
        return Ok(Json(HashMap::new()));
    }

    // Get connections for this destination
    let connections = mapping_dst.unwrap().get(dst_area).and_then(|v| v.as_array());

    let mut response = HashMap::new();
    response.insert("connections".to_string(), serde_json::json!(connections.unwrap_or(&vec![])));

    Ok(Json(response))
}

/// GET /v1/cortical_mapping/mapping_list
/// Get list of all cortical_area mappings
#[utoipa::path(
    get,
    path = "/v1/cortical_mapping/mapping_list",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "List of all mappings", body = Vec<Vec<String>>)
    )
)]
pub async fn get_mapping_list(State(state): State<ApiState>) -> ApiResult<Json<Vec<Vec<String>>>> {
    let connectome_service = state.connectome_service.as_ref();

    let areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list areas: {}", e)))?;

    let mut mappings = Vec::new();

    // Scan all cortical_mapping_dst properties
    for area in &areas {
        if let Ok(area_detail) = connectome_service.get_cortical_area(&area.cortical_id).await {
            if let Some(mapping_dst) = area_detail.properties.get("cortical_mapping_dst") {
                if let Some(dst_map) = mapping_dst.as_object() {
                    for dst_area_id in dst_map.keys() {
                        mappings.push(vec![area.cortical_id.clone(), dst_area_id.clone()]);
                    }
                }
            }
        }
    }

    Ok(Json(mappings))
}

/// DELETE /v1/cortical_mapping/mapping
///
/// Delete the cortical_area mapping between two areas. This clears the rule data on the source
/// area, prunes all synapses from `src_cortical_area` to `dst_cortical_area`, and persists
/// the change to the RuntimeGenome. Equivalent to `PUT /v1/cortical_mapping/mapping_properties`
/// with `mapping_string=[]`, but exposes a clean DELETE semantic for clients that need to
/// drop a mapping (and reset its learned synapse weights) without enumerating an empty
/// rules array.
///
/// Source/target IDs are accepted as query string parameters to match the existing
/// `GET /v1/cortical_mapping/mapping` route and avoid HTTP 415 surprises on bodyless DELETEs.
#[utoipa::path(
    delete,
    path = "/v1/cortical_mapping/mapping",
    tag = "cortical_mapping",
    params(
        ("src_cortical_area" = String, Query, description = "Source cortical_area area ID"),
        ("dst_cortical_area" = String, Query, description = "Destination cortical_area area ID")
    ),
    responses(
        (status = 200, description = "Mapping deleted", body = HashMap<String, serde_json::Value>),
        (status = 400, description = "Missing src_cortical_area or dst_cortical_area"),
        (status = 404, description = "Cortical area not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_mapping(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use tracing::info;

    let src_area = params
        .get("src_cortical_area")
        .ok_or_else(|| ApiError::invalid_input("src_cortical_area required"))?
        .to_string();
    let dst_area = params
        .get("dst_cortical_area")
        .ok_or_else(|| ApiError::invalid_input("dst_cortical_area required"))?
        .to_string();

    info!(
        target: "feagi-api",
        "DELETE cortical_area mapping: {} -> {}",
        src_area,
        dst_area
    );

    let connectome_service = state.connectome_service.as_ref();

    // Reuse the canonical update path with empty rules. `update_cortical_mapping` is the
    // single source of truth for: clearing the rule data, pruning synapses via the BDU
    // `regenerate_synapses_for_mapping` codepath, refreshing the burst runner cache, and
    // persisting to RuntimeGenome / region IO registry. Returning anything else here would
    // diverge from the PUT-with-empty-rules behaviour and risk drift.
    let synapse_count = connectome_service
        .update_cortical_mapping(src_area.clone(), dst_area.clone(), Vec::new())
        .await
        .map_err(|e| match e {
            feagi_services::types::ServiceError::InvalidInput(msg) => ApiError::invalid_input(msg),
            feagi_services::types::ServiceError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Failed to delete cortical_area mapping: {}", e)),
        })?;

    info!(
        target: "feagi-api",
        "Cortical mapping deleted: {} -> {} ({} synapses remaining)",
        src_area,
        dst_area,
        synapse_count
    );

    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!(format!("Cortical mapping deleted from {} to {}", src_area, dst_area)),
    );
    response.insert("src_cortical_area".to_string(), serde_json::json!(src_area));
    response.insert("dst_cortical_area".to_string(), serde_json::json!(dst_area));
    response.insert("synapse_count".to_string(), serde_json::json!(synapse_count));

    Ok(Json(response))
}

/// POST /v1/cortical_mapping/batch_update
/// Batch update multiple cortical_area mappings
#[utoipa::path(
    post,
    path = "/v1/cortical_mapping/batch_update",
    tag = "cortical_mapping",
    responses(
        (status = 200, description = "Batch update completed", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_batch_update(
    State(_state): State<ApiState>,
    Json(_request): Json<Vec<HashMap<String, String>>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement batch update
    let mut response = HashMap::new();
    response.insert("message".to_string(), serde_json::json!("Batch update not yet implemented"));
    response.insert("updated_count".to_string(), serde_json::json!(0));

    Ok(Json(response))
}

// EXACT Python paths:
/// POST /v1/cortical_mapping/mapping
#[utoipa::path(post, path = "/v1/cortical_mapping/mapping", tag = "cortical_mapping")]
pub async fn post_mapping(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// PUT /v1/cortical_mapping/mapping
#[utoipa::path(put, path = "/v1/cortical_mapping/mapping", tag = "cortical_mapping")]
pub async fn put_mapping(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

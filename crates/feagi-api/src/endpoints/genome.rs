// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Genome API Endpoints - Exact port from Python `/v1/genome/*`

// Removed - using crate::common::State instead
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Query, State};
use feagi_services::types::LoadGenomeParams;
use std::collections::HashMap;
use tracing::info;

/// Get the current genome file name.
#[utoipa::path(get, path = "/v1/genome/file_name", tag = "genome")]
pub async fn get_file_name(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Get current genome filename
    Ok(Json(HashMap::from([(
        "genome_file_name".to_string(),
        "".to_string(),
    )])))
}

/// Get list of available circuit templates from the circuit library.
#[utoipa::path(get, path = "/v1/genome/circuits", tag = "genome")]
pub async fn get_circuits(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    // TODO: Get available circuit library
    Ok(Json(vec![]))
}

/// Set the destination for genome amalgamation (merging genomes).
#[utoipa::path(post, path = "/v1/genome/amalgamation_destination", tag = "genome")]
pub async fn post_amalgamation_destination(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Cancel a pending genome amalgamation operation.
#[utoipa::path(delete, path = "/v1/genome/amalgamation_cancellation", tag = "genome")]
pub async fn delete_amalgamation_cancellation(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Append additional structures to the current genome.
#[utoipa::path(post, path = "/v1/feagi/genome/append", tag = "genome")]
pub async fn post_genome_append(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Load the minimal barebones genome with only essential neural structures.
#[utoipa::path(
    post,
    path = "/v1/genome/upload/barebones",
    responses(
        (status = 200, description = "Barebones genome loaded successfully"),
        (status = 500, description = "Failed to load genome")
    ),
    tag = "genome"
)]
pub async fn post_upload_barebones_genome(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    tracing::debug!(target: "feagi-api", "📥 POST /v1/genome/upload/barebones - Request received");
    let result = load_default_genome(state, "barebones").await;
    match &result {
        Ok(_) => {
            tracing::debug!(target: "feagi-api", "✅ POST /v1/genome/upload/barebones - Success")
        }
        Err(e) => {
            tracing::error!(target: "feagi-api", "❌ POST /v1/genome/upload/barebones - Error: {:?}", e)
        }
    }
    result
}

/// Load the essential genome with core sensory and motor areas.
#[utoipa::path(
    post,
    path = "/v1/genome/upload/essential",
    responses(
        (status = 200, description = "Essential genome loaded successfully"),
        (status = 500, description = "Failed to load genome")
    ),
    tag = "genome"
)]
pub async fn post_upload_essential_genome(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    load_default_genome(state, "essential").await
}

/// Helper function to load a default genome by name from embedded Rust genomes
async fn load_default_genome(
    state: ApiState,
    genome_name: &str,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    tracing::info!(target: "feagi-api", "🔄 Loading {} genome from embedded Rust genomes", genome_name);
    tracing::debug!(target: "feagi-api", "   State components available: genome_service=true, runtime_service=true");

    // Load genome from embedded Rust templates (no file I/O!)
    let genome_json = match genome_name {
        "barebones" => feagi_evolutionary::BAREBONES_GENOME_JSON,
        "essential" => feagi_evolutionary::ESSENTIAL_GENOME_JSON,
        "test" => feagi_evolutionary::TEST_GENOME_JSON,
        "vision" => feagi_evolutionary::VISION_GENOME_JSON,
        _ => {
            return Err(ApiError::invalid_input(format!(
                "Unknown genome name '{}'. Available: barebones, essential, test, vision",
                genome_name
            )))
        }
    };

    tracing::info!(target: "feagi-api","Using embedded {} genome ({} bytes), starting conversion...",
                   genome_name, genome_json.len());

    // Load genome via service (which will automatically ensure core components)
    let genome_service = state.genome_service.as_ref();
    let params = LoadGenomeParams {
        json_str: genome_json.to_string(),
    };

    tracing::info!(target: "feagi-api","Calling genome service load_genome...");
    let genome_info = genome_service
        .load_genome(params)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load genome: {}", e)))?;

    tracing::info!(target: "feagi-api","Successfully loaded {} genome: {} cortical areas, {} brain regions",
               genome_name, genome_info.cortical_area_count, genome_info.brain_region_count);

    // CRITICAL: Update burst frequency from genome's simulation_timestep
    // Genome specifies timestep in seconds, convert to Hz: frequency = 1 / timestep
    let burst_frequency_hz = 1.0 / genome_info.simulation_timestep;
    tracing::info!(target: "feagi-api","Updating burst frequency from genome: {} seconds timestep → {:.0} Hz",
                   genome_info.simulation_timestep, burst_frequency_hz);

    // Update runtime service with new frequency
    let runtime_service = state.runtime_service.as_ref();
    runtime_service
        .set_frequency(burst_frequency_hz)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update burst frequency: {}", e)))?;

    tracing::info!(target: "feagi-api","✅ Burst frequency updated to {:.0} Hz from genome physiology", burst_frequency_hz);

    // Return response matching Python format
    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::Value::Bool(true));
    response.insert(
        "message".to_string(),
        serde_json::Value::String(format!("{} genome loaded successfully", genome_name)),
    );
    response.insert(
        "cortical_area_count".to_string(),
        serde_json::Value::Number(genome_info.cortical_area_count.into()),
    );
    response.insert(
        "brain_region_count".to_string(),
        serde_json::Value::Number(genome_info.brain_region_count.into()),
    );
    response.insert(
        "genome_id".to_string(),
        serde_json::Value::String(genome_info.genome_id),
    );
    response.insert(
        "genome_title".to_string(),
        serde_json::Value::String(genome_info.genome_title),
    );

    Ok(Json(response))
}

/// Get the current genome name.
#[utoipa::path(
    get,
    path = "/v1/genome/name",
    tag = "genome",
    responses(
        (status = 200, description = "Genome name", body = String)
    )
)]
pub async fn get_name(State(_state): State<ApiState>) -> ApiResult<Json<String>> {
    // Get genome metadata to extract name
    // TODO: Implement proper genome name retrieval from genome service
    Ok(Json("default_genome".to_string()))
}

/// Get the genome creation or modification timestamp.
#[utoipa::path(
    get,
    path = "/v1/genome/timestamp",
    tag = "genome",
    responses(
        (status = 200, description = "Genome timestamp", body = i64)
    )
)]
pub async fn get_timestamp(State(_state): State<ApiState>) -> ApiResult<Json<i64>> {
    // TODO: Store and retrieve genome timestamp
    Ok(Json(0))
}

/// Save the current genome to a file with optional ID and title parameters.
#[utoipa::path(
    post,
    path = "/v1/genome/save",
    tag = "genome",
    responses(
        (status = 200, description = "Genome saved", body = HashMap<String, String>)
    )
)]
pub async fn post_save(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    use std::fs;
    use std::path::Path;

    info!("Saving genome to file");

    // Get parameters
    let genome_id = request.get("genome_id").cloned();
    let genome_title = request.get("genome_title").cloned();
    let file_path = request.get("file_path").cloned();

    // Create save parameters
    let params = feagi_services::SaveGenomeParams {
        genome_id,
        genome_title,
    };

    // Call genome service to generate JSON
    let genome_service = state.genome_service.as_ref();
    let genome_json = genome_service
        .save_genome(params)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to save genome: {}", e)))?;

    // Determine file path
    let save_path = if let Some(path) = file_path {
        path
    } else {
        // Default to genomes directory with timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("genomes/saved_genome_{}.json", timestamp)
    };

    // Ensure parent directory exists
    if let Some(parent) = Path::new(&save_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| ApiError::internal(format!("Failed to create directory: {}", e)))?;
    }

    // Write to file
    fs::write(&save_path, genome_json)
        .map_err(|e| ApiError::internal(format!("Failed to write file: {}", e)))?;

    info!("✅ Genome saved successfully to: {}", save_path);

    Ok(Json(HashMap::from([
        (
            "message".to_string(),
            "Genome saved successfully".to_string(),
        ),
        ("file_path".to_string(), save_path),
    ])))
}

/// Load a genome from a file by name.
#[utoipa::path(
    post,
    path = "/v1/genome/load",
    tag = "genome",
    responses(
        (status = 200, description = "Genome loaded", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_load(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let genome_name = request
        .get("genome_name")
        .ok_or_else(|| ApiError::invalid_input("genome_name required"))?;

    // Load genome from defaults
    let genome_service = state.genome_service.as_ref();
    let params = feagi_services::LoadGenomeParams {
        json_str: format!("{{\"genome_title\": \"{}\"}}", genome_name),
    };

    let genome_info = genome_service
        .load_genome(params)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load genome: {}", e)))?;

    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome loaded successfully"),
    );
    response.insert(
        "genome_title".to_string(),
        serde_json::json!(genome_info.genome_title),
    );

    Ok(Json(response))
}

/// Upload and load a genome from JSON payload.
#[utoipa::path(
    post,
    path = "/v1/genome/upload",
    tag = "genome",
    responses(
        (status = 200, description = "Genome uploaded", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_upload(
    State(state): State<ApiState>,
    Json(genome_json): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let genome_service = state.genome_service.as_ref();

    // Convert to JSON string
    let json_str = serde_json::to_string(&genome_json)
        .map_err(|e| ApiError::invalid_input(format!("Invalid JSON: {}", e)))?;

    let params = LoadGenomeParams { json_str };
    let genome_info = genome_service
        .load_genome(params)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to upload genome: {}", e)))?;

    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::json!(true));
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome uploaded successfully"),
    );
    response.insert(
        "cortical_area_count".to_string(),
        serde_json::json!(genome_info.cortical_area_count),
    );
    response.insert(
        "brain_region_count".to_string(),
        serde_json::json!(genome_info.brain_region_count),
    );

    Ok(Json(response))
}

/// Download the current genome as a JSON document.
#[utoipa::path(
    get,
    path = "/v1/genome/download",
    tag = "genome",
    responses(
        (status = 200, description = "Genome JSON", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_download(State(state): State<ApiState>) -> ApiResult<Json<serde_json::Value>> {
    info!("🦀 [API] GET /v1/genome/download - Downloading current genome");
    let genome_service = state.genome_service.as_ref();

    // Get genome as JSON string
    let genome_json_str = genome_service
        .save_genome(feagi_services::types::SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .map_err(|e| {
            tracing::error!("Failed to export genome: {}", e);
            ApiError::internal(format!("Failed to export genome: {}", e))
        })?;

    // Parse to Value for JSON response
    let genome_value: serde_json::Value = serde_json::from_str(&genome_json_str)
        .map_err(|e| ApiError::internal(format!("Failed to parse genome JSON: {}", e)))?;

    info!("✅ Genome download complete, {} bytes", genome_json_str.len());
    Ok(Json(genome_value))
}

/// Get genome properties including metadata, size, and configuration details.
#[utoipa::path(
    get,
    path = "/v1/genome/properties",
    tag = "genome",
    responses(
        (status = 200, description = "Genome properties", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_properties(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement proper metadata retrieval from genome service
    Ok(Json(HashMap::new()))
}

/// Validate a genome structure for correctness and completeness.
#[utoipa::path(
    post,
    path = "/v1/genome/validate",
    tag = "genome",
    responses(
        (status = 200, description = "Validation result", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_validate(
    State(_state): State<ApiState>,
    Json(_genome): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome validation
    let mut response = HashMap::new();
    response.insert("valid".to_string(), serde_json::json!(true));
    response.insert("errors".to_string(), serde_json::json!([]));
    response.insert("warnings".to_string(), serde_json::json!([]));

    Ok(Json(response))
}

/// Transform genome between different formats (flat to hierarchical or vice versa).
#[utoipa::path(
    post,
    path = "/v1/genome/transform",
    tag = "genome",
    responses(
        (status = 200, description = "Transformed genome", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_transform(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome transformation
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome transformation not yet implemented"),
    );

    Ok(Json(response))
}

/// Clone the current genome with a new name, creating an independent copy.
#[utoipa::path(
    post,
    path = "/v1/genome/clone",
    tag = "genome",
    responses(
        (status = 200, description = "Genome cloned", body = HashMap<String, String>)
    )
)]
pub async fn post_clone(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement genome cloning
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Genome cloning not yet implemented".to_string(),
    )])))
}

/// Reset genome to its default state, clearing all customizations.
#[utoipa::path(
    post,
    path = "/v1/genome/reset",
    tag = "genome",
    responses(
        (status = 200, description = "Genome reset", body = HashMap<String, String>)
    )
)]
pub async fn post_reset(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement genome reset
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Genome reset not yet implemented".to_string(),
    )])))
}

/// Get genome metadata (alternative endpoint to properties).
#[utoipa::path(
    get,
    path = "/v1/genome/metadata",
    tag = "genome",
    responses(
        (status = 200, description = "Genome metadata", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_metadata(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    get_properties(State(state)).await
}

/// Merge another genome into the current genome, combining their structures.
#[utoipa::path(
    post,
    path = "/v1/genome/merge",
    tag = "genome",
    responses(
        (status = 200, description = "Genome merged", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_merge(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome merging
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome merging not yet implemented"),
    );

    Ok(Json(response))
}

/// Get a diff comparison between two genomes showing their differences.
#[utoipa::path(
    get,
    path = "/v1/genome/diff",
    tag = "genome",
    params(
        ("genome_a" = String, Query, description = "First genome name"),
        ("genome_b" = String, Query, description = "Second genome name")
    ),
    responses(
        (status = 200, description = "Genome diff", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_diff(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome diffing
    let mut response = HashMap::new();
    response.insert("differences".to_string(), serde_json::json!([]));

    Ok(Json(response))
}

/// Export genome in a specific format (JSON, YAML, binary, etc.).
#[utoipa::path(
    post,
    path = "/v1/genome/export_format",
    tag = "genome",
    responses(
        (status = 200, description = "Exported genome", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_export_format(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement format-specific export
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Format export not yet implemented"),
    );

    Ok(Json(response))
}

// EXACT Python paths:
/// Get current amalgamation status and configuration.
#[utoipa::path(get, path = "/v1/genome/amalgamation", tag = "genome")]
pub async fn get_amalgamation(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
}

/// Get history of all genome amalgamation operations performed.
#[utoipa::path(get, path = "/v1/genome/amalgamation_history", tag = "genome")]
pub async fn get_amalgamation_history_exact(
    State(_state): State<ApiState>,
) -> ApiResult<Json<Vec<HashMap<String, serde_json::Value>>>> {
    Ok(Json(Vec::new()))
}

/// Get metadata about all available cortical types including supported encodings and configurations.
#[utoipa::path(get, path = "/v1/genome/cortical_template", tag = "genome")]
pub async fn get_cortical_template(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
    };
    use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
    use serde_json::json;

    let mut templates = HashMap::new();

    // Helper to convert data type to human-readable format.
    //
    // NOTE: This endpoint is designed for tool/UIs (e.g. BV) and must be
    // deterministic across platforms and runs. No fallbacks.
    let data_type_to_json = |dt: IOCorticalAreaConfigurationFlag| -> serde_json::Value {
        let (variant, frame, positioning) = match dt {
            IOCorticalAreaConfigurationFlag::Boolean => ("Boolean", FrameChangeHandling::Absolute, None),
            IOCorticalAreaConfigurationFlag::Percentage(f, p) => ("Percentage", f, Some(p)),
            IOCorticalAreaConfigurationFlag::Percentage2D(f, p) => ("Percentage2D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::Percentage3D(f, p) => ("Percentage3D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::Percentage4D(f, p) => ("Percentage4D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage(f, p) => ("SignedPercentage", f, Some(p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage2D(f, p) => ("SignedPercentage2D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage3D(f, p) => ("SignedPercentage3D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage4D(f, p) => ("SignedPercentage4D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::CartesianPlane(f) => ("CartesianPlane", f, None),
            IOCorticalAreaConfigurationFlag::Misc(f) => ("Misc", f, None),
        };

        let frame_str = match frame {
            FrameChangeHandling::Absolute => "Absolute",
            FrameChangeHandling::Incremental => "Incremental",
        };

        let positioning_str = positioning.map(|p| match p {
            PercentageNeuronPositioning::Linear => "Linear",
            PercentageNeuronPositioning::Fractional => "Fractional",
        });

        json!({
            "variant": variant,
            "frame_change_handling": frame_str,
            "percentage_positioning": positioning_str,
            "config_value": dt.to_data_type_configuration_flag()
        })
    };

    // Add motor types
    for motor_unit in MotorCorticalUnit::list_all() {
        let friendly_name = motor_unit.get_friendly_name();
        let cortical_id_ref = motor_unit.get_cortical_id_unit_reference();
        let num_areas = motor_unit.get_number_cortical_areas();
        let topology = motor_unit.get_unit_default_topology();

        // BREAKING CHANGE (unreleased API):
        // - Remove unit-level `supported_data_types`.
        // - Expose per-subunit metadata, because some units (e.g. Gaze) have heterogeneous subunits
        //   with different IOCorticalAreaConfigurationFlag variants (Percentage2D vs Percentage).
        //
        // We derive supported types by:
        // - generating canonical cortical IDs from the MotorCorticalUnit template for each
        //   (frame_change_handling, percentage_neuron_positioning) combination
        // - extracting the IO configuration flag from each cortical ID
        // - grouping supported_data_types per subunit index
        use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
        use serde_json::{Map, Value};
        use std::collections::HashMap as StdHashMap;

        let mut subunits: StdHashMap<String, serde_json::Value> = StdHashMap::new();

        // Initialize subunits with topology-derived properties.
        for (sub_idx, topo) in topology {
            subunits.insert(
                sub_idx.get().to_string(),
                json!({
                    "relative_position": topo.relative_position,
                    "channel_dimensions_default": topo.channel_dimensions_default,
                    "channel_dimensions_min": topo.channel_dimensions_min,
                    "channel_dimensions_max": topo.channel_dimensions_max,
                    "supported_data_types": Vec::<serde_json::Value>::new(),
                }),
            );
        }

        // Build per-subunit supported_data_types (deduped).
        let allowed_frames = motor_unit.get_allowed_frame_change_handling();
        let frames: Vec<FrameChangeHandling> = match allowed_frames {
            Some(allowed) => allowed.to_vec(),
            None => vec![FrameChangeHandling::Absolute, FrameChangeHandling::Incremental],
        };

        let positionings = [
            PercentageNeuronPositioning::Linear,
            PercentageNeuronPositioning::Fractional,
        ];

        let mut per_subunit_dedup: StdHashMap<String, std::collections::HashSet<String>> =
            StdHashMap::new();

        for frame in frames {
            for positioning in positionings {
                let mut map: Map<String, Value> = Map::new();
                map.insert(
                    "frame_change_handling".to_string(),
                    serde_json::to_value(frame).unwrap_or(Value::Null),
                );
                map.insert(
                    "percentage_neuron_positioning".to_string(),
                    serde_json::to_value(positioning).unwrap_or(Value::Null),
                );

                // Use unit index 0 for template enumeration (index does not affect IO flags).
                let cortical_ids = motor_unit
                    .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
                        CorticalUnitIndex::from(0u8),
                        map,
                    );

                if let Ok(ids) = cortical_ids {
                    for (i, id) in ids.into_iter().enumerate() {
                        if let Ok(flag) = id.extract_io_data_flag() {
                            let dt_json = data_type_to_json(flag);
                            let subunit_key = i.to_string();

                            let dedup_key = format!(
                                "{}|{}|{}",
                                dt_json.get("variant").and_then(|v| v.as_str()).unwrap_or(""),
                                dt_json
                                    .get("frame_change_handling")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(""),
                                dt_json
                                    .get("percentage_positioning")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                            );

                            let seen = per_subunit_dedup
                                .entry(subunit_key.clone())
                                .or_insert_with(std::collections::HashSet::new);
                            if !seen.insert(dedup_key) {
                                continue;
                            }

                            if let Some(subunit_obj) = subunits.get_mut(&subunit_key) {
                                if let Some(arr) = subunit_obj.get_mut("supported_data_types").and_then(|v| v.as_array_mut()) {
                                    arr.push(dt_json);
                                }
                            }
                        }
                    }
                }
            }
        }

        templates.insert(
            format!("o{}", String::from_utf8_lossy(&cortical_id_ref)),
            json!({
                "type": "motor",
                "friendly_name": friendly_name,
                "cortical_id_prefix": String::from_utf8_lossy(&cortical_id_ref).to_string(),
                "number_of_cortical_areas": num_areas,
                "subunits": subunits,
                "description": format!("Motor output: {}", friendly_name)
            }),
        );
    }

    // Add sensory types
    for sensory_unit in SensoryCorticalUnit::list_all() {
        let friendly_name = sensory_unit.get_friendly_name();
        let cortical_id_ref = sensory_unit.get_cortical_id_unit_reference();
        let num_areas = sensory_unit.get_number_cortical_areas();
        let topology = sensory_unit.get_unit_default_topology();

        use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
        use serde_json::{Map, Value};
        use std::collections::HashMap as StdHashMap;

        let mut subunits: StdHashMap<String, serde_json::Value> = StdHashMap::new();

        for (sub_idx, topo) in topology {
            subunits.insert(
                sub_idx.get().to_string(),
                json!({
                    "relative_position": topo.relative_position,
                    "channel_dimensions_default": topo.channel_dimensions_default,
                    "channel_dimensions_min": topo.channel_dimensions_min,
                    "channel_dimensions_max": topo.channel_dimensions_max,
                    "supported_data_types": Vec::<serde_json::Value>::new(),
                }),
            );
        }

        let allowed_frames = sensory_unit.get_allowed_frame_change_handling();
        let frames: Vec<FrameChangeHandling> = match allowed_frames {
            Some(allowed) => allowed.to_vec(),
            None => vec![FrameChangeHandling::Absolute, FrameChangeHandling::Incremental],
        };

        let positionings = [
            PercentageNeuronPositioning::Linear,
            PercentageNeuronPositioning::Fractional,
        ];

        let mut per_subunit_dedup: StdHashMap<String, std::collections::HashSet<String>> =
            StdHashMap::new();

        for frame in frames {
            for positioning in positionings {
                let mut map: Map<String, Value> = Map::new();
                map.insert(
                    "frame_change_handling".to_string(),
                    serde_json::to_value(frame).unwrap_or(Value::Null),
                );
                map.insert(
                    "percentage_neuron_positioning".to_string(),
                    serde_json::to_value(positioning).unwrap_or(Value::Null),
                );

                let cortical_ids = sensory_unit
                    .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
                        CorticalUnitIndex::from(0u8),
                        map,
                    );

                if let Ok(ids) = cortical_ids {
                    for (i, id) in ids.into_iter().enumerate() {
                        if let Ok(flag) = id.extract_io_data_flag() {
                            let dt_json = data_type_to_json(flag);
                            let subunit_key = i.to_string();

                            let dedup_key = format!(
                                "{}|{}|{}",
                                dt_json.get("variant").and_then(|v| v.as_str()).unwrap_or(""),
                                dt_json
                                    .get("frame_change_handling")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(""),
                                dt_json
                                    .get("percentage_positioning")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                            );

                            let seen = per_subunit_dedup
                                .entry(subunit_key.clone())
                                .or_insert_with(std::collections::HashSet::new);
                            if !seen.insert(dedup_key) {
                                continue;
                            }

                            if let Some(subunit_obj) = subunits.get_mut(&subunit_key) {
                                if let Some(arr) = subunit_obj.get_mut("supported_data_types").and_then(|v| v.as_array_mut()) {
                                    arr.push(dt_json);
                                }
                            }
                        }
                    }
                }
            }
        }

        templates.insert(
            format!("i{}", String::from_utf8_lossy(&cortical_id_ref)),
            json!({
                "type": "sensory",
                "friendly_name": friendly_name,
                "cortical_id_prefix": String::from_utf8_lossy(&cortical_id_ref).to_string(),
                "number_of_cortical_areas": num_areas,
                "subunits": subunits,
                "description": format!("Sensory input: {}", friendly_name)
            }),
        );
    }

    Ok(Json(templates))
}

/// Get list of available embedded default genome templates (barebones, essential, test, vision).
#[utoipa::path(get, path = "/v1/genome/defaults/files", tag = "genome")]
pub async fn get_defaults_files(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![
        "barebones".to_string(),
        "essential".to_string(),
        "test".to_string(),
        "vision".to_string(),
    ]))
}

/// Download a specific brain region from the genome.
#[utoipa::path(get, path = "/v1/genome/download_region", tag = "genome")]
pub async fn get_download_region(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
}

/// Get the current genome number or generation identifier.
#[utoipa::path(get, path = "/v1/genome/genome_number", tag = "genome")]
pub async fn get_genome_number(State(_state): State<ApiState>) -> ApiResult<Json<i32>> {
    Ok(Json(0))
}

/// Perform genome amalgamation by specifying a filename.
#[utoipa::path(post, path = "/v1/genome/amalgamation_by_filename", tag = "genome")]
pub async fn post_amalgamation_by_filename(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Perform genome amalgamation using a direct JSON payload.
#[utoipa::path(post, path = "/v1/genome/amalgamation_by_payload", tag = "genome")]
pub async fn post_amalgamation_by_payload(
    State(_state): State<ApiState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Perform genome amalgamation by uploading a genome file.
#[utoipa::path(post, path = "/v1/genome/amalgamation_by_upload", tag = "genome")]
pub async fn post_amalgamation_by_upload(
    State(_state): State<ApiState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Append structures to the genome from a file.
#[utoipa::path(post, path = "/v1/genome/append-file", tag = "genome")]
pub async fn post_append_file(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Upload and load a genome from a file.
#[utoipa::path(post, path = "/v1/genome/upload/file", tag = "genome")]
pub async fn post_upload_file(
    State(_state): State<ApiState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Upload a genome file with edit mode enabled.
#[utoipa::path(post, path = "/v1/genome/upload/file/edit", tag = "genome")]
pub async fn post_upload_file_edit(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Upload and load a genome from a JSON string.
#[utoipa::path(post, path = "/v1/genome/upload/string", tag = "genome")]
pub async fn post_upload_string(
    State(_state): State<ApiState>,
    Json(_req): Json<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

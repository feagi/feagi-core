// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Genome API Endpoints - Exact port from Python `/v1/genome/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;
use feagi_services::types::LoadGenomeParams;

/// GET /v1/genome/file_name
#[utoipa::path(get, path = "/v1/genome/file_name", tag = "genome")]
pub async fn get_file_name(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Get current genome filename
    Ok(Json(HashMap::from([("genome_file_name".to_string(), "".to_string())])))
}

/// GET /v1/genome/circuits
#[utoipa::path(get, path = "/v1/genome/circuits", tag = "genome")]
pub async fn get_circuits(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    // TODO: Get available circuit library
    Ok(Json(vec![]))
}

/// POST /v1/genome/amalgamation_destination
#[utoipa::path(post, path = "/v1/genome/amalgamation_destination", tag = "genome")]
pub async fn post_amalgamation_destination(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// DELETE /v1/genome/amalgamation_cancellation
#[utoipa::path(delete, path = "/v1/genome/amalgamation_cancellation", tag = "genome")]
pub async fn delete_amalgamation_cancellation(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/feagi/genome/append
#[utoipa::path(post, path = "/v1/feagi/genome/append", tag = "genome")]
pub async fn post_genome_append(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/genome/upload/barebones
/// 
/// Load the barebones genome from default templates
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
    load_default_genome(state, "barebones").await
}

/// POST /v1/genome/upload/essential
/// 
/// Load the essential genome from default templates
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
    tracing::info!(target: "feagi-api", "Loading {} genome from embedded Rust genomes", genome_name);
    
    // Load genome from embedded Rust templates (no file I/O!)
    let genome_json = match genome_name {
        "barebones" => feagi_evo::BAREBONES_GENOME_JSON,
        "essential" => feagi_evo::ESSENTIAL_GENOME_JSON,
        "test" => feagi_evo::TEST_GENOME_JSON,
        "vision" => feagi_evo::VISION_GENOME_JSON,
        _ => return Err(ApiError::invalid_input(&format!(
            "Unknown genome name '{}'. Available: barebones, essential, test, vision", 
            genome_name
        ))),
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
    
    // Return response matching Python format
    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::Value::Bool(true));
    response.insert("message".to_string(), serde_json::Value::String(format!("{} genome loaded successfully", genome_name)));
    response.insert("cortical_area_count".to_string(), serde_json::Value::Number(genome_info.cortical_area_count.into()));
    response.insert("brain_region_count".to_string(), serde_json::Value::Number(genome_info.brain_region_count.into()));
    response.insert("genome_id".to_string(), serde_json::Value::String(genome_info.genome_id));
    response.insert("genome_title".to_string(), serde_json::Value::String(genome_info.genome_title));
    
    Ok(Json(response))
}

/// GET /v1/genome/name
/// Get the current genome name
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

/// GET /v1/genome/timestamp
/// Get the current genome timestamp
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

/// POST /v1/genome/save
/// Save current genome to file
#[utoipa::path(
    post,
    path = "/v1/genome/save",
    tag = "genome",
    responses(
        (status = 200, description = "Genome saved", body = HashMap<String, String>)
    )
)]
pub async fn post_save(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement genome save
    Ok(Json(HashMap::from([
        ("message".to_string(), "Genome save not yet implemented".to_string())
    ])))
}

/// POST /v1/genome/load
/// Load genome from file
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
    let genome_name = request.get("genome_name")
        .ok_or_else(|| ApiError::invalid_input("genome_name required"))?;
    
    // Load genome from defaults
    let genome_service = state.genome_service.as_ref();
    let params = feagi_services::LoadGenomeParams {
        json_str: format!("{{\"genome_title\": \"{}\"}}", genome_name),
    };
    
    let genome_info = genome_service.load_genome(params).await
        .map_err(|e| ApiError::internal(format!("Failed to load genome: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("message".to_string(), serde_json::json!("Genome loaded successfully"));
    response.insert("genome_title".to_string(), serde_json::json!(genome_info.genome_title));
    
    Ok(Json(response))
}

/// POST /v1/genome/upload
/// Upload and load genome from JSON
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
        .map_err(|e| ApiError::invalid_input(&format!("Invalid JSON: {}", e)))?;
    
    let params = LoadGenomeParams { json_str };
    let genome_info = genome_service.load_genome(params).await
        .map_err(|e| ApiError::internal(format!("Failed to upload genome: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::json!(true));
    response.insert("message".to_string(), serde_json::json!("Genome uploaded successfully"));
    response.insert("cortical_area_count".to_string(), serde_json::json!(genome_info.cortical_area_count));
    response.insert("brain_region_count".to_string(), serde_json::json!(genome_info.brain_region_count));
    
    Ok(Json(response))
}

/// GET /v1/genome/download
/// Download current genome as JSON
#[utoipa::path(
    get,
    path = "/v1/genome/download",
    tag = "genome",
    responses(
        (status = 200, description = "Genome JSON", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_download(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement proper genome export from genome service
    Ok(Json(HashMap::new()))
}

/// GET /v1/genome/properties
/// Get genome properties and metadata
#[utoipa::path(
    get,
    path = "/v1/genome/properties",
    tag = "genome",
    responses(
        (status = 200, description = "Genome properties", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_properties(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement proper metadata retrieval from genome service
    Ok(Json(HashMap::new()))
}

/// POST /v1/genome/validate
/// Validate a genome structure
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

/// POST /v1/genome/transform
/// Transform genome between formats (flat <-> hierarchical)
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
    response.insert("message".to_string(), serde_json::json!("Genome transformation not yet implemented"));
    
    Ok(Json(response))
}

/// POST /v1/genome/clone
/// Clone the current genome with a new name
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
    Ok(Json(HashMap::from([
        ("message".to_string(), "Genome cloning not yet implemented".to_string())
    ])))
}

/// POST /v1/genome/reset
/// Reset genome to default state
#[utoipa::path(
    post,
    path = "/v1/genome/reset",
    tag = "genome",
    responses(
        (status = 200, description = "Genome reset", body = HashMap<String, String>)
    )
)]
pub async fn post_reset(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement genome reset
    Ok(Json(HashMap::from([
        ("message".to_string(), "Genome reset not yet implemented".to_string())
    ])))
}

/// GET /v1/genome/metadata
/// Get genome metadata (alternative to properties)
#[utoipa::path(
    get,
    path = "/v1/genome/metadata",
    tag = "genome",
    responses(
        (status = 200, description = "Genome metadata", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_metadata(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    get_properties(State(state)).await
}

/// POST /v1/genome/merge
/// Merge another genome into current genome
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
    response.insert("message".to_string(), serde_json::json!("Genome merging not yet implemented"));
    
    Ok(Json(response))
}

/// GET /v1/genome/diff
/// Get diff between two genomes
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
    axum::extract::Query(_params): axum::extract::Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome diffing
    let mut response = HashMap::new();
    response.insert("differences".to_string(), serde_json::json!([]));
    
    Ok(Json(response))
}

/// POST /v1/genome/export_format
/// Export genome in specific format
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
    response.insert("message".to_string(), serde_json::json!("Format export not yet implemented"));
    
    Ok(Json(response))
}

// EXACT Python paths:
/// GET /v1/genome/amalgamation
#[utoipa::path(get, path = "/v1/genome/amalgamation", tag = "genome")]
pub async fn get_amalgamation(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
}

/// GET /v1/genome/amalgamation_history
#[utoipa::path(get, path = "/v1/genome/amalgamation_history", tag = "genome")]
pub async fn get_amalgamation_history_exact(State(_state): State<ApiState>) -> ApiResult<Json<Vec<HashMap<String, serde_json::Value>>>> {
    Ok(Json(Vec::new()))
}

/// GET /v1/genome/cortical_template
#[utoipa::path(get, path = "/v1/genome/cortical_template", tag = "genome")]
pub async fn get_cortical_template(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
}

/// GET /v1/genome/defaults/files
/// 
/// Returns list of available embedded default genomes
#[utoipa::path(get, path = "/v1/genome/defaults/files", tag = "genome")]
pub async fn get_defaults_files(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![
        "barebones".to_string(), 
        "essential".to_string(),
        "test".to_string(),
        "vision".to_string(),
    ]))
}

/// GET /v1/genome/download_region
#[utoipa::path(get, path = "/v1/genome/download_region", tag = "genome")]
pub async fn get_download_region(State(_state): State<ApiState>, axum::extract::Query(_params): axum::extract::Query<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    Ok(Json(HashMap::new()))
}

/// GET /v1/genome/genome_number
#[utoipa::path(get, path = "/v1/genome/genome_number", tag = "genome")]
pub async fn get_genome_number(State(_state): State<ApiState>) -> ApiResult<Json<i32>> {
    Ok(Json(0))
}

/// POST /v1/genome/amalgamation_by_filename
#[utoipa::path(post, path = "/v1/genome/amalgamation_by_filename", tag = "genome")]
pub async fn post_amalgamation_by_filename(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/genome/amalgamation_by_payload
#[utoipa::path(post, path = "/v1/genome/amalgamation_by_payload", tag = "genome")]
pub async fn post_amalgamation_by_payload(State(_state): State<ApiState>, Json(_req): Json<serde_json::Value>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/genome/amalgamation_by_upload
#[utoipa::path(post, path = "/v1/genome/amalgamation_by_upload", tag = "genome")]
pub async fn post_amalgamation_by_upload(State(_state): State<ApiState>, Json(_req): Json<serde_json::Value>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/genome/append-file
#[utoipa::path(post, path = "/v1/genome/append-file", tag = "genome")]
pub async fn post_append_file(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/genome/upload/file
#[utoipa::path(post, path = "/v1/genome/upload/file", tag = "genome")]
pub async fn post_upload_file(State(_state): State<ApiState>, Json(_req): Json<serde_json::Value>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/genome/upload/file/edit
#[utoipa::path(post, path = "/v1/genome/upload/file/edit", tag = "genome")]
pub async fn post_upload_file_edit(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, String>>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}

/// POST /v1/genome/upload/string
#[utoipa::path(post, path = "/v1/genome/upload/string", tag = "genome")]
pub async fn post_upload_string(State(_state): State<ApiState>, Json(_req): Json<String>) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([("message".to_string(), "Not yet implemented".to_string())])))
}



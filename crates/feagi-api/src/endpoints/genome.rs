// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Genome API Endpoints - Exact port from Python `/v1/genome/*`

use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
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

/// Helper function to load a default genome by name
async fn load_default_genome(
    state: ApiState,
    genome_name: &str,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // Find genome file path
    // Try multiple possible locations:
    // 1. Relative to current working directory
    // 2. Relative to FEAGI workspace root (if running from feagi-core or feagi directory)
    // 3. Using environment variable FEAGI_WORKSPACE_ROOT if set
    let genome_filename = format!("{}_genome.json", genome_name);
    
    let workspace_root = std::env::var("FEAGI_WORKSPACE_ROOT")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            // Try to find workspace root by looking for feagi-py directory
            let current_dir = std::env::current_dir().ok()?;
            let mut path = current_dir.clone();
            
            // Walk up the directory tree looking for feagi-py
            for _ in 0..5 {
                if path.join("feagi-py").exists() {
                    return Some(path);
                }
                path = path.parent()?.to_path_buf();
            }
            None
        });
    
    let possible_paths = if let Some(root) = workspace_root {
        vec![
            root.join("feagi-py").join("feagi").join("evo").join("defaults").join("genome").join(&genome_filename),
            PathBuf::from(&genome_filename), // Current directory fallback
        ]
    } else {
        vec![
            PathBuf::from(format!("../feagi-py/feagi/evo/defaults/genome/{}", genome_filename)),
            PathBuf::from(format!("../../feagi-py/feagi/evo/defaults/genome/{}", genome_filename)),
            PathBuf::from(format!("../../../feagi-py/feagi/evo/defaults/genome/{}", genome_filename)),
            PathBuf::from(&genome_filename), // Current directory fallback
        ]
    };
    
    let genome_path = possible_paths.iter()
        .find(|p| p.exists())
        .ok_or_else(|| ApiError::not_found(
            "Genome file",
            &format!("Default genome '{}' not found. Searched: {:?}", genome_name, possible_paths)
        ))?;
    
    tracing::info!(target: "feagi-api",target: "feagi-api", "Loading {} genome from: {}", genome_name, genome_path.display());
    
    // Read genome file
    let genome_json = std::fs::read_to_string(genome_path)
        .map_err(|e| ApiError::internal(format!("Failed to read genome file {}: {}", genome_path.display(), e)))?;
    
    tracing::info!(target: "feagi-api","Read {} bytes of genome JSON, starting conversion...", genome_json.len());
    
    // Load genome via service
    let genome_service = state.genome_service.as_ref();
    let params = LoadGenomeParams {
        json_str: genome_json,
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



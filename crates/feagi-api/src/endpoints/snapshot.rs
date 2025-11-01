/*!
 * FEAGI v1 Snapshot API
 * 
 * Endpoints for creating, managing, and restoring brain snapshots
 * Maps to Python: feagi/api/v1/snapshot.py
 */

use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;
use axum::{extract::{Path, State}, Json};
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// SNAPSHOT MANAGEMENT
// ============================================================================

/// POST /v1/snapshot/create
/// Create a new snapshot
#[utoipa::path(
    post,
    path = "/v1/snapshot/create",
    tag = "snapshot",
    responses(
        (status = 200, description = "Snapshot created", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_create(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, Value>>> {
    let snapshot_service = state.snapshot_service.as_ref()
        .ok_or_else(|| ApiError::internal("Snapshot service not available"))?;
    
    // Parse options from request
    let stateful = request.get("stateful").and_then(|v| v.as_bool()).unwrap_or(false);
    let compression = request.get("compression").and_then(|v| v.as_bool()).unwrap_or(true);
    let name = request.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let description = request.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    let options = feagi_services::SnapshotCreateOptions {
        name,
        description,
        stateful,
        compression,
    };
    
    // Create snapshot via service layer
    let metadata = snapshot_service.create_snapshot(options).await
        .map_err(|e| ApiError::internal(format!("Failed to create snapshot: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("snapshot_id".to_string(), json!(metadata.snapshot_id));
    response.insert("success".to_string(), json!(true));
    response.insert("message".to_string(), json!("Snapshot created successfully"));
    response.insert("path".to_string(), json!(format!("./snapshots/{}", metadata.snapshot_id)));
    
    Ok(Json(response))
}

/// POST /v1/snapshot/restore
/// Restore from a snapshot
#[utoipa::path(
    post,
    path = "/v1/snapshot/restore",
    tag = "snapshot",
    responses(
        (status = 200, description = "Snapshot restored", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_restore(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let snapshot_service = state.snapshot_service.as_ref()
        .ok_or_else(|| ApiError::internal("Snapshot service not available"))?;
    
    // Validate snapshot_id is provided
    let snapshot_id = request.get("snapshot_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("Missing 'snapshot_id' field"))?;
    
    // Restore snapshot via service layer
    snapshot_service.restore_snapshot(snapshot_id).await
        .map_err(|e| ApiError::internal(format!("Failed to restore snapshot: {}", e)))?;
    
    tracing::info!(target: "feagi-api", "Restored snapshot: {}", snapshot_id);
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Snapshot restored successfully".to_string())
    ])))
}

/// GET /v1/snapshot/
/// List all snapshots
#[utoipa::path(
    get,
    path = "/v1/snapshot/",
    tag = "snapshot",
    responses(
        (status = 200, description = "Snapshot list", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_list(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    let snapshot_service = state.snapshot_service.as_ref()
        .ok_or_else(|| ApiError::internal("Snapshot service not available"))?;
    
    // Get snapshot list from service layer
    let snapshots = snapshot_service.list_snapshots().await
        .map_err(|e| ApiError::internal(format!("Failed to list snapshots: {}", e)))?;
    
    let mut response = HashMap::new();
    response.insert("snapshots".to_string(), json!(snapshots));
    
    Ok(Json(response))
}

/// DELETE /v1/snapshot/{snapshot_id}
/// Delete a snapshot
#[utoipa::path(
    delete,
    path = "/v1/snapshot/{snapshot_id}",
    tag = "snapshot",
    responses(
        (status = 200, description = "Snapshot deleted", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_snapshot(
    State(state): State<ApiState>,
    Path(snapshot_id): Path<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let snapshot_service = state.snapshot_service.as_ref()
        .ok_or_else(|| ApiError::internal("Snapshot service not available"))?;
    
    // Delete snapshot via service layer
    snapshot_service.delete_snapshot(&snapshot_id).await
        .map_err(|e| ApiError::internal(format!("Failed to delete snapshot: {}", e)))?;
    
    tracing::info!(target: "feagi-api", "Deleted snapshot: {}", snapshot_id);
    
    Ok(Json(HashMap::from([
        ("message".to_string(), "Snapshot deleted successfully".to_string())
    ])))
}

/// GET /v1/snapshot/{snapshot_id}/artifact/{fmt}
/// Get snapshot artifact in specified format
#[utoipa::path(
    get,
    path = "/v1/snapshot/{snapshot_id}/artifact/{fmt}",
    tag = "snapshot",
    responses(
        (status = 200, description = "Snapshot artifact", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_artifact(
    State(_state): State<ApiState>,
    Path((_snapshot_id, _fmt)): Path<(String, String)>,
) -> ApiResult<Json<HashMap<String, Value>>> {
    // TODO: Retrieve snapshot artifact
    let mut response = HashMap::new();
    response.insert("artifact".to_string(), json!({}));
    
    Ok(Json(response))
}

/// POST /v1/snapshot/compare
/// Compare two snapshots
#[utoipa::path(
    post,
    path = "/v1/snapshot/compare",
    tag = "snapshot",
    responses(
        (status = 200, description = "Snapshot comparison", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_compare(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, Value>>> {
    // TODO: Compare snapshots
    let mut response = HashMap::new();
    response.insert("diff".to_string(), json!({}));
    
    Ok(Json(response))
}

/// POST /v1/snapshot/upload
/// Upload a snapshot
#[utoipa::path(
    post,
    path = "/v1/snapshot/upload",
    tag = "snapshot",
    responses(
        (status = 200, description = "Snapshot uploaded", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_upload(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, Value>>> {
    // TODO: Upload snapshot
    let snapshot_id = uuid::Uuid::new_v4().to_string();
    
    let mut response = HashMap::new();
    response.insert("snapshot_id".to_string(), json!(snapshot_id));
    response.insert("success".to_string(), json!(true));
    
    Ok(Json(response))
}


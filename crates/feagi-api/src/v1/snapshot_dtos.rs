// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Snapshot API DTOs
//! 
//! Request/response types for snapshot management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Snapshot creation request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Snapshot creation response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotCreateResponse {
    pub snapshot_id: String,
    pub success: bool,
    pub message: String,
}

/// Snapshot restore request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotRestoreRequest {
    pub snapshot_id: String,
}

/// Snapshot list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotListResponse {
    pub snapshots: Vec<SnapshotInfo>,
}

/// Snapshot information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotInfo {
    pub snapshot_id: String,
    pub name: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Snapshot artifact response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotArtifactResponse {
    pub artifact: HashMap<String, serde_json::Value>,
}

/// Snapshot comparison request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotCompareRequest {
    pub snapshot_id_1: String,
    pub snapshot_id_2: String,
}

/// Snapshot comparison response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotCompareResponse {
    pub diff: HashMap<String, serde_json::Value>,
}

/// Snapshot upload request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotUploadRequest {
    pub data: HashMap<String, serde_json::Value>,
}

/// Snapshot upload response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotUploadResponse {
    pub snapshot_id: String,
    pub success: bool,
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotSuccessResponse {
    pub message: String,
    pub success: bool,
}


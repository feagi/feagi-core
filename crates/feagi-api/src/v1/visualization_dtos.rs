// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Visualization API DTOs
//!
//! Request/response types for visualization client management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Visualization client registration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VisualizationClientRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Visualization client registration response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VisualizationClientResponse {
    pub client_id: String,
    pub success: bool,
    pub message: String,
}

/// Visualization heartbeat request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VisualizationHeartbeatRequest {
    pub client_id: String,
}

/// Visualization status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VisualizationStatusResponse {
    pub enabled: bool,
    pub active_clients: usize,
    pub fq_sampler_enabled: bool,
    pub message: String,
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VisualizationSuccessResponse {
    pub message: String,
    pub success: bool,
}

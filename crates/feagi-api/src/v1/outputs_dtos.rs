// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Outputs API DTOs
//! 
//! Request/response types for motor and output target management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Response for output targets
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OutputTargetsResponse {
    /// List of available output/motor agent IDs
    pub targets: Vec<String>,
}

/// Request to configure outputs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OutputConfigRequest {
    /// Configuration mapping for outputs
    pub config: HashMap<String, serde_json::Value>,
}

/// Success response for output configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OutputConfigResponse {
    pub message: String,
    pub success: bool,
}



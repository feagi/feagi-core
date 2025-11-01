// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Network API DTOs
//! 
//! Request/response types for network configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Network status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkStatusResponse {
    pub zmq_enabled: bool,
    pub http_enabled: bool,
    pub websocket_enabled: bool,
}

/// Network configuration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkConfigRequest {
    pub config: HashMap<String, serde_json::Value>,
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkSuccessResponse {
    pub message: String,
    pub success: bool,
}


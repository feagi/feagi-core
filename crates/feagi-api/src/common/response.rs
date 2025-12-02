// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// API response types

use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Whether the operation succeeded
    pub success: bool,
    
    /// Response data (present if success = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    
    /// ISO 8601 timestamp
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
    
    /// Create an error response (no data)
    pub fn error() -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

/// Empty response for operations that return no data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EmptyResponse {
    /// Operation message
    pub message: String,
}

impl EmptyResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

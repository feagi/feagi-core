// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Plasticity API Endpoints (Temporary/Debug)

use crate::common::{ApiError, ApiResult, Json, State};
use crate::common::ApiState;
use std::collections::HashMap;

/// POST /v1/plasticity/register_memory_area
/// Manually register a memory area with the plasticity executor (temporary debug endpoint)
#[utoipa::path(
    post,
    path = "/v1/plasticity/register_memory_area",
    tag = "plasticity",
    responses(
        (status = 200, description = "Memory area registered successfully", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_register_memory_area(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: This endpoint needs plasticity_executor in ApiState
    // For now, return not implemented
    Err(ApiError::internal(
        "Memory area registration not yet wired to API layer. \
         Registration should happen automatically when mappings are created."
    ))
}


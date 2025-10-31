// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Insight API Endpoints - Exact port from Python `/v1/insight/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

/// POST /v1/insight/neurons/membrane_potential_status
#[utoipa::path(post, path = "/v1/insight/neurons/membrane_potential_status", tag = "insight")]
pub async fn post_neurons_membrane_potential_status(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, f32>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/insight/neuron/synaptic_potential_status
#[utoipa::path(post, path = "/v1/insight/neuron/synaptic_potential_status", tag = "insight")]
pub async fn post_neuron_synaptic_potential_status(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/insight/neurons/membrane_potential_set
#[utoipa::path(post, path = "/v1/insight/neurons/membrane_potential_set", tag = "insight")]
pub async fn post_neurons_membrane_potential_set(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// POST /v1/insight/neuron/synaptic_potential_set
#[utoipa::path(post, path = "/v1/insight/neuron/synaptic_potential_set", tag = "insight")]
pub async fn post_neuron_synaptic_potential_set(State(_state): State<ApiState>, Json(_req): Json<HashMap<String, serde_json::Value>>) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}




// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Burst Engine API Endpoints - Exact port from Python `/v1/burst_engine/*`

use axum::{extract::State, response::Json};
use std::collections::HashMap;
use crate::common::{ApiError, ApiResult};
use crate::transports::http::server::ApiState;

/// GET /v1/burst_engine/simulation_timestep
#[utoipa::path(get, path = "/v1/burst_engine/simulation_timestep", tag = "burst_engine")]
pub async fn get_simulation_timestep(State(state): State<ApiState>) -> ApiResult<Json<f64>> {
    let runtime_service = state.runtime_service.as_ref();
    match runtime_service.get_status().await {
        Ok(status) => {
            // Convert frequency to timestep (1/Hz = seconds)
            let timestep = if status.frequency_hz > 0.0 {
                1.0 / status.frequency_hz
            } else {
                0.0
            };
            Ok(Json(timestep))
        },
        Err(e) => Err(ApiError::internal(format!("Failed to get timestep: {}", e))),
    }
}

/// POST /v1/burst_engine/simulation_timestep
#[utoipa::path(post, path = "/v1/burst_engine/simulation_timestep", tag = "burst_engine")]
pub async fn post_simulation_timestep(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, f64>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let runtime_service = state.runtime_service.as_ref();
    
    if let Some(&timestep) = request.get("simulation_timestep") {
        // Convert timestep (seconds) to frequency (Hz)
        let frequency = if timestep > 0.0 { 1.0 / timestep } else { 0.0 };
        
        match runtime_service.set_frequency(frequency).await {
            Ok(_) => Ok(Json(HashMap::from([("message".to_string(), format!("Timestep set to {}", timestep))]))),
            Err(e) => Err(ApiError::internal(format!("Failed to set timestep: {}", e))),
        }
    } else {
        Err(ApiError::invalid_input("simulation_timestep required"))
    }
}



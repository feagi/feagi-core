// Runtime control endpoints (transport-agnostic)
//
// These endpoints provide control over the FEAGI burst engine runtime.

use std::sync::Arc;
use utoipa;

use crate::{
    common::{ApiError, ApiResult},
    security::AuthContext,
    v1::{RuntimeStatusResponse, SetFrequencyRequest, BurstCountResponse},
};
use feagi_services::RuntimeService;

/// Get runtime status
///
/// Returns the current status of the burst engine.
#[utoipa::path(
    get,
    path = "/api/v1/runtime/status",
    tag = "Runtime",
    responses(
        (status = 200, description = "Runtime status retrieved successfully", body = RuntimeStatusResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_runtime_status(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
) -> ApiResult<RuntimeStatusResponse> {
    // Get status from runtime service
    let status = runtime_service
        .get_status()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get runtime status: {}", e)))?;
    
    // Map service DTO to API DTO
    Ok(RuntimeStatusResponse {
        is_running: status.is_running,
        is_paused: status.is_paused,
        frequency_hz: status.frequency_hz,
        burst_count: status.burst_count,
        current_rate_hz: status.current_rate_hz,
        last_burst_neuron_count: status.last_burst_neuron_count,
        avg_burst_time_ms: status.avg_burst_time_ms,
    })
}

/// Start the burst engine
///
/// Begins executing neural bursts at the configured frequency.
#[utoipa::path(
    post,
    path = "/api/v1/runtime/start",
    tag = "Runtime",
    responses(
        (status = 200, description = "Burst engine started successfully"),
        (status = 409, description = "Already running", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn start_runtime(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
) -> ApiResult<()> {
    runtime_service
        .start()
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::InvalidState(msg) => {
                ApiError::conflict(msg)
            }
            _ => ApiError::internal(&format!("Failed to start burst engine: {}", e)),
        })?;
    
    Ok(())
}

/// Stop the burst engine
///
/// Gracefully stops burst execution.
#[utoipa::path(
    post,
    path = "/api/v1/runtime/stop",
    tag = "Runtime",
    responses(
        (status = 200, description = "Burst engine stopped successfully"),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn stop_runtime(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
) -> ApiResult<()> {
    runtime_service
        .stop()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to stop burst engine: {}", e)))?;
    
    Ok(())
}

/// Pause the burst engine
///
/// Temporarily pauses burst execution without stopping the thread.
#[utoipa::path(
    post,
    path = "/api/v1/runtime/pause",
    tag = "Runtime",
    responses(
        (status = 200, description = "Burst engine paused successfully"),
        (status = 409, description = "Not running or invalid state", body = ApiError),
        (status = 501, description = "Not implemented", body = ApiError)
    )
)]
pub async fn pause_runtime(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
) -> ApiResult<()> {
    runtime_service
        .pause()
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::InvalidState(msg) => {
                ApiError::conflict(msg)
            }
            feagi_services::ServiceError::NotImplemented(msg) => {
                ApiError::not_implemented(msg)
            }
            _ => ApiError::internal(&format!("Failed to pause burst engine: {}", e)),
        })?;
    
    Ok(())
}

/// Resume the burst engine
///
/// Resumes burst execution after pause.
#[utoipa::path(
    post,
    path = "/api/v1/runtime/resume",
    tag = "Runtime",
    responses(
        (status = 200, description = "Burst engine resumed successfully"),
        (status = 409, description = "Not paused or invalid state", body = ApiError),
        (status = 501, description = "Not implemented", body = ApiError)
    )
)]
pub async fn resume_runtime(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
) -> ApiResult<()> {
    runtime_service
        .resume()
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::InvalidState(msg) => {
                ApiError::conflict(msg)
            }
            feagi_services::ServiceError::NotImplemented(msg) => {
                ApiError::not_implemented(msg)
            }
            _ => ApiError::internal(&format!("Failed to resume burst engine: {}", e)),
        })?;
    
    Ok(())
}

/// Execute a single burst step
///
/// Executes one burst cycle and then pauses. Useful for debugging.
#[utoipa::path(
    post,
    path = "/api/v1/runtime/step",
    tag = "Runtime",
    responses(
        (status = 200, description = "Burst step executed successfully"),
        (status = 409, description = "Already running in continuous mode", body = ApiError),
        (status = 501, description = "Not implemented", body = ApiError)
    )
)]
pub async fn step_runtime(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
) -> ApiResult<()> {
    runtime_service
        .step()
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::InvalidState(msg) => {
                ApiError::conflict(msg)
            }
            feagi_services::ServiceError::NotImplemented(msg) => {
                ApiError::not_implemented(msg)
            }
            _ => ApiError::internal(&format!("Failed to execute burst step: {}", e)),
        })?;
    
    Ok(())
}

/// Set burst frequency
///
/// Changes the burst execution frequency (Hz).
#[utoipa::path(
    post,
    path = "/api/v1/runtime/frequency",
    tag = "Runtime",
    request_body = SetFrequencyRequest,
    responses(
        (status = 200, description = "Frequency set successfully"),
        (status = 400, description = "Invalid frequency", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn set_frequency(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
    request: SetFrequencyRequest,
) -> ApiResult<()> {
    runtime_service
        .set_frequency(request.frequency_hz)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::InvalidInput(msg) => {
                ApiError::invalid_input(msg)
            }
            _ => ApiError::internal(&format!("Failed to set frequency: {}", e)),
        })?;
    
    Ok(())
}

/// Get burst count
///
/// Returns the total number of bursts executed since start.
#[utoipa::path(
    get,
    path = "/api/v1/runtime/burst-count",
    tag = "Runtime",
    responses(
        (status = 200, description = "Burst count retrieved successfully", body = BurstCountResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_burst_count(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
) -> ApiResult<BurstCountResponse> {
    let burst_count = runtime_service
        .get_burst_count()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get burst count: {}", e)))?;
    
    Ok(BurstCountResponse { burst_count })
}

/// Reset burst count
///
/// Resets the burst counter to zero.
#[utoipa::path(
    post,
    path = "/api/v1/runtime/reset-count",
    tag = "Runtime",
    responses(
        (status = 200, description = "Burst count reset successfully"),
        (status = 501, description = "Not implemented", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn reset_burst_count(
    _auth_ctx: &AuthContext,
    runtime_service: Arc<dyn RuntimeService + Send + Sync>,
) -> ApiResult<()> {
    runtime_service
        .reset_burst_count()
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotImplemented(msg) => {
                ApiError::not_implemented(msg)
            }
            _ => ApiError::internal(&format!("Failed to reset burst count: {}", e)),
        })?;
    
    Ok(())
}


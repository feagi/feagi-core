// Health check endpoints (transport-agnostic)
//
// These endpoints provide system health and readiness information.
// They are called by both HTTP and ZMQ adapters.

use std::sync::Arc;
use utoipa;

use crate::{
    common::{ApiError, ApiResult},
    security::AuthContext,
    v1::dtos::{ComponentReadiness, HealthCheckResponseV1, ReadinessCheckResponseV1},
};
use feagi_services::AnalyticsService;

/// Get system health status
///
/// Returns comprehensive health information including brain readiness,
/// burst engine status, neuron/synapse counts, and more.
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "System health retrieved successfully", body = HealthCheckResponseV1),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn health_check(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
) -> ApiResult<HealthCheckResponseV1> {
    // Get system health from analytics service
    let health = analytics_service
        .get_system_health()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get system health: {}", e)))?;

    // Map to V1 response format (exactly matching Python FastAPI)
    Ok(HealthCheckResponseV1 {
        status: if health.brain_readiness { "healthy".to_string() } else { "degraded".to_string() },
        brain_readiness: health.brain_readiness,
        burst_engine: health.burst_engine_active,
        neuron_count: health.neuron_count,
        
        // TODO: These fields are not yet available in SystemHealth DTO
        // They need to be added to feagi-services::SystemHealth
        synapse_count: 0, // TODO: Get from NPU when available
        cortical_area_count: health.cortical_area_count,
        genome_validity: true, // TODO: Get from genome validator
        influxdb_availability: false, // TODO: Get from analytics service
        connectome_path: String::new(), // TODO: Get from state manager
        genome_timestamp: String::new(), // TODO: Get from genome service
        change_state: "unknown".to_string(), // TODO: Get from state manager
        changes_saved_externally: false, // TODO: Get from state manager
    })
}

/// Get system readiness status
///
/// Simple health check for load balancers and orchestration systems.
/// Returns true if the system is ready to accept requests.
#[utoipa::path(
    get,
    path = "/api/v1/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Readiness status retrieved successfully", body = ReadinessCheckResponseV1),
        (status = 503, description = "Service unavailable", body = ApiError)
    )
)]
pub async fn readiness_check(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
) -> ApiResult<ReadinessCheckResponseV1> {
    // Get system health
    let health = analytics_service
        .get_system_health()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get system health: {}", e)))?;

    // System is ready if brain is ready and burst engine is active
    let ready = health.brain_readiness && health.burst_engine_active;

    Ok(ReadinessCheckResponseV1 {
        ready,
        components: ComponentReadiness {
            api: true, // If we're responding, API is ready
            burst_engine: health.burst_engine_active,
            state_manager: true, // TODO: Get from state manager
            connectome: health.brain_readiness,
        },
    })
}

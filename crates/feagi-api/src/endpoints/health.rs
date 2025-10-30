// Health and readiness check endpoints (transport-agnostic)

use crate::common::{ApiError, ApiResult};
use crate::security::AuthContext;
use crate::v1::dtos::{ComponentReadiness, HealthCheckResponseV1, ReadinessCheckResponseV1};
use feagi_services::AnalyticsService;
use std::sync::Arc;

/// Health check endpoint (transport-agnostic)
/// 
/// This endpoint returns detailed system health information.
/// It's called by both HTTP and ZMQ adapters.
pub async fn health_check(
    _auth_ctx: &AuthContext,  // Stub: not used yet
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
) -> ApiResult<HealthCheckResponseV1> {
    // Get system health from analytics service
    let health = analytics_service
        .get_system_health()
        .await
        .map_err(ApiError::from)?;
    
    // Convert service DTO to API V1 DTO (matching Python FastAPI format exactly)
    let response = HealthCheckResponseV1 {
        status: if health.brain_readiness { "healthy".to_string() } else { "initializing".to_string() },
        brain_readiness: health.brain_readiness,
        burst_engine: health.burst_engine_active,
        neuron_count: health.neuron_count as u64,
        synapse_count: 0,  // TODO: Add synapse count to service layer
        cortical_area_count: health.cortical_area_count,
        genome_validity: true,  // TODO: Add genome validation to service layer
        influxdb_availability: false,  // TODO: Add InfluxDB check to service layer
        connectome_path: "".to_string(),  // TODO: Add connectome path to service layer
        genome_timestamp: "".to_string(),  // TODO: Add genome timestamp to service layer
        change_state: "unknown".to_string(),  // TODO: Add change tracking to service layer
        changes_saved_externally: false,  // TODO: Add change tracking to service layer
    };
    
    Ok(response)
}

/// Readiness check endpoint (transport-agnostic)
/// 
/// This endpoint returns a simple ready/not-ready status for load balancers.
pub async fn readiness_check(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
) -> ApiResult<ReadinessCheckResponseV1> {
    let health = analytics_service
        .get_system_health()
        .await
        .map_err(ApiError::from)?;
    
    // System is ready if burst engine is initialized and brain is ready
    let ready = health.burst_engine_active && health.brain_readiness;
    
    let response = ReadinessCheckResponseV1 {
        ready,
        components: ComponentReadiness {
            api: true,  // If we're here, API is ready
            burst_engine: health.burst_engine_active,
            state_manager: true,  // Assumed ready if we can query
            connectome: health.brain_readiness,
        },
    };
    
    Ok(response)
}

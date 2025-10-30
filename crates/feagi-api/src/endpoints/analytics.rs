// Analytics endpoints (transport-agnostic)
//
// These endpoints provide system monitoring and statistics.

use std::sync::Arc;
use utoipa;

use crate::{
    common::{ApiError, ApiResult},
    security::AuthContext,
    v1::{
        SystemHealthResponse, CorticalAreaStatsResponse, ConnectivityStatsResponse,
        ConnectomeAnalyticsResponse, PopulatedAreasResponse, PopulatedAreaInfo,
        NeuronDensityResponse,
    },
};
use feagi_services::AnalyticsService;

/// Get system health
///
/// Returns overall system health status.
#[utoipa::path(
    get,
    path = "/api/v1/analytics/health",
    tag = "Analytics",
    responses(
        (status = 200, description = "System health retrieved successfully", body = SystemHealthResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_system_health(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
) -> ApiResult<SystemHealthResponse> {
    // Get system health from analytics service
    let health = analytics_service
        .get_system_health()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get system health: {}", e)))?;
    
    // Map service DTO to API DTO
    Ok(SystemHealthResponse {
        burst_engine_active: health.burst_engine_active,
        brain_readiness: health.brain_readiness,
        neuron_count: health.neuron_count,
        cortical_area_count: health.cortical_area_count,
        burst_count: health.burst_count,
    })
}

/// Get cortical area statistics
///
/// Returns statistics for a specific cortical area.
#[utoipa::path(
    get,
    path = "/api/v1/analytics/areas/{id}/stats",
    tag = "Analytics",
    params(
        ("id" = String, Path, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Statistics retrieved successfully", body = CorticalAreaStatsResponse),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_cortical_area_stats(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
    cortical_id: String,
) -> ApiResult<CorticalAreaStatsResponse> {
    // Get stats from analytics service
    let stats = analytics_service
        .get_cortical_area_stats(&cortical_id)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Cortical area", &cortical_id)
            }
            _ => ApiError::internal(&format!("Failed to get cortical area stats: {}", e)),
        })?;
    
    // Map service DTO to API DTO
    Ok(CorticalAreaStatsResponse {
        cortical_id: stats.cortical_id,
        neuron_count: stats.neuron_count,
        synapse_count: stats.synapse_count,
        density: stats.density,
        populated: stats.populated,
    })
}

/// Get all cortical area statistics
///
/// Returns statistics for all cortical areas.
#[utoipa::path(
    get,
    path = "/api/v1/analytics/areas/stats",
    tag = "Analytics",
    responses(
        (status = 200, description = "Statistics retrieved successfully", body = Vec<CorticalAreaStatsResponse>),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_all_cortical_area_stats(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
) -> ApiResult<Vec<CorticalAreaStatsResponse>> {
    // Get stats from analytics service
    let stats_list = analytics_service
        .get_all_cortical_area_stats()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get cortical area stats: {}", e)))?;
    
    // Map service DTOs to API DTOs
    let responses: Vec<CorticalAreaStatsResponse> = stats_list
        .into_iter()
        .map(|stats| CorticalAreaStatsResponse {
            cortical_id: stats.cortical_id,
            neuron_count: stats.neuron_count,
            synapse_count: stats.synapse_count,
            density: stats.density,
            populated: stats.populated,
        })
        .collect();
    
    Ok(responses)
}

/// Get connectivity statistics
///
/// Returns connectivity statistics between two cortical areas.
#[utoipa::path(
    get,
    path = "/api/v1/analytics/connectivity/{source}/{target}",
    tag = "Analytics",
    params(
        ("source" = String, Path, description = "Source cortical area ID"),
        ("target" = String, Path, description = "Target cortical area ID")
    ),
    responses(
        (status = 200, description = "Connectivity statistics retrieved successfully", body = ConnectivityStatsResponse),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_connectivity_stats(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
    source_area: String,
    target_area: String,
) -> ApiResult<ConnectivityStatsResponse> {
    // Get connectivity stats from analytics service
    let stats = analytics_service
        .get_connectivity_stats(&source_area, &target_area)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { resource, id } => {
                ApiError::not_found(&resource, &id)
            }
            _ => ApiError::internal(&format!("Failed to get connectivity stats: {}", e)),
        })?;
    
    // Map service DTO to API DTO
    Ok(ConnectivityStatsResponse {
        source_area: stats.source_area,
        target_area: stats.target_area,
        synapse_count: stats.synapse_count,
        avg_weight: stats.avg_weight,
        excitatory_count: stats.excitatory_count,
        inhibitory_count: stats.inhibitory_count,
    })
}

/// Get connectome statistics
///
/// Returns comprehensive statistics about the entire connectome.
#[utoipa::path(
    get,
    path = "/api/v1/analytics/connectome/stats",
    tag = "Analytics",
    responses(
        (status = 200, description = "Connectome statistics retrieved successfully", body = ConnectomeAnalyticsResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_connectome_stats(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
) -> ApiResult<ConnectomeAnalyticsResponse> {
    // Get total counts
    let total_neurons = analytics_service
        .get_total_neuron_count()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get total neuron count: {}", e)))?;
    
    let total_synapses = analytics_service
        .get_total_synapse_count()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get total synapse count: {}", e)))?;
    
    // Get all area stats
    let all_stats = analytics_service
        .get_all_cortical_area_stats()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get all area stats: {}", e)))?;
    
    // Calculate populated areas and average density
    let populated_areas = all_stats.iter().filter(|s| s.populated).count();
    let avg_density = if !all_stats.is_empty() {
        all_stats.iter().map(|s| s.density).sum::<f32>() / all_stats.len() as f32
    } else {
        0.0
    };
    
    // Build per-area stats map
    let per_area_stats: std::collections::HashMap<String, CorticalAreaStatsResponse> = all_stats
        .into_iter()
        .map(|stats| {
            let area_id = stats.cortical_id.clone();
            let response = CorticalAreaStatsResponse {
                cortical_id: stats.cortical_id,
                neuron_count: stats.neuron_count,
                synapse_count: stats.synapse_count,
                density: stats.density,
                populated: stats.populated,
            };
            (area_id, response)
        })
        .collect();
    
    Ok(ConnectomeAnalyticsResponse {
        total_neurons,
        total_synapses,
        total_cortical_areas: per_area_stats.len(),
        populated_areas,
        avg_density,
        per_area_stats,
    })
}

/// Get populated areas
///
/// Returns list of cortical areas that have neurons.
#[utoipa::path(
    get,
    path = "/api/v1/analytics/areas/populated",
    tag = "Analytics",
    responses(
        (status = 200, description = "Populated areas retrieved successfully", body = PopulatedAreasResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_populated_areas(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
) -> ApiResult<PopulatedAreasResponse> {
    // Get populated areas from analytics service
    let populated = analytics_service
        .get_populated_areas()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get populated areas: {}", e)))?;
    
    // Map to API DTOs
    let areas: Vec<PopulatedAreaInfo> = populated
        .into_iter()
        .map(|(cortical_id, neuron_count)| PopulatedAreaInfo {
            cortical_id,
            neuron_count,
        })
        .collect();
    
    let total_count = areas.len();
    
    Ok(PopulatedAreasResponse {
        areas,
        total_count,
    })
}

/// Get neuron density
///
/// Returns the neuron density for a specific cortical area.
#[utoipa::path(
    get,
    path = "/api/v1/analytics/areas/{id}/density",
    tag = "Analytics",
    params(
        ("id" = String, Path, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Neuron density retrieved successfully", body = NeuronDensityResponse),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_neuron_density(
    _auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
    cortical_id: String,
) -> ApiResult<NeuronDensityResponse> {
    // Get density from analytics service
    let density = analytics_service
        .get_neuron_density(&cortical_id)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Cortical area", &cortical_id)
            }
            _ => ApiError::internal(&format!("Failed to get neuron density: {}", e)),
        })?;
    
    Ok(NeuronDensityResponse {
        cortical_id,
        density,
    })
}


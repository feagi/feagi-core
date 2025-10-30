// Cortical area endpoints (transport-agnostic)
//
// These endpoints provide CRUD operations for cortical areas.

use std::sync::Arc;
use utoipa;

use crate::{
    common::{ApiError, ApiResult},
    security::AuthContext,
    v1::{
        CorticalAreaSummary, CorticalAreaDetail, CorticalAreaListResponse,
        CreateCorticalAreaRequest, UpdateCorticalAreaRequest,
        Coordinates3D, Dimensions3D,
    },
};
use feagi_services::{ConnectomeService, CreateCorticalAreaParams, UpdateCorticalAreaParams};

/// List all cortical areas
///
/// Returns a list of all cortical areas with summary information.
#[utoipa::path(
    get,
    path = "/api/v1/cortical-areas",
    tag = "Cortical Areas",
    responses(
        (status = 200, description = "Cortical areas retrieved successfully", body = CorticalAreaListResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn list_cortical_areas(
    _auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
) -> ApiResult<CorticalAreaListResponse> {
    // Get all cortical areas from connectome service
    let cortical_areas = connectome_service
        .list_cortical_areas()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to list cortical areas: {}", e)))?;
    
    // Map service DTOs to API DTOs
    let summaries: Vec<CorticalAreaSummary> = cortical_areas
        .iter()
        .map(|area| CorticalAreaSummary {
            cortical_id: area.cortical_id.clone(),
            cortical_name: area.name.clone(),
            cortical_group: area.area_type.clone(),
            coordinates_3d: Coordinates3D {
                x: area.position.0,
                y: area.position.1,
                z: area.position.2,
            },
            cortical_dimensions: Dimensions3D {
                x: area.dimensions.0 as u32,
                y: area.dimensions.1 as u32,
                z: area.dimensions.2 as u32,
            },
            neuron_count: area.neuron_count,
            cortical_visibility: area.visible,
        })
        .collect();
    
    Ok(CorticalAreaListResponse {
        total_count: summaries.len(),
        cortical_areas: summaries,
    })
}

/// Get cortical area by ID
///
/// Returns detailed information about a specific cortical area.
#[utoipa::path(
    get,
    path = "/api/v1/cortical-areas/{id}",
    tag = "Cortical Areas",
    params(
        ("id" = String, Path, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Cortical area retrieved successfully", body = CorticalAreaDetail),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_cortical_area(
    _auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    cortical_id: String,
) -> ApiResult<CorticalAreaDetail> {
    // Get cortical area from connectome service
    let area = connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Cortical area", &cortical_id)
            }
            _ => ApiError::internal(&format!("Failed to get cortical area: {}", e)),
        })?;
    
    // Map service DTO to API DTO
    Ok(CorticalAreaDetail {
        cortical_id: area.cortical_id.clone(),
        cortical_name: area.name.clone(),
        cortical_group: area.area_type.clone(),
        coordinates_3d: Coordinates3D {
            x: area.position.0,
            y: area.position.1,
            z: area.position.2,
        },
        cortical_dimensions: Dimensions3D {
            x: area.dimensions.0 as u32,
            y: area.dimensions.1 as u32,
            z: area.dimensions.2 as u32,
        },
        neuron_count: area.neuron_count,
        synapse_count: area.synapse_count,
        cortical_visibility: area.visible,
        cortical_sub_group_name: area.sub_group.unwrap_or_default(),
        cortical_neuron_per_vox_count: area.neurons_per_voxel,
        postsynaptic_current: area.postsynaptic_current,
        plasticity_constant: area.plasticity_constant,
        degeneration: area.degeneration,
        psp_uniform_distribution: area.psp_uniform_distribution,
        firing_threshold_increment: area.firing_threshold_increment,
        firing_threshold_limit: area.firing_threshold_limit,
        consecutive_fire_count: area.consecutive_fire_count,
        snooze_period: area.snooze_period,
        refractory_period: area.refractory_period,
        leak_coefficient: area.leak_coefficient,
        leak_variability: area.leak_variability,
        burst_engine_activation: area.burst_engine_active,
    })
}

/// Create a new cortical area
///
/// Creates a new cortical area with the specified parameters.
#[utoipa::path(
    post,
    path = "/api/v1/cortical-areas",
    tag = "Cortical Areas",
    request_body = CreateCorticalAreaRequest,
    responses(
        (status = 201, description = "Cortical area created successfully", body = CorticalAreaDetail),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 409, description = "Cortical area already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn create_cortical_area(
    auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    request: CreateCorticalAreaRequest,
) -> ApiResult<CorticalAreaDetail> {
    // Validate input
    if request.cortical_id.is_empty() {
        return Err(ApiError::invalid_input("Cortical ID cannot be empty"));
    }
    if request.cortical_dimensions.x == 0 || request.cortical_dimensions.y == 0 || request.cortical_dimensions.z == 0 {
        return Err(ApiError::invalid_input("Cortical dimensions must be positive"));
    }
    
    // Map API request to service params
    let params = CreateCorticalAreaParams {
        cortical_id: request.cortical_id.clone(),
        name: request.cortical_name,
        dimensions: (
            request.cortical_dimensions.x as usize,
            request.cortical_dimensions.y as usize,
            request.cortical_dimensions.z as usize,
        ),
        position: (
            request.coordinates_3d.x,
            request.coordinates_3d.y,
            request.coordinates_3d.z,
        ),
        area_type: request.cortical_group,
        visible: Some(request.cortical_visibility),
        sub_group: request.cortical_sub_group_name,
        neurons_per_voxel: Some(request.cortical_neuron_per_vox_count),
        postsynaptic_current: Some(request.postsynaptic_current),
        plasticity_constant: Some(request.plasticity_constant),
        degeneration: None,
        psp_uniform_distribution: None,
        firing_threshold_increment: None,
        firing_threshold_limit: None,
        consecutive_fire_count: None,
        snooze_period: None,
        refractory_period: None,
        leak_coefficient: None,
        leak_variability: None,
        burst_engine_active: None,
        properties: None,
    };
    
    // Create cortical area via service
    let created = connectome_service
        .create_cortical_area(params)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::AlreadyExists { .. } => {
                ApiError::conflict(format!("Cortical area '{}' already exists", request.cortical_id))
            }
            feagi_services::ServiceError::InvalidInput(msg) => {
                ApiError::invalid_input(msg)
            }
            _ => ApiError::internal(&format!("Failed to create cortical area: {}", e)),
        })?;
    
    // Return created area (use get_cortical_area to get full details)
    get_cortical_area(auth_ctx, connectome_service, created.cortical_id).await
}

/// Update a cortical area
///
/// Updates an existing cortical area with the specified parameters.
#[utoipa::path(
    put,
    path = "/api/v1/cortical-areas/{id}",
    tag = "Cortical Areas",
    params(
        ("id" = String, Path, description = "Cortical area ID")
    ),
    request_body = UpdateCorticalAreaRequest,
    responses(
        (status = 200, description = "Cortical area updated successfully", body = CorticalAreaDetail),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn update_cortical_area(
    auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    cortical_id: String,
    request: UpdateCorticalAreaRequest,
) -> ApiResult<CorticalAreaDetail> {
    // Map API request to service params
    let params = UpdateCorticalAreaParams {
        name: request.cortical_name,
        position: request.coordinates_3d.map(|c| (c.x, c.y, c.z)),
        dimensions: request.cortical_dimensions.map(|d| (d.x as usize, d.y as usize, d.z as usize)),
        area_type: request.cortical_group,
        visible: request.cortical_visibility,
        postsynaptic_current: request.postsynaptic_current,
        plasticity_constant: request.plasticity_constant,
        degeneration: None,
        psp_uniform_distribution: None,
        firing_threshold_increment: None,
        firing_threshold_limit: None,
        consecutive_fire_count: None,
        snooze_period: None,
        refractory_period: None,
        leak_coefficient: None,
        leak_variability: None,
        burst_engine_active: None,
    };
    
    // Update cortical area via service
    connectome_service
        .update_cortical_area(&cortical_id, params)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Cortical area", &cortical_id)
            }
            feagi_services::ServiceError::InvalidInput(msg) => {
                ApiError::invalid_input(msg)
            }
            _ => ApiError::internal(&format!("Failed to update cortical area: {}", e)),
        })?;
    
    // Return updated area
    get_cortical_area(auth_ctx, connectome_service, cortical_id).await
}

/// Delete a cortical area
///
/// Deletes the specified cortical area.
#[utoipa::path(
    delete,
    path = "/api/v1/cortical-areas/{id}",
    tag = "Cortical Areas",
    params(
        ("id" = String, Path, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Cortical area deleted successfully"),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn delete_cortical_area(
    _auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    cortical_id: String,
) -> ApiResult<()> {
    // Delete cortical area via service
    connectome_service
        .delete_cortical_area(&cortical_id)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Cortical area", &cortical_id)
            }
            _ => ApiError::internal(&format!("Failed to delete cortical area: {}", e)),
        })?;
    
    Ok(())
}

// Brain region endpoints (transport-agnostic)
//
// These endpoints provide CRUD operations for brain regions.

use std::sync::Arc;
use utoipa;

use crate::{
    common::{ApiError, ApiResult},
    security::AuthContext,
    v1::{
        BrainRegionSummary, BrainRegionDetail, BrainRegionListResponse,
        CreateBrainRegionRequest,
    },
};
use feagi_services::{ConnectomeService, CreateBrainRegionParams};

/// List all brain regions
///
/// Returns a list of all brain regions with summary information.
#[utoipa::path(
    get,
    path = "/api/v1/brain-regions",
    tag = "Brain Regions",
    responses(
        (status = 200, description = "Brain regions retrieved successfully", body = BrainRegionListResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn list_brain_regions(
    _auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
) -> ApiResult<BrainRegionListResponse> {
    // Get all brain regions from connectome service
    let brain_regions = connectome_service
        .list_brain_regions()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to list brain regions: {}", e)))?;
    
    // Map service DTOs to API DTOs
    let summaries: Vec<BrainRegionSummary> = brain_regions
        .iter()
        .map(|region| BrainRegionSummary {
            region_id: region.region_id.clone(),
            name: region.name.clone(),
            region_type: region.region_type.clone(),
            parent_id: region.parent_id.clone(),
            cortical_areas: region.cortical_areas.clone(),
            child_count: region.child_regions.len(),
        })
        .collect();
    
    Ok(BrainRegionListResponse {
        total_count: summaries.len(),
        brain_regions: summaries,
    })
}

/// Get brain region by ID
///
/// Returns detailed information about a specific brain region.
#[utoipa::path(
    get,
    path = "/api/v1/brain-regions/{id}",
    tag = "Brain Regions",
    params(
        ("id" = String, Path, description = "Brain region ID")
    ),
    responses(
        (status = 200, description = "Brain region retrieved successfully", body = BrainRegionDetail),
        (status = 404, description = "Brain region not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_brain_region(
    _auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    region_id: String,
) -> ApiResult<BrainRegionDetail> {
    // Get brain region from connectome service
    let region = connectome_service
        .get_brain_region(&region_id)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Brain region", &region_id)
            }
            _ => ApiError::internal(&format!("Failed to get brain region: {}", e)),
        })?;
    
    // Map service DTO to API DTO
    Ok(BrainRegionDetail {
        region_id: region.region_id.clone(),
        name: region.name.clone(),
        region_type: region.region_type.clone(),
        parent_id: region.parent_id.clone(),
        cortical_areas: region.cortical_areas.clone(),
        child_regions: region.child_regions.clone(),
        properties: serde_json::to_value(&region.properties)
            .unwrap_or(serde_json::json!({})),
    })
}

/// Create a new brain region
///
/// Creates a new brain region with the specified parameters.
#[utoipa::path(
    post,
    path = "/api/v1/brain-regions",
    tag = "Brain Regions",
    request_body = CreateBrainRegionRequest,
    responses(
        (status = 201, description = "Brain region created successfully", body = BrainRegionDetail),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 409, description = "Brain region already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn create_brain_region(
    auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    request: CreateBrainRegionRequest,
) -> ApiResult<BrainRegionDetail> {
    // Validate input
    if request.region_id.is_empty() {
        return Err(ApiError::invalid_input("Region ID cannot be empty"));
    }
    if request.name.is_empty() {
        return Err(ApiError::invalid_input("Region name cannot be empty"));
    }
    
    // Map API request to service params
    let params = CreateBrainRegionParams {
        region_id: request.region_id.clone(),
        name: request.name,
        region_type: request.region_type,
        parent_id: request.parent_id,
        properties: None,
    };
    
    // Create brain region via service
    let created = connectome_service
        .create_brain_region(params)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::AlreadyExists { .. } => {
                ApiError::conflict(format!("Brain region '{}' already exists", request.region_id))
            }
            feagi_services::ServiceError::InvalidInput(msg) => {
                ApiError::invalid_input(msg)
            }
            feagi_services::ServiceError::NotFound { resource, id } => {
                ApiError::not_found(&resource, &id).with_details("Parent region not found")
            }
            _ => ApiError::internal(&format!("Failed to create brain region: {}", e)),
        })?;
    
    // Return created region (use get_brain_region to get full details)
    get_brain_region(auth_ctx, connectome_service, created.region_id).await
}

/// Delete a brain region
///
/// Deletes the specified brain region.
#[utoipa::path(
    delete,
    path = "/api/v1/brain-regions/{id}",
    tag = "Brain Regions",
    params(
        ("id" = String, Path, description = "Brain region ID")
    ),
    responses(
        (status = 200, description = "Brain region deleted successfully"),
        (status = 404, description = "Brain region not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn delete_brain_region(
    _auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    region_id: String,
) -> ApiResult<()> {
    // Delete brain region via service
    connectome_service
        .delete_brain_region(&region_id)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Brain region", &region_id)
            }
            _ => ApiError::internal(&format!("Failed to delete brain region: {}", e)),
        })?;
    
    Ok(())
}


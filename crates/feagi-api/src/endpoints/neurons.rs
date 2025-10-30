// Neuron endpoints (transport-agnostic)
//
// These endpoints provide operations for neuron management.

use std::sync::Arc;
use utoipa;

use crate::{
    common::{ApiError, ApiResult},
    security::AuthContext,
    v1::{CreateNeuronRequest, NeuronInfoResponse, NeuronListResponse, NeuronCountResponse},
};
use feagi_services::{NeuronService, CreateNeuronParams};

/// List neurons in a cortical area
///
/// Returns a list of neurons in the specified cortical area.
#[utoipa::path(
    get,
    path = "/api/v1/neurons",
    tag = "Neurons",
    params(
        ("cortical_area" = String, Query, description = "Cortical area ID to filter by"),
        ("limit" = Option<usize>, Query, description = "Maximum number of neurons to return")
    ),
    responses(
        (status = 200, description = "Neurons retrieved successfully", body = NeuronListResponse),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn list_neurons(
    _auth_ctx: &AuthContext,
    neuron_service: Arc<dyn NeuronService + Send + Sync>,
    cortical_area: String,
    limit: Option<usize>,
) -> ApiResult<NeuronListResponse> {
    // Get neurons from neuron service
    let neurons = neuron_service
        .list_neurons_in_area(&cortical_area, limit)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Cortical area", &cortical_area)
            }
            _ => ApiError::internal(&format!("Failed to list neurons: {}", e)),
        })?;
    
    // Map service DTOs to API DTOs
    let neuron_responses: Vec<NeuronInfoResponse> = neurons
        .iter()
        .map(|neuron| NeuronInfoResponse {
            neuron_id: neuron.id,
            cortical_area: neuron.cortical_id.clone(),
            coordinates: [neuron.coordinates.0, neuron.coordinates.1, neuron.coordinates.2],
            membrane_potential: neuron
                .properties
                .get("membrane_potential")
                .and_then(|v| v.as_f64())
                .unwrap_or(-70.0) as f32,
            is_firing: neuron
                .properties
                .get("is_firing")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            synaptic_inputs: neuron
                .properties
                .get("synaptic_inputs")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
            synaptic_outputs: neuron
                .properties
                .get("synaptic_outputs")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
        })
        .collect();
    
    Ok(NeuronListResponse {
        total_count: neuron_responses.len(),
        neurons: neuron_responses,
        cortical_area: Some(cortical_area),
    })
}

/// Get neuron by ID
///
/// Returns detailed information about a specific neuron.
#[utoipa::path(
    get,
    path = "/api/v1/neurons/{id}",
    tag = "Neurons",
    params(
        ("id" = u64, Path, description = "Neuron ID")
    ),
    responses(
        (status = 200, description = "Neuron retrieved successfully", body = NeuronInfoResponse),
        (status = 404, description = "Neuron not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_neuron(
    _auth_ctx: &AuthContext,
    neuron_service: Arc<dyn NeuronService + Send + Sync>,
    neuron_id: u64,
) -> ApiResult<NeuronInfoResponse> {
    // Get neuron from neuron service
    let neuron = neuron_service
        .get_neuron(neuron_id)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Neuron", &neuron_id.to_string())
            }
            _ => ApiError::internal(&format!("Failed to get neuron: {}", e)),
        })?;
    
    // Map service DTO to API DTO
    Ok(NeuronInfoResponse {
        neuron_id: neuron.id,
        cortical_area: neuron.cortical_id.clone(),
        coordinates: [neuron.coordinates.0, neuron.coordinates.1, neuron.coordinates.2],
        membrane_potential: neuron
            .properties
            .get("membrane_potential")
            .and_then(|v| v.as_f64())
            .unwrap_or(-70.0) as f32,
        is_firing: neuron
            .properties
            .get("is_firing")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        synaptic_inputs: neuron
            .properties
            .get("synaptic_inputs")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        synaptic_outputs: neuron
            .properties
            .get("synaptic_outputs")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
    })
}

/// Create a neuron
///
/// Creates a new neuron at the specified coordinates within a cortical area.
#[utoipa::path(
    post,
    path = "/api/v1/neurons",
    tag = "Neurons",
    request_body = CreateNeuronRequest,
    responses(
        (status = 201, description = "Neuron created successfully", body = NeuronInfoResponse),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 409, description = "Neuron already exists at coordinates", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn create_neuron(
    auth_ctx: &AuthContext,
    neuron_service: Arc<dyn NeuronService + Send + Sync>,
    request: CreateNeuronRequest,
) -> ApiResult<NeuronInfoResponse> {
    // Validate input
    if request.cortical_area.is_empty() {
        return Err(ApiError::invalid_input("Cortical area ID cannot be empty"));
    }
    
    // Map API request to service params
    let params = CreateNeuronParams {
        cortical_id: request.cortical_area.clone(),
        coordinates: (request.coordinates[0], request.coordinates[1], request.coordinates[2]),
        properties: None,
    };
    
    // Create neuron via service
    let created = neuron_service
        .create_neuron(params)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { resource, id } => {
                ApiError::not_found(&resource, &id)
            }
            feagi_services::ServiceError::AlreadyExists { .. } => {
                ApiError::conflict(format!(
                    "Neuron already exists at coordinates {:?} in cortical area '{}'",
                    request.coordinates, request.cortical_area
                ))
            }
            feagi_services::ServiceError::InvalidInput(msg) => {
                ApiError::invalid_input(msg)
            }
            _ => ApiError::internal(&format!("Failed to create neuron: {}", e)),
        })?;
    
    // Return created neuron (use get_neuron to get full details)
    get_neuron(auth_ctx, neuron_service, created.id).await
}

/// Delete a neuron
///
/// Deletes the specified neuron by ID.
#[utoipa::path(
    delete,
    path = "/api/v1/neurons/{id}",
    tag = "Neurons",
    params(
        ("id" = u64, Path, description = "Neuron ID")
    ),
    responses(
        (status = 200, description = "Neuron deleted successfully"),
        (status = 404, description = "Neuron not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn delete_neuron(
    _auth_ctx: &AuthContext,
    neuron_service: Arc<dyn NeuronService + Send + Sync>,
    neuron_id: u64,
) -> ApiResult<()> {
    // Delete neuron via service
    neuron_service
        .delete_neuron(neuron_id)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Neuron", &neuron_id.to_string())
            }
            _ => ApiError::internal(&format!("Failed to delete neuron: {}", e)),
        })?;
    
    Ok(())
}

/// Get neuron count in cortical area
///
/// Returns the number of neurons in the specified cortical area.
#[utoipa::path(
    get,
    path = "/api/v1/neurons/count",
    tag = "Neurons",
    params(
        ("cortical_area" = String, Query, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Neuron count retrieved successfully", body = NeuronCountResponse),
        (status = 404, description = "Cortical area not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_neuron_count(
    _auth_ctx: &AuthContext,
    neuron_service: Arc<dyn NeuronService + Send + Sync>,
    cortical_area: String,
) -> ApiResult<NeuronCountResponse> {
    // Get neuron count from neuron service
    let count = neuron_service
        .get_neuron_count(&cortical_area)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::NotFound { .. } => {
                ApiError::not_found("Cortical area", &cortical_area)
            }
            _ => ApiError::internal(&format!("Failed to get neuron count: {}", e)),
        })?;
    
    Ok(NeuronCountResponse {
        cortical_area,
        neuron_count: count,
    })
}


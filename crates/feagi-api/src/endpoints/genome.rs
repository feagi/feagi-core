// Genome endpoints (transport-agnostic)
//
// These endpoints provide genome management operations (load, save, validate).

use std::sync::Arc;
use utoipa;

use crate::{
    common::{ApiError, ApiResult},
    security::AuthContext,
    v1::{
        GenomeInfoResponse, LoadGenomeRequest, SaveGenomeRequest, SaveGenomeResponse,
        ValidateGenomeRequest, ValidateGenomeResponse,
    },
};
use feagi_services::{GenomeService, LoadGenomeParams, SaveGenomeParams};

/// Get current genome information
///
/// Returns metadata about the currently loaded genome.
#[utoipa::path(
    get,
    path = "/api/v1/genome",
    tag = "Genome",
    responses(
        (status = 200, description = "Genome info retrieved successfully", body = GenomeInfoResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_genome_info(
    _auth_ctx: &AuthContext,
    genome_service: Arc<dyn GenomeService + Send + Sync>,
) -> ApiResult<GenomeInfoResponse> {
    // Get genome info from service
    let genome_info = genome_service
        .get_genome_info()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get genome info: {}", e)))?;
    
    // Map service DTO to API DTO
    Ok(GenomeInfoResponse {
        genome_id: Some(genome_info.genome_id),
        title: Some(genome_info.genome_title),
        version: Some(genome_info.version),
        cortical_area_count: genome_info.cortical_area_count,
        brain_region_count: genome_info.brain_region_count,
        created_at: None,  // TODO: Add timestamps to GenomeInfo in feagi-services
        modified_at: None,
    })
}

/// Load a genome
///
/// Loads a genome from JSON, creating all cortical areas and brain regions.
#[utoipa::path(
    post,
    path = "/api/v1/genome/load",
    tag = "Genome",
    request_body = LoadGenomeRequest,
    responses(
        (status = 200, description = "Genome loaded successfully", body = GenomeInfoResponse),
        (status = 400, description = "Invalid genome JSON", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn load_genome(
    _auth_ctx: &AuthContext,
    genome_service: Arc<dyn GenomeService + Send + Sync>,
    request: LoadGenomeRequest,
) -> ApiResult<GenomeInfoResponse> {
    // Validate input
    if request.genome_json.is_empty() {
        return Err(ApiError::invalid_input("Genome JSON cannot be empty"));
    }
    
    // Reset connectome if requested
    if request.reset_before_load {
        genome_service
            .reset_connectome()
            .await
            .map_err(|e| ApiError::internal(&format!("Failed to reset connectome: {}", e)))?;
    }
    
    // Load genome via service
    let params = LoadGenomeParams {
        json_str: request.genome_json,
    };
    
    let genome_info = genome_service
        .load_genome(params)
        .await
        .map_err(|e| match e {
            feagi_services::ServiceError::InvalidInput(msg) => {
                ApiError::invalid_input(msg)
            }
            _ => ApiError::internal(&format!("Failed to load genome: {}", e)),
        })?;
    
    // Map service DTO to API DTO
    Ok(GenomeInfoResponse {
        genome_id: Some(genome_info.genome_id),
        title: Some(genome_info.genome_title),
        version: Some(genome_info.version),
        cortical_area_count: genome_info.cortical_area_count,
        brain_region_count: genome_info.brain_region_count,
        created_at: None,
        modified_at: None,
    })
}

/// Save the current genome
///
/// Serializes the current brain state to genome JSON format.
#[utoipa::path(
    post,
    path = "/api/v1/genome/save",
    tag = "Genome",
    request_body = SaveGenomeRequest,
    responses(
        (status = 200, description = "Genome saved successfully", body = SaveGenomeResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn save_genome(
    _auth_ctx: &AuthContext,
    genome_service: Arc<dyn GenomeService + Send + Sync>,
    request: SaveGenomeRequest,
) -> ApiResult<SaveGenomeResponse> {
    // Save genome via service
    let params = SaveGenomeParams {
        genome_id: request.genome_id,
        genome_title: request.title,
    };
    
    let genome_json = genome_service
        .save_genome(params)
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to save genome: {}", e)))?;
    
    // Get updated genome info
    let genome_info = genome_service
        .get_genome_info()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get genome info: {}", e)))?;
    
    Ok(SaveGenomeResponse {
        genome_json,
        genome_info: GenomeInfoResponse {
            genome_id: Some(genome_info.genome_id),
            title: Some(genome_info.genome_title),
            version: Some(genome_info.version),
            cortical_area_count: genome_info.cortical_area_count,
            brain_region_count: genome_info.brain_region_count,
            created_at: None,
            modified_at: None,
        },
    })
}

/// Validate a genome JSON
///
/// Validates genome JSON without loading it into the connectome.
#[utoipa::path(
    post,
    path = "/api/v1/genome/validate",
    tag = "Genome",
    request_body = ValidateGenomeRequest,
    responses(
        (status = 200, description = "Validation result", body = ValidateGenomeResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn validate_genome(
    _auth_ctx: &AuthContext,
    genome_service: Arc<dyn GenomeService + Send + Sync>,
    request: ValidateGenomeRequest,
) -> ApiResult<ValidateGenomeResponse> {
    // Validate genome via service
    match genome_service
        .validate_genome(request.genome_json)
        .await
    {
        Ok(is_valid) => Ok(ValidateGenomeResponse {
            is_valid,
            errors: None,
        }),
        Err(e) => match e {
            feagi_services::ServiceError::InvalidInput(msg) => {
                Ok(ValidateGenomeResponse {
                    is_valid: false,
                    errors: Some(vec![msg]),
                })
            }
            _ => Err(ApiError::internal(&format!("Failed to validate genome: {}", e))),
        },
    }
}

/// Reset the connectome
///
/// Clears all cortical areas and brain regions from the connectome.
#[utoipa::path(
    post,
    path = "/api/v1/genome/reset",
    tag = "Genome",
    responses(
        (status = 200, description = "Connectome reset successfully"),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn reset_connectome(
    _auth_ctx: &AuthContext,
    genome_service: Arc<dyn GenomeService + Send + Sync>,
) -> ApiResult<()> {
    // Reset connectome via service
    genome_service
        .reset_connectome()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to reset connectome: {}", e)))?;
    
    Ok(())
}

/// Load the barebones genome
///
/// Loads the pre-configured barebones genome template from the defaults directory.
/// This is a minimal genome with only core areas.
#[utoipa::path(
    post,
    path = "/api/v1/genome/upload/barebones",
    tag = "Genome",
    responses(
        (status = 200, description = "Barebones genome loaded successfully", body = GenomeInfoResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn load_barebones_genome(
    _auth_ctx: &AuthContext,
    genome_service: Arc<dyn GenomeService + Send + Sync>,
) -> ApiResult<GenomeInfoResponse> {
    // Load barebones genome from defaults
    let genome_json = load_default_genome_file("barebones")?;
    
    // Use the load_genome service method
    let params = feagi_services::LoadGenomeParams {
        json_str: genome_json,
    };
    
    genome_service
        .load_genome(params)
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to load barebones genome: {}", e)))?;
    
    // Get updated genome info
    let genome_info = genome_service
        .get_genome_info()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get genome info: {}", e)))?;
    
    // Map service DTO to API DTO
    Ok(GenomeInfoResponse {
        genome_id: Some(genome_info.genome_id),
        title: Some(genome_info.genome_title),
        version: Some(genome_info.version),
        cortical_area_count: genome_info.cortical_area_count,
        brain_region_count: genome_info.brain_region_count,
        created_at: None,
        modified_at: None,
    })
}

/// Load the essential genome
///
/// Loads the pre-configured essential genome template from the defaults directory.
/// This genome includes basic sensory and motor areas for general-purpose use.
#[utoipa::path(
    post,
    path = "/api/v1/genome/upload/essential",
    tag = "Genome",
    responses(
        (status = 200, description = "Essential genome loaded successfully", body = GenomeInfoResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn load_essential_genome(
    _auth_ctx: &AuthContext,
    genome_service: Arc<dyn GenomeService + Send + Sync>,
) -> ApiResult<GenomeInfoResponse> {
    // Load essential genome from defaults
    let genome_json = load_default_genome_file("essential")?;
    
    // Use the load_genome service method
    let params = feagi_services::LoadGenomeParams {
        json_str: genome_json,
    };
    
    genome_service
        .load_genome(params)
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to load essential genome: {}", e)))?;
    
    // Get updated genome info
    let genome_info = genome_service
        .get_genome_info()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to get genome info: {}", e)))?;
    
    // Map service DTO to API DTO
    Ok(GenomeInfoResponse {
        genome_id: Some(genome_info.genome_id),
        title: Some(genome_info.genome_title),
        version: Some(genome_info.version),
        cortical_area_count: genome_info.cortical_area_count,
        brain_region_count: genome_info.brain_region_count,
        created_at: None,
        modified_at: None,
    })
}

/// Helper function to load default genome files from the defaults directory
fn load_default_genome_file(genome_name: &str) -> ApiResult<String> {
    use std::path::PathBuf;
    use std::fs;
    
    let filename = format!("{}_genome.json", genome_name);
    
    // Search paths for genome files (in order of preference)
    let mut search_paths = vec![
        PathBuf::from("../feagi-py/feagi/evo/defaults/genome"),  // Relative to feagi binary
        PathBuf::from("feagi-py/feagi/evo/defaults/genome"),      // From workspace root
        PathBuf::from("feagi/evo/defaults/genome"),                // Alt location
    ];
    
    // Also check FEAGI_GENOME_PATH environment variable
    if let Ok(genome_path) = std::env::var("FEAGI_GENOME_PATH") {
        search_paths.insert(0, PathBuf::from(genome_path));
    }
    
    // Find the genome file
    for base_path in &search_paths {
        let genome_path = base_path.join(&filename);
        if genome_path.exists() {
            // Read and return the genome JSON
            return fs::read_to_string(&genome_path)
                .map_err(|e| ApiError::internal(&format!("Failed to read genome file {}: {}", filename, e)));
        }
    }
    
    Err(ApiError::not_found(
        "Default genome",
        &format!("{} (searched in: {:?})", genome_name, search_paths)
    ))
}


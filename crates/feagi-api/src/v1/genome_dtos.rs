// Genome DTOs for V1 API
//
// These DTOs must match Python FastAPI response structures exactly.

use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

/// Genome information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "genome_id": "human_cortex_v1",
    "title": "Human Cortex Model v1",
    "version": "1.0.0",
    "cortical_area_count": 52,
    "brain_region_count": 12,
    "created_at": "2025-01-15T10:30:00Z",
    "modified_at": "2025-01-20T14:45:00Z"
}))]
pub struct GenomeInfoResponse {
    /// Genome ID
    pub genome_id: Option<String>,
    
    /// Human-readable title
    pub title: Option<String>,
    
    /// Genome version
    pub version: Option<String>,
    
    /// Number of cortical areas
    pub cortical_area_count: usize,
    
    /// Number of brain regions
    pub brain_region_count: usize,
    
    /// Creation timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    
    /// Last modification timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

/// Load genome request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoadGenomeRequest {
    /// Genome JSON string
    pub genome_json: String,
    
    /// Whether to reset connectome before loading
    #[serde(default)]
    pub reset_before_load: bool,
}

/// Save genome request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SaveGenomeRequest {
    /// Optional: Genome ID to assign
    #[serde(default)]
    pub genome_id: Option<String>,
    
    /// Optional: Human-readable title
    #[serde(default)]
    pub title: Option<String>,
}

/// Save genome response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SaveGenomeResponse {
    /// Genome JSON string
    pub genome_json: String,
    
    /// Genome metadata
    pub genome_info: GenomeInfoResponse,
}

/// Validate genome request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidateGenomeRequest {
    /// Genome JSON string to validate
    pub genome_json: String,
}

/// Validate genome response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidateGenomeResponse {
    /// Whether the genome is valid
    pub is_valid: bool,
    
    /// Validation errors (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}



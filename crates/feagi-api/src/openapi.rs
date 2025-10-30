// OpenAPI documentation generation
//
// This module generates the OpenAPI 3.0 specification at compile-time
// using utoipa, ensuring the documentation stays in sync with the code.

use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, ApiKey, ApiKeyValue};

use crate::{
    common::{ApiError, ApiResponse},
    v1::{
        // Health DTOs
        HealthCheckResponseV1, ReadinessCheckResponseV1, ComponentReadiness,
        
        // Cortical area DTOs
        CorticalAreaSummary, CorticalAreaDetail, CorticalAreaListResponse,
        CreateCorticalAreaRequest, UpdateCorticalAreaRequest,
        Coordinates3D, Dimensions3D,
        
        // Brain region DTOs
        BrainRegionSummary, BrainRegionDetail, BrainRegionListResponse,
        CreateBrainRegionRequest,
        
        // Genome DTOs
        GenomeInfoResponse, LoadGenomeRequest, SaveGenomeRequest,
        SaveGenomeResponse, ValidateGenomeRequest, ValidateGenomeResponse,
        
        // Neuron DTOs
        NeuronInfoResponse, CreateNeuronRequest, NeuronListResponse,
        NeuronCountResponse,
        
        // Runtime DTOs
        RuntimeStatusResponse, SetFrequencyRequest, BurstCountResponse,
        
        // Analytics DTOs
        SystemHealthResponse, CorticalAreaStatsResponse, ConnectivityStatsResponse,
        ConnectomeAnalyticsResponse, PopulatedAreasResponse, PopulatedAreaInfo,
        NeuronDensityResponse,
    },
};

/// OpenAPI documentation for FEAGI REST API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "FEAGI REST API",
        version = "1.0.0",
        description = "Comprehensive REST API for FEAGI (Foundational Engine for Artificial General Intelligence)",
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        ),
        contact(
            name = "FEAGI Team",
            url = "https://feagi.org",
            email = "contact@feagi.org"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "http://localhost:8000", description = "Python FastAPI compatibility")
    ),
    paths(
        // Health endpoints
        crate::endpoints::health::health_check,
        crate::endpoints::health::readiness_check,
        
        // Cortical area endpoints
        crate::endpoints::cortical_areas::list_cortical_areas,
        crate::endpoints::cortical_areas::get_cortical_area,
        crate::endpoints::cortical_areas::create_cortical_area,
        crate::endpoints::cortical_areas::update_cortical_area,
        crate::endpoints::cortical_areas::delete_cortical_area,
        
        // Brain region endpoints
        crate::endpoints::brain_regions::list_brain_regions,
        crate::endpoints::brain_regions::get_brain_region,
        crate::endpoints::brain_regions::create_brain_region,
        crate::endpoints::brain_regions::delete_brain_region,
        
        // Genome endpoints
        crate::endpoints::genome::get_genome_info,
        crate::endpoints::genome::load_genome,
        crate::endpoints::genome::save_genome,
        crate::endpoints::genome::validate_genome,
        crate::endpoints::genome::reset_connectome,
        crate::endpoints::genome::load_barebones_genome,
        crate::endpoints::genome::load_essential_genome,
        
        // Neuron endpoints
        crate::endpoints::neurons::list_neurons,
        crate::endpoints::neurons::get_neuron,
        crate::endpoints::neurons::create_neuron,
        crate::endpoints::neurons::delete_neuron,
        crate::endpoints::neurons::get_neuron_count,
        
        // Runtime endpoints
        crate::endpoints::runtime::get_runtime_status,
        crate::endpoints::runtime::start_runtime,
        crate::endpoints::runtime::stop_runtime,
        crate::endpoints::runtime::pause_runtime,
        crate::endpoints::runtime::resume_runtime,
        crate::endpoints::runtime::step_runtime,
        crate::endpoints::runtime::set_frequency,
        crate::endpoints::runtime::get_burst_count,
        crate::endpoints::runtime::reset_burst_count,
        
        // Analytics endpoints
        crate::endpoints::analytics::get_system_health,
        crate::endpoints::analytics::get_cortical_area_stats,
        crate::endpoints::analytics::get_all_cortical_area_stats,
        crate::endpoints::analytics::get_connectivity_stats,
        crate::endpoints::analytics::get_connectome_stats,
        crate::endpoints::analytics::get_populated_areas,
        crate::endpoints::analytics::get_neuron_density,
    ),
    components(
        schemas(
            // Common
            ApiError,
            
            // Health
            HealthCheckResponseV1,
            ReadinessCheckResponseV1,
            ComponentReadiness,
            
            // Cortical Areas
            CorticalAreaSummary,
            CorticalAreaDetail,
            CorticalAreaListResponse,
            CreateCorticalAreaRequest,
            UpdateCorticalAreaRequest,
            Coordinates3D,
            Dimensions3D,
            
            // Brain Regions
            BrainRegionSummary,
            BrainRegionDetail,
            BrainRegionListResponse,
            CreateBrainRegionRequest,
            
            // Genome
            GenomeInfoResponse,
            LoadGenomeRequest,
            SaveGenomeRequest,
            SaveGenomeResponse,
            ValidateGenomeRequest,
            ValidateGenomeResponse,
            
            // Neurons
            NeuronInfoResponse,
            CreateNeuronRequest,
            NeuronListResponse,
            NeuronCountResponse,
            
            // Runtime
            RuntimeStatusResponse,
            SetFrequencyRequest,
            BurstCountResponse,
            
            // Analytics
            SystemHealthResponse,
            CorticalAreaStatsResponse,
            ConnectivityStatsResponse,
            ConnectomeAnalyticsResponse,
            PopulatedAreasResponse,
            PopulatedAreaInfo,
            NeuronDensityResponse,
        )
    ),
    tags(
        (name = "Health", description = "System health and readiness endpoints"),
        (name = "Cortical Areas", description = "Cortical area management (CRUD)"),
        (name = "Brain Regions", description = "Brain region management (CRUD)"),
        (name = "Genome", description = "Genome operations (load, save, validate)"),
        (name = "Analytics", description = "System analytics and metrics"),
        (name = "Agents", description = "Agent registration and heartbeat"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Security scheme configuration
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            // API Key authentication (for future use)
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
            
            // JWT Bearer authentication (for future use)
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// Get OpenAPI JSON specification
pub fn get_openapi_json() -> String {
    ApiDoc::openapi().to_pretty_json().unwrap_or_else(|e| {
        format!(r#"{{"error": "Failed to generate OpenAPI spec: {}"}}"#, e)
    })
}

/// Get OpenAPI YAML specification
pub fn get_openapi_yaml() -> String {
    // utoipa supports YAML output, but we need to implement it
    // For now, return a notice
    "# OpenAPI YAML generation not yet implemented\n# Use /openapi.json instead".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_generation() {
        let json = get_openapi_json();
        assert!(json.contains("FEAGI REST API"));
        assert!(json.contains("health"));
        assert!(json.contains("ready"));
    }

    #[test]
    fn test_openapi_components() {
        let openapi = ApiDoc::openapi();
        assert!(openapi.components.is_some());
        
        let components = openapi.components.unwrap();
        assert!(components.schemas.contains_key("HealthCheckResponseV1"));
        assert!(components.schemas.contains_key("ApiError"));
    }

    #[test]
    fn test_security_schemes() {
        let openapi = ApiDoc::openapi();
        let components = openapi.components.unwrap();
        
        assert!(components.security_schemes.contains_key("api_key"));
        assert!(components.security_schemes.contains_key("bearer_auth"));
    }
}


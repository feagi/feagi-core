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
        // Agent endpoints
        crate::endpoints::agent::register_agent,
        crate::endpoints::agent::heartbeat,
        crate::endpoints::agent::list_agents,
        crate::endpoints::agent::get_agent_properties,
        crate::endpoints::agent::get_shared_memory,
        crate::endpoints::agent::deregister_agent,
        crate::endpoints::agent::manual_stimulation,
        
        // System endpoints
        crate::endpoints::system::get_health_check,
        crate::endpoints::system::get_cortical_area_visualization_skip_rate,
        crate::endpoints::system::set_cortical_area_visualization_skip_rate,
        crate::endpoints::system::get_cortical_area_visualization_suppression_threshold,
        crate::endpoints::system::set_cortical_area_visualization_suppression_threshold,
        
        // Cortical Area endpoints
        crate::endpoints::cortical_area::get_ipu,
        crate::endpoints::cortical_area::get_opu,
        crate::endpoints::cortical_area::get_cortical_area_id_list,
        crate::endpoints::cortical_area::get_cortical_area_name_list,
        crate::endpoints::cortical_area::get_cortical_id_name_mapping,
        crate::endpoints::cortical_area::get_cortical_types,
        crate::endpoints::cortical_area::get_cortical_map_detailed,
        crate::endpoints::cortical_area::get_cortical_locations_2d,
        crate::endpoints::cortical_area::get_cortical_area_geometry,
        crate::endpoints::cortical_area::get_cortical_visibility,
        crate::endpoints::cortical_area::post_cortical_name_location,
        crate::endpoints::cortical_area::post_cortical_area_properties,
        crate::endpoints::cortical_area::post_multi_cortical_area_properties,
        crate::endpoints::cortical_area::post_cortical_area,
        crate::endpoints::cortical_area::put_cortical_area,
        crate::endpoints::cortical_area::delete_cortical_area,
        crate::endpoints::cortical_area::post_custom_cortical_area,
        crate::endpoints::cortical_area::post_clone,
        crate::endpoints::cortical_area::put_multi_cortical_area,
        crate::endpoints::cortical_area::delete_multi_cortical_area,
        crate::endpoints::cortical_area::put_coord_2d,
        crate::endpoints::cortical_area::put_suppress_cortical_visibility,
        crate::endpoints::cortical_area::put_reset,
        
        // Morphology endpoints
        crate::endpoints::morphology::get_morphology_list,
        crate::endpoints::morphology::get_morphology_types,
        crate::endpoints::morphology::get_list_types,
        crate::endpoints::morphology::get_morphologies,
        crate::endpoints::morphology::post_morphology,
        crate::endpoints::morphology::put_morphology,
        crate::endpoints::morphology::delete_morphology,
        crate::endpoints::morphology::post_morphology_properties,
        crate::endpoints::morphology::post_morphology_usage,
        
        // Genome endpoints
        crate::endpoints::genome::get_file_name,
        crate::endpoints::genome::get_circuits,
        crate::endpoints::genome::post_amalgamation_destination,
        crate::endpoints::genome::delete_amalgamation_cancellation,
        crate::endpoints::genome::post_genome_append,
        crate::endpoints::genome::post_upload_barebones_genome,
        crate::endpoints::genome::post_upload_essential_genome,
        
        // Cortical Mapping endpoints
        crate::endpoints::cortical_mapping::post_afferents,
        crate::endpoints::cortical_mapping::post_efferents,
        crate::endpoints::cortical_mapping::post_mapping_properties,
        crate::endpoints::cortical_mapping::put_mapping_properties,
        
        // Region endpoints
        crate::endpoints::region::get_regions_members,
        crate::endpoints::region::post_region,
        crate::endpoints::region::put_region,
        crate::endpoints::region::delete_region,
        crate::endpoints::region::post_clone,
        crate::endpoints::region::put_relocate_members,
        crate::endpoints::region::delete_region_and_members,
        
        // Connectome endpoints
        crate::endpoints::connectome::get_cortical_areas_list_detailed,
        crate::endpoints::connectome::get_properties_dimensions,
        crate::endpoints::connectome::get_properties_mappings,
        
        // Burst Engine endpoints
        crate::endpoints::burst_engine::get_simulation_timestep,
        crate::endpoints::burst_engine::post_simulation_timestep,
        
        // Insight endpoints
        crate::endpoints::insight::post_neurons_membrane_potential_status,
        crate::endpoints::insight::post_neuron_synaptic_potential_status,
        crate::endpoints::insight::post_neurons_membrane_potential_set,
        crate::endpoints::insight::post_neuron_synaptic_potential_set,
        
        // Neuroplasticity endpoints
        crate::endpoints::neuroplasticity::get_plasticity_queue_depth,
        crate::endpoints::neuroplasticity::put_plasticity_queue_depth,
        
        // Input endpoints
        crate::endpoints::input::get_vision,
        crate::endpoints::input::post_vision,
    ),
    components(
        schemas(
            // Common
            ApiError,
            
            // Health
            HealthCheckResponseV1,
            ReadinessCheckResponseV1,
            ComponentReadiness,
            
            // Agent
            crate::v1::AgentRegistrationRequest,
            crate::v1::AgentRegistrationResponse,
            crate::v1::HeartbeatRequest,
            crate::v1::HeartbeatResponse,
            crate::v1::AgentListResponse,
            crate::v1::AgentPropertiesResponse,
            crate::v1::AgentDeregistrationRequest,
            crate::v1::SuccessResponse,
            crate::v1::ManualStimulationRequest,
            crate::v1::ManualStimulationResponse,
            
            // System
            crate::endpoints::system::HealthCheckResponse,
            
            // Cortical Area
            crate::endpoints::cortical_area::CorticalAreaIdListResponse,
            crate::endpoints::cortical_area::CorticalAreaNameListResponse,
            
            // Morphology
            crate::endpoints::morphology::MorphologyListResponse,
            
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
        (name = "agent", description = "Agent registration and heartbeat"),
        (name = "system", description = "System health and configuration"),
        (name = "cortical_area", description = "Cortical area management"),
        (name = "morphology", description = "Morphology management"),
        (name = "genome", description = "Genome operations"),
        (name = "cortical_mapping", description = "Cortical mapping operations"),
        (name = "region", description = "Brain region management"),
        (name = "connectome", description = "Connectome operations"),
        (name = "burst_engine", description = "Burst engine configuration"),
        (name = "insight", description = "Neuron insight operations"),
        (name = "neuroplasticity", description = "Neuroplasticity configuration"),
        (name = "input", description = "Input operations"),
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


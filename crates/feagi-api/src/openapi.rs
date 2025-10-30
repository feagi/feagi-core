// OpenAPI documentation generation
//
// This module generates the OpenAPI 3.0 specification at compile-time
// using utoipa, ensuring the documentation stays in sync with the code.

use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, ApiKey, ApiKeyValue};

use crate::{
    common::{ApiError, ApiResponse},
    v1::dtos::{HealthCheckResponseV1, ReadinessCheckResponseV1, ComponentReadiness},
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
        crate::endpoints::health::health_check,
        crate::endpoints::health::readiness_check,
    ),
    components(
        schemas(
            HealthCheckResponseV1,
            ReadinessCheckResponseV1,
            ComponentReadiness,
            ApiError,
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


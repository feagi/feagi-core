// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// CORS middleware for HTTP API

use tower_http::cors::{Any, CorsLayer};

/// Create CORS layer with permissive settings
/// 
/// This allows requests from any origin, which is appropriate for development
/// and internal FEAGI deployments. For production, this should be configured
/// via TOML config to restrict allowed origins.
pub fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false)
}

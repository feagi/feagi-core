// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Evolution API DTOs
//! 
//! Request/response types for evolutionary algorithms

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Evolution status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvolutionStatusResponse {
    pub active: bool,
    pub generation: u64,
    pub population_size: usize,
}

/// Evolution configuration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvolutionConfigRequest {
    pub config: HashMap<String, serde_json::Value>,
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvolutionSuccessResponse {
    pub message: String,
    pub success: bool,
}


// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Training API DTOs
//! 
//! Request/response types for reinforcement learning and training

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Shock configuration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShockConfigRequest {
    pub shock: Vec<String>,
}

/// Shock options response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShockOptionsResponse {
    pub options: Vec<String>,
}

/// Shock status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShockStatusResponse {
    pub active: bool,
    pub scenarios: Vec<String>,
}

/// Intensity configuration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IntensityRequest {
    pub intensity: f64,
}

/// Brain fitness response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BrainFitnessResponse {
    pub fitness: f64,
}

/// Fitness criteria response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FitnessCriteriaResponse {
    pub criteria: HashMap<String, f64>,
}

/// Fitness criteria update request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FitnessCriteriaUpdateRequest {
    pub criteria: HashMap<String, f64>,
}

/// Fitness statistics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FitnessStatsResponse {
    pub stats: HashMap<String, serde_json::Value>,
}

/// Training report response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrainingReportResponse {
    pub report: HashMap<String, serde_json::Value>,
}

/// Training status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrainingStatusResponse {
    pub active: bool,
    pub mode: String,
}

/// Training statistics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrainingStatsResponse {
    pub total_episodes: u64,
    pub total_rewards: f64,
}

/// Training configuration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrainingConfigRequest {
    pub config: HashMap<String, serde_json::Value>,
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrainingSuccessResponse {
    pub message: String,
    pub success: bool,
}



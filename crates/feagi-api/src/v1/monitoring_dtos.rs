// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Monitoring API DTOs
//! 
//! Request/response types for system monitoring and metrics

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Monitoring system status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitoringStatusResponse {
    pub enabled: bool,
    pub metrics_collected: usize,
    pub brain_readiness: bool,
    pub burst_engine_active: bool,
}

/// System metrics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemMetricsResponse {
    pub burst_frequency_hz: f64,
    pub burst_count: u64,
    pub neuron_count: usize,
    pub cortical_area_count: usize,
    pub brain_readiness: bool,
    pub burst_engine_active: bool,
}

/// Detailed monitoring data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitoringData {
    pub neuron_count: usize,
    pub cortical_area_count: usize,
    pub burst_count: u64,
    pub brain_readiness: bool,
    pub burst_engine_active: bool,
}

/// Monitoring data response with timestamp
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitoringDataResponse {
    pub data: MonitoringData,
    pub timestamp: String,
}


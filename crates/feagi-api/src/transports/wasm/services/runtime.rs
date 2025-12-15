// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Runtime Service (stub)

use async_trait::async_trait;
use feagi_services::traits::runtime_service::RuntimeService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;
use std::sync::Arc;

pub struct WasmRuntimeService {
    // TODO: Add NPU reference if needed for burst count, etc.
}

impl WasmRuntimeService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RuntimeService for WasmRuntimeService {
    async fn get_status(&self) -> ServiceResult<RuntimeStatus> {
        Ok(RuntimeStatus {
            is_running: true,
            is_paused: false,
            frequency_hz: 0.0, // TODO: Get from NPU if available
            burst_count: 0, // TODO: Get from NPU if available
            current_rate_hz: 0.0, // TODO: Get from NPU if available
            last_burst_neuron_count: 0, // TODO: Get from NPU if available
            avg_burst_time_ms: 0.0, // TODO: Get from NPU if available
        })
    }

    async fn get_burst_count(&self) -> ServiceResult<u64> {
        Ok(0) // TODO: Get from NPU if available
    }

    async fn set_frequency(&self, _frequency: f64) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented("WASM mode frequency control not yet implemented".to_string()))
    }

    async fn get_fcl_snapshot_with_cortical_idx(&self) -> ServiceResult<Vec<(u64, u32, f32)>> {
        Ok(vec![]) // TODO: Get from NPU if available
    }
}


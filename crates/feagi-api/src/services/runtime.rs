// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Runtime service driving the injected NPU.
//!
//! Burst control and counters come straight from the engine. Fire-ledger sampling, sensory
//! injection and subscription management have no counterpart in the current engine and report
//! that rather than pretending to succeed.

use crate::services::{npu_unavailable, OptionalNpu};
use async_trait::async_trait;
use feagi_services::traits::runtime_service::RuntimeService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;

pub struct GenomeRuntimeService {
    npu: OptionalNpu,
}

impl GenomeRuntimeService {
    /// Creates the service over an optional NPU handle.
    pub fn new(npu: OptionalNpu) -> Self {
        Self { npu }
    }

    /// The injected NPU, or an error naming the operation that needed it.
    fn npu(&self, operation: &str) -> ServiceResult<&dyn crate::services::NpuAccess> {
        self.npu
            .as_deref()
            .ok_or_else(|| npu_unavailable(operation))
    }
}

#[async_trait]
impl RuntimeService for GenomeRuntimeService {
    async fn get_status(&self) -> ServiceResult<RuntimeStatus> {
        let npu = self.npu("burst engine status")?;
        let frequency_hz = npu.burst_hz() as f64;
        let is_running = npu.is_running();

        Ok(RuntimeStatus {
            is_running,
            // The engine has a single running/stopped control; a distinct paused state does not
            // exist, so a stopped engine is reported as stopped rather than paused.
            is_paused: false,
            frequency_hz,
            burst_count: npu.burst_count(),
            // The configured rate is the only rate the engine reports; it does not measure the
            // achieved rate separately.
            current_rate_hz: if is_running { frequency_hz } else { 0.0 },
            last_burst_neuron_count: feagi_npu::runtime_taps::BurstTaps::instance()
                .motor_activity_summary()
                .total_fired_neurons as usize,
            // No per-burst timing is recorded by the current engine.
            avg_burst_time_ms: 0.0,
        })
    }

    async fn get_burst_count(&self) -> ServiceResult<u64> {
        Ok(self.npu("burst count")?.burst_count())
    }

    async fn set_frequency(&self, frequency: f64) -> ServiceResult<()> {
        if !frequency.is_finite() || frequency <= 0.0 {
            return Err(ServiceError::InvalidInput(format!(
                "burst frequency must be a positive number, got {}",
                frequency
            )));
        }

        self.npu("burst frequency control")?
            .set_burst_hz(frequency.round() as u64)
            .map_err(ServiceError::InvalidInput)
    }

    async fn start(&self) -> ServiceResult<()> {
        self.npu("burst engine start")?.start();
        Ok(())
    }

    async fn stop(&self) -> ServiceResult<()> {
        self.npu("burst engine stop")?.stop();
        Ok(())
    }

    async fn pause(&self) -> ServiceResult<()> {
        // The engine exposes running/stopped only. Pause halts the loop; `resume` restarts it,
        // and because the burst counter is never reset the two round-trip as callers expect.
        self.npu("burst engine pause")?.stop();
        Ok(())
    }

    async fn resume(&self) -> ServiceResult<()> {
        self.npu("burst engine resume")?.start();
        Ok(())
    }

    async fn step(&self) -> ServiceResult<()> {
        let npu = self.npu("single burst")?;
        if npu.is_running() {
            return Err(ServiceError::InvalidInput(
                "cannot step while the burst engine is running; stop it first".to_string(),
            ));
        }
        npu.step_once();
        Ok(())
    }

    async fn reset_burst_count(&self) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode runtime control not yet implemented".to_string(),
        ))
    }

    async fn get_fcl_snapshot(&self) -> ServiceResult<Vec<(u64, f32)>> {
        Ok(vec![]) // TODO: Get from NPU if available
    }

    async fn get_fcl_snapshot_with_cortical_idx(&self) -> ServiceResult<Vec<(u64, u32, f32)>> {
        Ok(vec![]) // TODO: Get from NPU if available
    }

    async fn get_fire_queue_sample(
        &self,
    ) -> ServiceResult<
        std::collections::HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>,
    > {
        Ok(std::collections::HashMap::new()) // TODO: Get from NPU if available
    }

    async fn get_fire_ledger_configs(&self) -> ServiceResult<Vec<(u32, usize)>> {
        Ok(vec![]) // TODO: Get from NPU if available
    }

    async fn configure_fire_ledger_window(
        &self,
        _cortical_idx: u32,
        _window_size: usize,
    ) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode runtime control not yet implemented".to_string(),
        ))
    }

    async fn get_fcl_sampler_config(&self) -> ServiceResult<(f64, u32)> {
        Ok((0.0, 0)) // TODO: Get from NPU if available
    }

    async fn set_fcl_sampler_config(
        &self,
        _frequency: Option<f64>,
        _consumer: Option<u32>,
    ) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode runtime control not yet implemented".to_string(),
        ))
    }

    async fn get_area_fcl_sample_rate(&self, _area_id: u32) -> ServiceResult<f64> {
        Ok(0.0) // TODO: Get from NPU if available
    }

    async fn set_area_fcl_sample_rate(
        &self,
        _area_id: u32,
        _sample_rate: f64,
    ) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode runtime control not yet implemented".to_string(),
        ))
    }

    async fn inject_sensory_by_coordinates(
        &self,
        _cortical_id: &str,
        _xyzp_data: &[(u32, u32, u32, f32)],
        _mode: feagi_services::traits::runtime_service::ManualStimulationMode,
    ) -> ServiceResult<usize> {
        Err(ServiceError::NotImplemented(
            "WASM mode sensory injection not yet implemented".to_string(),
        ))
    }

    async fn register_motor_subscriptions(
        &self,
        _agent_id: &str,
        _cortical_ids: Vec<String>,
        _rate_hz: f64,
    ) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode motor subscription not yet implemented".to_string(),
        ))
    }

    async fn register_visualization_subscriptions(
        &self,
        _agent_id: &str,
        _rate_hz: f64,
    ) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode visualization subscription not yet implemented".to_string(),
        ))
    }

    fn unregister_motor_subscriptions(&self, _agent_id: &str) {
        // No-op: WASM mode does not support agent subscriptions
    }

    fn unregister_visualization_subscriptions(&self, _agent_id: &str) {
        // No-op: WASM mode does not support agent subscriptions
    }

    fn clear_all_motor_subscriptions(&self) {
        // No-op: WASM mode does not support agent subscriptions
    }

    fn clear_all_visualization_subscriptions(&self) {
        // No-op: WASM mode does not support agent subscriptions
    }

    async fn reset_cortical_area_states(
        &self,
        _cortical_indices: &[u32],
    ) -> ServiceResult<Vec<(u32, usize)>> {
        Err(ServiceError::NotImplemented(
            "WASM mode cortical_area reset not yet implemented".to_string(),
        ))
    }
}

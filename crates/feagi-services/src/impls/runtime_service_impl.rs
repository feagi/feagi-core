// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Runtime control service implementation.

Provides control over the FEAGI burst engine runtime.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::sync::Arc;

use async_trait::async_trait;
use feagi_npu_burst_engine::BurstLoopRunner;
use feagi_structures::genomic::cortical_area::CorticalID;
use parking_lot::RwLock;
use tracing::{info, warn};

use crate::traits::RuntimeService;
use crate::types::{RuntimeStatus, ServiceError, ServiceResult};

/// Default implementation of RuntimeService
///
/// Wraps the BurstLoopRunner and provides async interface for runtime control.
pub struct RuntimeServiceImpl {
    burst_runner: Arc<RwLock<BurstLoopRunner>>,
    paused: Arc<RwLock<bool>>,
}

impl RuntimeServiceImpl {
    /// Create a new RuntimeServiceImpl
    pub fn new(burst_runner: Arc<RwLock<BurstLoopRunner>>) -> Self {
        Self {
            burst_runner,
            paused: Arc::new(RwLock::new(false)),
        }
    }
}

#[async_trait]
impl RuntimeService for RuntimeServiceImpl {
    async fn start(&self) -> ServiceResult<()> {
        info!(target: "feagi-services", "Starting burst engine");

        let mut runner = self.burst_runner.write();

        runner
            .start()
            .map_err(|e| ServiceError::InvalidState(e.to_string()))?;

        // Clear paused flag
        *self.paused.write() = false;

        Ok(())
    }

    async fn stop(&self) -> ServiceResult<()> {
        info!(target: "feagi-services", "Stopping burst engine");

        let mut runner = self.burst_runner.write();
        runner.stop();

        // Clear paused flag
        *self.paused.write() = false;

        Ok(())
    }

    async fn pause(&self) -> ServiceResult<()> {
        info!(target: "feagi-services", "Pausing burst engine");

        let runner = self.burst_runner.read();
        if !runner.is_running() {
            return Err(ServiceError::InvalidState(
                "Burst engine is not running".to_string(),
            ));
        }

        // Set paused flag (actual pause implementation depends on burst loop design)
        *self.paused.write() = true;

        // TODO: Implement actual pause mechanism in BurstLoopRunner
        // For now, we just track the paused state
        warn!(target: "feagi-services", "Pause not yet implemented in BurstLoopRunner - using flag only");

        Ok(())
    }

    async fn resume(&self) -> ServiceResult<()> {
        info!(target: "feagi-services", "Resuming burst engine");

        let paused = *self.paused.read();
        if !paused {
            return Err(ServiceError::InvalidState(
                "Burst engine is not paused".to_string(),
            ));
        }

        // Clear paused flag
        *self.paused.write() = false;

        // TODO: Implement actual resume mechanism in BurstLoopRunner
        warn!(target: "feagi-services", "Resume not yet implemented in BurstLoopRunner - using flag only");

        Ok(())
    }

    async fn step(&self) -> ServiceResult<()> {
        info!(target: "feagi-services", "Executing single burst step");

        let runner = self.burst_runner.read();
        if runner.is_running() {
            return Err(ServiceError::InvalidState(
                "Cannot step while burst engine is running in continuous mode".to_string(),
            ));
        }

        // TODO: Implement single-step execution in BurstLoopRunner
        warn!(target: "feagi-services", "Single-step execution not yet implemented in BurstLoopRunner");

        Err(ServiceError::NotImplemented(
            "Single-step execution not yet implemented".to_string(),
        ))
    }

    async fn get_status(&self) -> ServiceResult<RuntimeStatus> {
        let runner = self.burst_runner.read();
        let is_running = runner.is_running();
        let burst_count = runner.get_burst_count();
        let is_paused = *self.paused.read();

        // Note: Some metrics not yet available from BurstLoopRunner
        // - current_rate_hz: Would require tracking actual execution rate
        // - last_burst_neuron_count: Not tracked by BurstLoopRunner
        // - avg_burst_time_ms: Not tracked by BurstLoopRunner
        Ok(RuntimeStatus {
            is_running,
            is_paused,
            frequency_hz: runner.get_frequency(),
            burst_count,
            current_rate_hz: if is_running {
                runner.get_frequency()
            } else {
                0.0
            },
            last_burst_neuron_count: 0, // Not yet tracked
            avg_burst_time_ms: 0.0,     // Not yet tracked
        })
    }

    async fn set_frequency(&self, frequency_hz: f64) -> ServiceResult<()> {
        if frequency_hz <= 0.0 {
            return Err(ServiceError::InvalidInput(
                "Frequency must be greater than 0".to_string(),
            ));
        }

        info!(target: "feagi-services", "Setting burst frequency to {} Hz", frequency_hz);

        let mut runner = self.burst_runner.write();
        runner.set_frequency(frequency_hz);

        Ok(())
    }

    async fn get_burst_count(&self) -> ServiceResult<u64> {
        let runner = self.burst_runner.read();
        Ok(runner.get_burst_count())
    }

    async fn reset_burst_count(&self) -> ServiceResult<()> {
        info!(target: "feagi-services", "Resetting burst count");

        // TODO: Implement burst count reset in BurstLoopRunner
        warn!(target: "feagi-services", "Burst count reset not yet implemented in BurstLoopRunner");

        Err(ServiceError::NotImplemented(
            "Burst count reset not yet implemented".to_string(),
        ))
    }

    async fn get_fcl_snapshot(&self) -> ServiceResult<Vec<(u64, f32)>> {
        let runner = self.burst_runner.read();
        let fcl_data = runner.get_fcl_snapshot();

        // Convert NeuronId (u32) to u64
        let result = fcl_data
            .iter()
            .map(|(neuron_id, potential)| (neuron_id.0 as u64, *potential))
            .collect();

        Ok(result)
    }

    async fn get_fcl_snapshot_with_cortical_idx(&self) -> ServiceResult<Vec<(u64, u32, f32)>> {
        let runner = self.burst_runner.read();
        let fcl_data = runner.get_fcl_snapshot();
        let npu = runner.get_npu();

        // Query cortical_area for each neuron from NPU (single source of truth)
        let result: Vec<(u64, u32, f32)> = fcl_data
            .iter()
            .map(|(neuron_id, potential)| {
                let cortical_idx = npu.lock().unwrap().get_neuron_cortical_area(neuron_id.0);
                (neuron_id.0 as u64, cortical_idx, *potential)
            })
            .collect();

        Ok(result)
    }

    async fn get_fire_queue_sample(
        &self,
    ) -> ServiceResult<
        std::collections::HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>,
    > {
        let mut runner = self.burst_runner.write();

        match runner.get_fire_queue_sample() {
            Some(sample) => {
                // Convert AHashMap to std::HashMap for service layer compatibility
                let result: std::collections::HashMap<_, _> = sample.into_iter().collect();
                Ok(result)
            }
            None => Ok(std::collections::HashMap::new()),
        }
    }

    async fn get_fire_ledger_configs(&self) -> ServiceResult<Vec<(u32, usize)>> {
        let runner = self.burst_runner.read();
        Ok(runner.get_fire_ledger_configs())
    }

    async fn configure_fire_ledger_window(
        &self,
        cortical_idx: u32,
        window_size: usize,
    ) -> ServiceResult<()> {
        let mut runner = self.burst_runner.write();
        runner
            .configure_fire_ledger_window(cortical_idx, window_size)
            .map_err(|e| ServiceError::Internal(format!("Failed to configure fire ledger window: {e}")))?;

        info!(target: "feagi-services", "Configured Fire Ledger window for area {}: {} bursts",
            cortical_idx, window_size);

        Ok(())
    }

    async fn get_fcl_sampler_config(&self) -> ServiceResult<(f64, u32)> {
        let runner = self.burst_runner.read();
        Ok(runner.get_fcl_sampler_config())
    }

    async fn set_fcl_sampler_config(
        &self,
        frequency: Option<f64>,
        consumer: Option<u32>,
    ) -> ServiceResult<()> {
        let runner = self.burst_runner.read();
        runner.set_fcl_sampler_config(frequency, consumer);
        Ok(())
    }

    async fn get_area_fcl_sample_rate(&self, area_id: u32) -> ServiceResult<f64> {
        let runner = self.burst_runner.read();
        Ok(runner.get_area_fcl_sample_rate(area_id))
    }

    async fn set_area_fcl_sample_rate(&self, area_id: u32, sample_rate: f64) -> ServiceResult<()> {
        if sample_rate <= 0.0 || sample_rate > 1000.0 {
            return Err(ServiceError::InvalidInput(
                "Sample rate must be between 0 and 1000 Hz".to_string(),
            ));
        }

        let runner = self.burst_runner.read();
        runner.set_area_fcl_sample_rate(area_id, sample_rate);

        info!(target: "feagi-services", "Set FCL sample rate for area {} to {}Hz", area_id, sample_rate);
        Ok(())
    }

    async fn inject_sensory_by_coordinates(
        &self,
        cortical_id: &str,
        xyzp_data: &[(u32, u32, u32, f32)],
    ) -> ServiceResult<usize> {
        // Parse cortical ID from base64 string
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id).map_err(|e| {
            ServiceError::InvalidInput(format!("Invalid cortical ID format: {}", e))
        })?;

        // Get NPU from burst runner
        let runner = self.burst_runner.read();
        let npu = runner.get_npu();

        // Inject using NPU's service layer method
        let injected_count = {
            let mut npu_lock = npu
                .lock()
                .map_err(|e| ServiceError::Backend(format!("Failed to lock NPU: {}", e)))?;

            npu_lock.inject_sensory_xyzp_by_id(&cortical_id_typed, xyzp_data)
        };

        if injected_count == 0 && !xyzp_data.is_empty() {
            warn!(target: "feagi-services",
                "No neurons found for injection: cortical_id={}, coordinates={}",
                cortical_id, xyzp_data.len());
        } else if injected_count > 0 {
            info!(target: "feagi-services",
                "Injected {} neurons into FCL for cortical area {}",
                injected_count, cortical_id);
        }

        Ok(injected_count)
    }
}

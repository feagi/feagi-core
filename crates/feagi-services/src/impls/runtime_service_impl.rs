// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Runtime control service implementation.

Provides control over the FEAGI burst engine runtime.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::sync::Arc;

use ahash::AHashSet;
use async_trait::async_trait;
use feagi_npu_burst_engine::BurstLoopRunner;
use feagi_structures::genomic::cortical_area::CorticalID;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

use crate::traits::runtime_service::{ManualStimulationMode, FIRE_HISTORY_SUBSCRIPTION_TTL};
use crate::traits::RuntimeService;
use crate::types::{
    FireHistory, FireHistoryArea, FireHistoryFrame, FiredNeuron, RuntimeStatus, ServiceError,
    ServiceResult,
};

/// Bookkeeping for one area subscribed through `subscribe_fire_history`.
#[derive(Debug, Clone)]
struct FireHistorySubscription {
    /// Ledger window before the subscription grew it; `None` when the area was untracked.
    prior_window: Option<usize>,
    last_access: std::time::Instant,
}

/// Default implementation of RuntimeService
///
/// Wraps the BurstLoopRunner and provides async interface for runtime control.
pub struct RuntimeServiceImpl {
    burst_runner: Arc<RwLock<BurstLoopRunner>>,
    paused: Arc<RwLock<bool>>,
    fire_history_subscriptions:
        parking_lot::Mutex<std::collections::HashMap<u32, FireHistorySubscription>>,
    /// Only present when `plasticity` feature is enabled (used in `reset_cortical_area_states`).
    #[cfg(feature = "plasticity")]
    plasticity_service: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl RuntimeServiceImpl {
    /// Create a new RuntimeServiceImpl
    pub fn new(burst_runner: Arc<RwLock<BurstLoopRunner>>) -> Self {
        Self {
            burst_runner,
            paused: Arc::new(RwLock::new(false)),
            fire_history_subscriptions: parking_lot::Mutex::new(std::collections::HashMap::new()),
            #[cfg(feature = "plasticity")]
            plasticity_service: None,
        }
    }

    /// Create a new RuntimeServiceImpl with plasticity support
    #[cfg(feature = "plasticity")]
    pub fn new_with_plasticity(
        burst_runner: Arc<RwLock<BurstLoopRunner>>,
        plasticity_service: Arc<feagi_npu_plasticity::PlasticityService>,
    ) -> Self {
        Self {
            burst_runner,
            paused: Arc::new(RwLock::new(false)),
            fire_history_subscriptions: parking_lot::Mutex::new(std::collections::HashMap::new()),
            plasticity_service: Some(plasticity_service as Arc<dyn std::any::Any + Send + Sync>),
        }
    }

    /// Create a new RuntimeServiceImpl with plasticity support (stub for no_plasticity builds)
    #[cfg(not(feature = "plasticity"))]
    pub fn new_with_plasticity(
        burst_runner: Arc<RwLock<BurstLoopRunner>>,
        _plasticity_service: Arc<dyn std::any::Any + Send + Sync>,
    ) -> Self {
        Self {
            burst_runner,
            paused: Arc::new(RwLock::new(false)),
            fire_history_subscriptions: parking_lot::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl RuntimeServiceImpl {
    /// Subscriptions currently held: `(cortical_idx, prior_window, idle_seconds)`.
    pub fn fire_history_subscriptions(&self) -> Vec<(u32, Option<usize>, u64)> {
        let now = std::time::Instant::now();
        let subs = self.fire_history_subscriptions.lock();
        let mut out: Vec<(u32, Option<usize>, u64)> = subs
            .iter()
            .map(|(idx, sub)| {
                (
                    *idx,
                    sub.prior_window,
                    now.saturating_duration_since(sub.last_access).as_secs(),
                )
            })
            .collect();
        out.sort_by_key(|entry| entry.0);
        out
    }

    /// Grow (never shrink) ledger tracking for `areas` to at least `bursts` and record the
    /// subscription. Returns the window now kept for each area.
    fn subscribe_fire_history_sync(
        &self,
        areas: &[u32],
        bursts: usize,
    ) -> ServiceResult<Vec<(u32, usize)>> {
        if bursts == 0 {
            return Err(ServiceError::InvalidInput(
                "bursts must be at least 1".to_string(),
            ));
        }
        let runner = self.burst_runner.read();
        let npu_arc = runner.get_npu();
        let mut npu = npu_arc
            .lock()
            .map_err(|e| ServiceError::Backend(format!("Failed to lock NPU: {}", e)))?;
        let mut subs = self.fire_history_subscriptions.lock();
        let now = std::time::Instant::now();
        let mut windows = Vec::with_capacity(areas.len());
        for &idx in areas {
            let current = npu.get_fire_ledger_window_size(idx).ok();
            let entry = subs.entry(idx).or_insert_with(|| FireHistorySubscription {
                prior_window: current,
                last_access: now,
            });
            entry.last_access = now;
            let window = match current {
                Some(window) if window >= bursts => window,
                _ => {
                    npu.configure_fire_ledger_window(idx, bursts).map_err(|e| {
                        ServiceError::Backend(format!(
                            "Failed to track area {} in the fire ledger: {}",
                            idx, e
                        ))
                    })?;
                    bursts
                }
            };
            windows.push((idx, window));
        }
        Ok(windows)
    }

    /// Drop subscriptions for `areas`, restoring the ledger window each had before.
    fn unsubscribe_fire_history_sync(&self, areas: &[u32]) -> ServiceResult<usize> {
        let runner = self.burst_runner.read();
        let npu_arc = runner.get_npu();
        let mut npu = npu_arc
            .lock()
            .map_err(|e| ServiceError::Backend(format!("Failed to lock NPU: {}", e)))?;
        let mut subs = self.fire_history_subscriptions.lock();
        let mut dropped = 0usize;
        for idx in areas {
            let Some(sub) = subs.remove(idx) else {
                continue;
            };
            dropped += 1;
            match sub.prior_window {
                None => {
                    npu.untrack_fire_ledger_area(*idx);
                }
                Some(prior) => {
                    if npu.get_fire_ledger_window_size(*idx).ok() != Some(prior) {
                        if let Err(e) = npu.configure_fire_ledger_window(*idx, prior) {
                            warn!(
                                target: "feagi-services",
                                "Could not restore fire ledger window {} for area {}: {}",
                                prior, idx, e
                            );
                        }
                    }
                }
            }
        }
        Ok(dropped)
    }

    fn expire_idle_fire_history_subscriptions(&self) {
        let expired: Vec<u32> = {
            let now = std::time::Instant::now();
            self.fire_history_subscriptions
                .lock()
                .iter()
                .filter(|(_, sub)| {
                    now.saturating_duration_since(sub.last_access) > FIRE_HISTORY_SUBSCRIPTION_TTL
                })
                .map(|(idx, _)| *idx)
                .collect()
        };
        if !expired.is_empty() {
            info!(
                target: "feagi-services",
                "Dropping {} idle fire-history subscription(s): {:?}",
                expired.len(),
                expired
            );
            if let Err(e) = self.unsubscribe_fire_history_sync(&expired) {
                warn!(target: "feagi-services", "Failed to drop idle fire-history subscriptions: {}", e);
            }
        }
    }
}

#[async_trait]
impl RuntimeService for RuntimeServiceImpl {
    async fn subscribe_fire_history(
        &self,
        areas: &[u32],
        bursts: usize,
    ) -> ServiceResult<Vec<(u32, usize)>> {
        self.subscribe_fire_history_sync(areas, bursts)
    }

    async fn unsubscribe_fire_history(&self, areas: &[u32]) -> ServiceResult<usize> {
        self.unsubscribe_fire_history_sync(areas)
    }

    async fn get_fire_history(&self, areas: &[u32], bursts: usize) -> ServiceResult<FireHistory> {
        self.expire_idle_fire_history_subscriptions();
        let windows = self.subscribe_fire_history_sync(areas, bursts)?;

        let runner = self.burst_runner.read();
        let npu_arc = runner.get_npu();
        let npu = npu_arc
            .lock()
            .map_err(|e| ServiceError::Backend(format!("Failed to lock NPU: {}", e)))?;
        let current_burst = npu.get_burst_count();
        let mut out = Vec::with_capacity(windows.len());
        for (idx, window) in windows {
            let frames = npu
                .get_fire_ledger_recent_frames(idx, bursts)
                .map_err(|e| {
                    ServiceError::Backend(format!(
                        "Fire history read failed for area {}: {}",
                        idx, e
                    ))
                })?;
            let frames = frames
                .into_iter()
                .map(|(burst, bitmap)| FireHistoryFrame {
                    burst,
                    neurons: bitmap
                        .iter()
                        .map(|neuron_id| FiredNeuron {
                            neuron_id,
                            voxel: npu.get_neuron_coordinates(neuron_id),
                        })
                        .collect(),
                })
                .collect();
            out.push(FireHistoryArea {
                cortical_idx: idx,
                window,
                frames,
            });
        }
        Ok(FireHistory {
            current_burst,
            areas: out,
        })
    }

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
        runner
            .stop_strict()
            .map_err(|e| ServiceError::Backend(format!("Failed to stop burst engine: {}", e)))?;

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

    async fn register_motor_subscriptions(
        &self,
        agent_id: &str,
        cortical_ids: Vec<String>,
        rate_hz: f64,
    ) -> ServiceResult<()> {
        if rate_hz <= 0.0 {
            return Err(ServiceError::InvalidInput(
                "Motor rate must be greater than 0".to_string(),
            ));
        }

        let cortical_set: AHashSet<String> = cortical_ids.into_iter().collect();
        let runner = self.burst_runner.read();
        runner
            .register_motor_subscriptions_with_rate(agent_id.to_string(), cortical_set, rate_hz)
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))
    }

    async fn register_visualization_subscriptions(
        &self,
        agent_id: &str,
        rate_hz: f64,
    ) -> ServiceResult<()> {
        if rate_hz <= 0.0 {
            return Err(ServiceError::InvalidInput(
                "Visualization rate must be greater than 0".to_string(),
            ));
        }

        let runner = self.burst_runner.read();
        runner
            .register_visualization_subscriptions_with_rate(agent_id.to_string(), rate_hz)
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))
    }

    fn unregister_motor_subscriptions(&self, agent_id: &str) {
        let runner = self.burst_runner.read();
        runner.unregister_motor_subscriptions(agent_id);
    }

    fn unregister_visualization_subscriptions(&self, agent_id: &str) {
        let runner = self.burst_runner.read();
        runner.unregister_visualization_subscriptions(agent_id);
    }

    fn clear_all_motor_subscriptions(&self) {
        let runner = self.burst_runner.read();
        runner.clear_all_motor_subscriptions();
    }

    fn clear_all_visualization_subscriptions(&self) {
        let runner = self.burst_runner.read();
        runner.clear_all_visualization_subscriptions();
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
        let npu = runner.get_npu();

        // CRITICAL: Acquire lock ONCE and do BOTH operations (FCL snapshot + cortical_idx lookup)
        // Previous code acquired lock twice: once for get_fcl_snapshot(), once for cortical_idx
        let lock_start = std::time::Instant::now();
        let thread_id = std::thread::current().id();
        debug!("[NPU-LOCK] RUNTIME-SERVICE: Thread {:?} attempting NPU lock for get_fcl_snapshot_with_cortical_idx at {:?}", thread_id, lock_start);
        let result: Vec<(u64, u32, f32)> = {
            // Acquire lock ONCE for both FCL snapshot and cortical_idx lookup
            let npu_lock = npu.lock().unwrap();
            let lock_acquired = std::time::Instant::now();
            let lock_wait = lock_acquired.duration_since(lock_start);
            debug!(
                "[NPU-LOCK] RUNTIME-SERVICE: Thread {:?} acquired lock after {:.2}ms wait for get_fcl_snapshot_with_cortical_idx",
                thread_id,
                lock_wait.as_secs_f64() * 1000.0
            );

            // STRICT: Resolve cortical_idx without fallbacks (memory neurons are handled explicitly).
            let fcl_data = npu_lock
                .get_last_fcl_snapshot_with_cortical_idx()
                .map_err(|e| {
                    ServiceError::Internal(format!("Failed to resolve FCL cortical_idx: {e}"))
                })?;
            debug!(
                "[NPU-LOCK] RUNTIME-SERVICE: Thread {:?} got FCL snapshot ({} neurons) with cortical_idx",
                thread_id,
                fcl_data.len()
            );

            fcl_data
                .into_iter()
                .map(|(neuron_id, cortical_idx, potential)| {
                    (neuron_id.0 as u64, cortical_idx, potential)
                })
                .collect()
        }; // Lock released here
        let lock_released = std::time::Instant::now();
        let _lock_hold_duration = lock_released.duration_since(lock_start);
        debug!("[NPU-LOCK] RUNTIME-SERVICE: Thread {:?} RELEASED NPU lock after get_fcl_snapshot_with_cortical_idx (total: {:.2}ms wait + {:.2}ms hold, {} neurons)", 
            thread_id,
            lock_released.duration_since(lock_start).as_secs_f64() * 1000.0,
            lock_released.duration_since(lock_start).as_secs_f64() * 1000.0,
            result.len());

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

    async fn get_fire_queue_counts(&self) -> ServiceResult<Vec<(u32, u32)>> {
        let runner = self.burst_runner.read();
        Ok(runner.get_fire_queue_counts().unwrap_or_default())
    }

    async fn get_fire_ledger_configs(&self) -> ServiceResult<Vec<(u32, usize)>> {
        debug!("[NPU-LOCK] RUNTIME-SERVICE: get_fire_ledger_configs() called - this acquires NPU lock!");
        let runner = self.burst_runner.read();
        let configs = runner.get_fire_ledger_configs();
        Ok(configs)
    }

    async fn configure_fire_ledger_window(
        &self,
        cortical_idx: u32,
        window_size: usize,
    ) -> ServiceResult<()> {
        let mut runner = self.burst_runner.write();
        runner
            .configure_fire_ledger_window(cortical_idx, window_size)
            .map_err(|e| {
                ServiceError::Internal(format!("Failed to configure fire ledger window: {e}"))
            })?;

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
        mode: ManualStimulationMode,
    ) -> ServiceResult<usize> {
        // Parse cortical ID from base64 string
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id).map_err(|e| {
            ServiceError::InvalidInput(format!("Invalid cortical ID format: {}", e))
        })?;

        // Get NPU from burst runner
        let runner = self.burst_runner.read();
        let npu = runner.get_npu();

        // Inject using NPU's service layer method
        let lock_start = std::time::Instant::now();
        debug!(
            "[NPU-LOCK] RUNTIME-SERVICE: Acquiring lock for manual stimulation ({} coordinates) - THIS CAN BLOCK BURST LOOP!",
            xyzp_data.len()
        );
        let injected_count = {
            let mut npu_lock = npu
                .lock()
                .map_err(|e| ServiceError::Backend(format!("Failed to lock NPU: {}", e)))?;
            let lock_wait = lock_start.elapsed();
            debug!(
                "[NPU-LOCK] RUNTIME-SERVICE: Lock acquired for manual stimulation (waited {:.2}ms)",
                lock_wait.as_secs_f64() * 1000.0
            );
            let result = match mode {
                ManualStimulationMode::Candidate => {
                    npu_lock.inject_sensory_xyzp_by_id(&cortical_id_typed, xyzp_data)
                }
                ManualStimulationMode::ForceFire => {
                    npu_lock.inject_force_fire_by_coordinates(&cortical_id_typed, xyzp_data)
                }
            };
            let lock_hold_duration = lock_start.elapsed();
            debug!(
                "[NPU-LOCK] RUNTIME-SERVICE: Releasing lock after manual stimulation (held for {:.2}ms)",
                lock_hold_duration.as_secs_f64() * 1000.0
            );
            result
        };

        if injected_count == 0 && !xyzp_data.is_empty() {
            warn!(target: "feagi-services",
                "No neurons found for injection: cortical_id={}, coordinates={}",
                cortical_id, xyzp_data.len());
        }

        Ok(injected_count)
    }

    async fn reset_cortical_area_states(
        &self,
        cortical_indices: &[u32],
    ) -> ServiceResult<Vec<(u32, usize)>> {
        if cortical_indices.is_empty() {
            return Ok(Vec::new());
        }

        let runner = self.burst_runner.read();
        let npu = runner.get_npu();
        let mut npu_lock = npu
            .lock()
            .map_err(|e| ServiceError::Backend(format!("Failed to lock NPU: {e}")))?;

        let mut results: Vec<(u32, usize)> = Vec::with_capacity(cortical_indices.len());
        for &idx in cortical_indices {
            let count = npu_lock.reset_cortical_area_runtime_state(idx);
            results.push((idx, count));
        }

        // Release NPU lock before calling plasticity service
        drop(npu_lock);
        drop(runner);

        // Reset memory neurons if plasticity service is available
        #[cfg(feature = "plasticity")]
        if let Some(plasticity_any) = &self.plasticity_service {
            if let Some(plasticity_service) =
                plasticity_any.downcast_ref::<feagi_npu_plasticity::PlasticityService>()
            {
                for &idx in cortical_indices {
                    let memory_neuron_count = plasticity_service.reset_memory_neurons_in_area(idx);
                    if memory_neuron_count > 0 {
                        info!(
                            target: "feagi-services",
                            "Reset {} memory neurons in cortical area {}",
                            memory_neuron_count,
                            idx
                        );
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod fire_history_tests {
    use super::*;
    use feagi_npu_burst_engine::{
        DynamicNPU, MotorPublisher, RawFireQueueSnapshot, TracingMutex, VisualizationPublisher,
    };

    struct NoViz;
    impl VisualizationPublisher for NoViz {
        fn publish_raw_fire_queue_for_agent(
            &self,
            _agent_id: &str,
            _fire_data: RawFireQueueSnapshot,
        ) -> Result<(), String> {
            Ok(())
        }
    }
    struct NoMotor;
    impl MotorPublisher for NoMotor {
        fn publish_motor(&self, _agent_id: &str, _data: &[u8]) -> Result<(), String> {
            Ok(())
        }
    }

    const AREA: u32 = 5;

    fn service_with_one_neuron() -> (RuntimeServiceImpl, Arc<TracingMutex<DynamicNPU>>, u32) {
        let mut rust_npu = feagi_npu_burst_engine::RustNPU::new(
            feagi_npu_runtime::StdRuntime,
            feagi_npu_burst_engine::backend::CPUBackend::new(),
            1000,
            10000,
            20,
        )
        .unwrap();
        rust_npu.register_cortical_area(
            AREA,
            CorticalID::try_from_bytes(b"cfirehis")
                .unwrap()
                .as_base_64(),
        );
        let neuron = rust_npu
            .add_neuron(
                1.0,
                f32::MAX,
                0.0,
                0.0,
                0,
                0,
                1.0,
                0,
                0,
                true,
                AREA,
                1,
                2,
                3,
            )
            .unwrap()
            .0;
        let npu = Arc::new(TracingMutex::new(DynamicNPU::F32(rust_npu), "TestNPU"));
        let runner = BurstLoopRunner::new::<NoViz, NoMotor>(npu.clone(), None, None, 10.0);
        (
            RuntimeServiceImpl::new(Arc::new(RwLock::new(runner))),
            npu,
            neuron,
        )
    }

    fn burst(npu: &Arc<TracingMutex<DynamicNPU>>, neuron: u32, fire: bool) {
        let mut npu = npu.lock().unwrap();
        if fire {
            npu.inject_sensory_with_potentials(&[(
                feagi_npu_neural::types::NeuronId(neuron),
                128.0,
            )]);
        }
        npu.process_burst().unwrap();
    }

    #[tokio::test]
    async fn fire_history_tracks_only_while_subscribed_and_returns_aligned_frames() {
        let (svc, npu, neuron) = service_with_one_neuron();
        burst(&npu, neuron, true); // burst 1: not tracked yet, must not appear

        let windows = svc.subscribe_fire_history(&[AREA], 3).await.unwrap();
        assert_eq!(windows, vec![(AREA, 3)]);
        for (b, fire) in [(2u64, false), (3, true), (4, false), (5, true)] {
            burst(&npu, neuron, fire);
            assert_eq!(npu.lock().unwrap().get_burst_count(), b);
        }

        let history = svc.get_fire_history(&[AREA], 3).await.unwrap();
        assert_eq!(history.current_burst, 5);
        assert_eq!(history.areas.len(), 1);
        let area = &history.areas[0];
        assert_eq!((area.cortical_idx, area.window), (AREA, 3));
        assert_eq!(
            area.frames.iter().map(|f| f.burst).collect::<Vec<_>>(),
            vec![3, 4, 5],
            "window of 3 ending at the current burst"
        );
        assert_eq!(
            area.frames[0].neurons,
            vec![FiredNeuron {
                neuron_id: neuron,
                voxel: Some((1, 2, 3))
            }]
        );
        assert!(
            area.frames[1].neurons.is_empty(),
            "silent burst is an explicit empty frame"
        );
        assert_eq!(area.frames[2].neurons.len(), 1);

        // A smaller read keeps the window, a larger read grows it.
        let history = svc.get_fire_history(&[AREA], 2).await.unwrap();
        assert_eq!(history.areas[0].window, 3);
        assert_eq!(history.areas[0].frames.len(), 2);
        let history = svc.get_fire_history(&[AREA], 6).await.unwrap();
        assert_eq!(history.areas[0].window, 6);
        assert_eq!(svc.fire_history_subscriptions()[0], (AREA, None, 0));

        assert_eq!(svc.unsubscribe_fire_history(&[AREA, 77]).await.unwrap(), 1);
        assert!(svc.fire_history_subscriptions().is_empty());
        assert!(
            npu.lock()
                .unwrap()
                .get_fire_ledger_window_size(AREA)
                .is_err(),
            "area untracked again after unsubscribe"
        );
    }

    #[tokio::test]
    async fn fire_history_never_shrinks_an_existing_window_and_restores_it() {
        let (svc, npu, _neuron) = service_with_one_neuron();
        npu.lock()
            .unwrap()
            .configure_fire_ledger_window(AREA, 8)
            .unwrap();

        let windows = svc.subscribe_fire_history(&[AREA], 2).await.unwrap();
        assert_eq!(windows, vec![(AREA, 8)], "plasticity window kept");
        svc.subscribe_fire_history(&[AREA], 12).await.unwrap();
        assert_eq!(
            npu.lock()
                .unwrap()
                .get_fire_ledger_window_size(AREA)
                .unwrap(),
            12
        );
        assert_eq!(svc.fire_history_subscriptions()[0].1, Some(8));

        svc.unsubscribe_fire_history(&[AREA]).await.unwrap();
        assert_eq!(
            npu.lock()
                .unwrap()
                .get_fire_ledger_window_size(AREA)
                .unwrap(),
            8,
            "prior window restored"
        );
        assert!(svc.get_fire_history(&[AREA], 0).await.is_err());
    }
}

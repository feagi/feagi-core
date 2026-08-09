// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! System service backed by the shared state manager and the loaded genome.
//!
//! The pre-refactor `SystemServiceImpl` read its counters from `ConnectomeManager` and the old
//! `BurstLoopRunner`. Both were dissolved by the NPU rewrite, so this implementation sources the
//! same values from their current owners: live counters come from
//! [`feagi_state_manager::StateManager`]'s core state, structural counts come from the genome, and
//! the burst counter comes from the NPU's runtime taps.
//!
//! Where the current runtime has no source for a value, the method reports
//! [`ServiceError::NotImplemented`] rather than returning an invented number.

use async_trait::async_trait;
use feagi_services::traits::SystemService;
use feagi_services::types::*;
use feagi_state_manager::StateManager;
use std::time::SystemTime;

/// System service reading live state from [`StateManager`] and structure from the genome.
pub struct GenomeSystemService {
    genome: crate::services::SharedGenome,
    start_time: SystemTime,
    version_info: VersionInfo,
}

impl GenomeSystemService {
    /// Creates the service against the shared genome handle.
    ///
    /// `version_info` is supplied by the application because only the final binary knows which
    /// crate versions were actually linked into it.
    pub fn new(genome: crate::services::SharedGenome, version_info: VersionInfo) -> Self {
        Self {
            genome,
            start_time: SystemTime::now(),
            version_info,
        }
    }

    /// Seconds since this service was constructed, which tracks process start.
    fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().map(|d| d.as_secs()).unwrap_or(0)
    }

    /// Cortical area and brain region counts from the current genome.
    ///
    /// With no genome loaded both counts are genuinely zero, which is what these status fields
    /// should report rather than failing the whole request.
    fn genome_counts(&self) -> (usize, usize) {
        crate::services::with_genome(&self.genome, |g| {
            (g.cortical_areas.len(), g.brain_regions.len())
        })
        .unwrap_or((0, 0))
    }

    /// Reads a value from the core state, or reports the state manager as unavailable.
    ///
    /// `try_read` is used so a REST call never blocks behind the burst thread.
    fn with_core_state<T>(
        f: impl FnOnce(&feagi_state_manager::MemoryMappedState) -> T,
    ) -> ServiceResult<T> {
        match StateManager::instance().try_read() {
            Some(state_manager) => Ok(f(state_manager.get_core_state())),
            None => Err(ServiceError::Internal(
                "state manager is busy; core state could not be read".to_string(),
            )),
        }
    }
}

#[async_trait]
impl SystemService for GenomeSystemService {
    async fn get_health(&self) -> ServiceResult<HealthStatus> {
        let burst_engine_state = Self::with_core_state(|core| core.get_burst_engine_state())?;

        let burst_status = match burst_engine_state {
            feagi_state_manager::BurstEngineState::Running => "healthy",
            feagi_state_manager::BurstEngineState::Ready
            | feagi_state_manager::BurstEngineState::Initializing
            | feagi_state_manager::BurstEngineState::Paused
            | feagi_state_manager::BurstEngineState::LightSleep
            | feagi_state_manager::BurstEngineState::DeepSleep => "degraded",
            feagi_state_manager::BurstEngineState::Unavailable
            | feagi_state_manager::BurstEngineState::Error => "unhealthy",
        };

        let (cortical_area_count, brain_region_count) = self.genome_counts();
        let genome_loaded = cortical_area_count > 0;
        let components = vec![
            ComponentHealth {
                name: "burst_engine".to_string(),
                status: burst_status.to_string(),
                message: Some(format!("{:?}", burst_engine_state)),
            },
            ComponentHealth {
                name: "genome".to_string(),
                status: if genome_loaded { "healthy" } else { "degraded" }.to_string(),
                message: Some(format!(
                    "{} cortical areas, {} brain regions",
                    cortical_area_count, brain_region_count
                )),
            },
        ];

        let overall_status = if components.iter().any(|c| c.status == "unhealthy") {
            "unhealthy"
        } else if components.iter().any(|c| c.status == "degraded") {
            "degraded"
        } else {
            "healthy"
        };

        Ok(HealthStatus {
            overall_status: overall_status.to_string(),
            components,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn get_status(&self) -> ServiceResult<SystemStatus> {
        let (burst_engine_state, neuron_count, synapse_count, burst_frequency) =
            Self::with_core_state(|core| {
                (
                    core.get_burst_engine_state(),
                    core.get_neuron_count(),
                    core.get_synapse_count(),
                    core.get_burst_frequency(),
                )
            })?;

        let (cortical_area_count, brain_region_count) = self.genome_counts();

        Ok(SystemStatus {
            is_initialized: cortical_area_count > 0,
            burst_engine_running: matches!(
                burst_engine_state,
                feagi_state_manager::BurstEngineState::Running
            ),
            burst_count: self.get_burst_count().await?,
            neuron_count: neuron_count as usize,
            synapse_count: synapse_count as usize,
            cortical_area_count,
            brain_region_count,
            uptime_seconds: self.uptime_seconds(),
            current_burst_rate_hz: burst_frequency as f64,
            // No per-burst timing is recorded by the current engine; the taps keep only the most
            // recent burst, so an average cannot be derived without inventing one.
            avg_burst_time_ms: 0.0,
        })
    }

    async fn get_version(&self) -> ServiceResult<VersionInfo> {
        Ok(self.version_info.clone())
    }

    async fn is_initialized(&self) -> ServiceResult<bool> {
        Ok(self.genome_counts().0 > 0)
    }

    async fn get_burst_count(&self) -> ServiceResult<u64> {
        // The taps record the burst number of the most recent motor publish, which is the burst
        // counter the burst loop last committed.
        Ok(feagi_npu::runtime_taps::BurstTaps::instance()
            .motor_snapshot()
            .burst_num)
    }

    async fn get_runtime_stats(&self) -> ServiceResult<RuntimeStats> {
        Err(ServiceError::NotImplemented(
            "cumulative burst timing and fired-neuron totals are not recorded by the current \
             engine"
                .to_string(),
        ))
    }

    async fn get_memory_usage(&self) -> ServiceResult<MemoryUsage> {
        Err(ServiceError::NotImplemented(
            "the current engine does not report allocation sizes for neuron and synapse storage"
                .to_string(),
        ))
    }

    async fn get_capacity(&self) -> ServiceResult<CapacityInfo> {
        let (neuron_count, neuron_capacity, synapse_count, synapse_capacity) =
            Self::with_core_state(|core| {
                (
                    core.get_neuron_count(),
                    core.get_neuron_capacity(),
                    core.get_synapse_count(),
                    core.get_synapse_capacity(),
                )
            })?;

        let percent = |used: u32, max: u32| -> f64 {
            if max == 0 {
                0.0
            } else {
                (used as f64 / max as f64) * 100.0
            }
        };

        Ok(CapacityInfo {
            current_neurons: neuron_count as usize,
            max_neurons: neuron_capacity as usize,
            neuron_utilization_percent: percent(neuron_count, neuron_capacity),
            current_synapses: synapse_count as usize,
            max_synapses: synapse_capacity as usize,
            synapse_utilization_percent: percent(synapse_count, synapse_capacity),
            current_cortical_areas: self.genome_counts().0,
            max_cortical_areas: Self::with_core_state(|core| core.get_cortical_area_count())?
                as usize,
        })
    }
}

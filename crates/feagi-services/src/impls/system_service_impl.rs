/*!
System service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::SystemService;
use crate::types::*;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use feagi_burst_engine::BurstLoopRunner;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::debug;

/// Default implementation of SystemService
pub struct SystemServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager>>,
    burst_runner: Option<Arc<RwLock<BurstLoopRunner>>>,
    start_time: SystemTime,
}

impl SystemServiceImpl {
    pub fn new(
        connectome: Arc<RwLock<ConnectomeManager>>,
        burst_runner: Option<Arc<RwLock<BurstLoopRunner>>>,
    ) -> Self {
        Self {
            connectome,
            burst_runner,
            start_time: SystemTime::now(),
        }
    }

    /// Get uptime in seconds
    fn get_uptime_seconds(&self) -> u64 {
        self.start_time
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Get FEAGI session timestamp in milliseconds (Unix timestamp when FEAGI started)
    pub fn get_feagi_session_timestamp(&self) -> i64 {
        self.start_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    /// Get current timestamp in ISO 8601 format
    fn get_timestamp() -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

#[async_trait]
impl SystemService for SystemServiceImpl {
    async fn get_health(&self) -> ServiceResult<HealthStatus> {
        debug!(target: "feagi-services","Getting system health");

        let mut components = Vec::new();

        // Check connectome health
        let connectome_initialized = self.connectome.read().is_initialized();
        components.push(ComponentHealth {
            name: "Connectome".to_string(),
            status: if connectome_initialized {
                "healthy".to_string()
            } else {
                "degraded".to_string()
            },
            message: if connectome_initialized {
                Some("Genome loaded and brain initialized".to_string())
            } else {
                Some("No genome loaded".to_string())
            },
        });

        // Check NPU health
        let has_npu = self.connectome.read().has_npu();
        components.push(ComponentHealth {
            name: "NPU".to_string(),
            status: if has_npu {
                "healthy".to_string()
            } else {
                "unhealthy".to_string()
            },
            message: if has_npu {
                Some("NPU connected".to_string())
            } else {
                Some("NPU not connected".to_string())
            },
        });

        // Check burst engine health
        let burst_engine_running = if let Some(ref runner) = self.burst_runner {
            runner.read().is_running()
        } else {
            false
        };
        components.push(ComponentHealth {
            name: "BurstEngine".to_string(),
            status: if burst_engine_running {
                "healthy".to_string()
            } else {
                "degraded".to_string()
            },
            message: if burst_engine_running {
                Some("Burst engine active".to_string())
            } else {
                Some("Burst engine stopped".to_string())
            },
        });

        // Determine overall status
        let overall_status = if components.iter().all(|c| c.status == "healthy") {
            "healthy"
        } else if components.iter().any(|c| c.status == "unhealthy") {
            "unhealthy"
        } else {
            "degraded"
        };

        Ok(HealthStatus {
            overall_status: overall_status.to_string(),
            components,
            timestamp: Self::get_timestamp(),
        })
    }

    async fn get_status(&self) -> ServiceResult<SystemStatus> {
        debug!(target: "feagi-services","Getting system status");

        let manager = self.connectome.read();

        let is_initialized = manager.is_initialized();
        let neuron_count = manager.get_neuron_count();
        let synapse_count = manager.get_synapse_count();
        let cortical_area_count = manager.get_cortical_area_count();
        let brain_region_count = manager.get_brain_region_ids().len();

        let (burst_engine_running, burst_count, current_burst_rate_hz, avg_burst_time_ms) =
            if let Some(ref runner) = self.burst_runner {
                let runner_lock = runner.read();
                (
                    runner_lock.is_running(),
                    runner_lock.get_burst_count(),
                    0.0, // TODO: Implement rate tracking in BurstLoopRunner
                    0.0, // TODO: Implement timing tracking in BurstLoopRunner
                )
            } else {
                (false, 0, 0.0, 0.0)
            };

        Ok(SystemStatus {
            is_initialized,
            burst_engine_running,
            burst_count,
            neuron_count,
            synapse_count,
            cortical_area_count,
            brain_region_count,
            uptime_seconds: self.get_uptime_seconds(),
            current_burst_rate_hz,
            avg_burst_time_ms,
        })
    }

    async fn get_version(&self) -> ServiceResult<VersionInfo> {
        debug!(target: "feagi-services","Getting version information");

        Ok(VersionInfo {
            feagi_core_version: env!("CARGO_PKG_VERSION").to_string(),
            feagi_bdu_version: "2.0.0".to_string(), // From feagi-bdu Cargo.toml
            feagi_burst_engine_version: "2.0.0".to_string(),
            feagi_evo_version: "2.0.0".to_string(),
            feagi_types_version: "2.0.0".to_string(),
            build_timestamp: option_env!("VERGEN_BUILD_TIMESTAMP")
                .unwrap_or("unknown")
                .to_string(),
            rust_version: option_env!("VERGEN_RUSTC_SEMVER")
                .unwrap_or(env!("CARGO_PKG_RUST_VERSION"))
                .to_string(),
        })
    }

    async fn is_initialized(&self) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if system is initialized");
        Ok(self.connectome.read().is_initialized())
    }

    async fn get_burst_count(&self) -> ServiceResult<u64> {
        debug!(target: "feagi-services","Getting burst count");

        if let Some(ref runner) = self.burst_runner {
            Ok(runner.read().get_burst_count())
        } else {
            Ok(0)
        }
    }

    async fn get_runtime_stats(&self) -> ServiceResult<RuntimeStats> {
        debug!(target: "feagi-services","Getting runtime statistics");

        let burst_count = if let Some(ref runner) = self.burst_runner {
            runner.read().get_burst_count()
        } else {
            0
        };

        // TODO: Implement detailed runtime statistics in BurstLoopRunner
        // For now, return basic stats
        Ok(RuntimeStats {
            total_bursts: burst_count,
            total_neurons_fired: 0,         // TODO: Track in BurstLoopRunner
            total_processing_time_ms: 0,    // TODO: Track in BurstLoopRunner
            avg_burst_time_ms: 0.0,         // TODO: Track in BurstLoopRunner
            avg_neurons_per_burst: 0.0,     // TODO: Track in BurstLoopRunner
            current_rate_hz: 0.0,           // TODO: Track in BurstLoopRunner
            peak_rate_hz: 0.0,              // TODO: Track in BurstLoopRunner
            uptime_seconds: self.get_uptime_seconds(),
        })
    }

    async fn get_memory_usage(&self) -> ServiceResult<MemoryUsage> {
        debug!(target: "feagi-services","Getting memory usage");

        // TODO: Implement actual memory tracking
        // For now, estimate based on neuron/synapse counts
        let manager = self.connectome.read();
        let neuron_count = manager.get_neuron_count();
        let synapse_count = manager.get_synapse_count();

        // Rough estimates (actual sizes depend on NPU implementation)
        let npu_neurons_bytes = neuron_count * 64; // ~64 bytes per neuron
        let npu_synapses_bytes = synapse_count * 16; // ~16 bytes per synapse
        let npu_total_bytes = npu_neurons_bytes + npu_synapses_bytes;

        let connectome_metadata_bytes = 
            manager.get_cortical_area_count() * 512 + // ~512 bytes per area metadata
            manager.get_brain_region_ids().len() * 256; // ~256 bytes per region

        let total_allocated_bytes = npu_total_bytes + connectome_metadata_bytes;

        // System memory (requires platform-specific code)
        let (system_total_bytes, system_available_bytes) = (0, 0); // TODO: Implement

        Ok(MemoryUsage {
            npu_neurons_bytes,
            npu_synapses_bytes,
            npu_total_bytes,
            connectome_metadata_bytes,
            total_allocated_bytes,
            system_total_bytes,
            system_available_bytes,
        })
    }

    async fn get_capacity(&self) -> ServiceResult<CapacityInfo> {
        debug!(target: "feagi-services","Getting capacity information");

        let manager = self.connectome.read();
        let config = manager.get_config();

        let current_neurons = manager.get_neuron_count();
        let max_neurons = config.max_neurons;
        let neuron_utilization_percent = (current_neurons as f64 / max_neurons as f64) * 100.0;

        let current_synapses = manager.get_synapse_count();
        let max_synapses = config.max_synapses;
        let synapse_utilization_percent = (current_synapses as f64 / max_synapses as f64) * 100.0;

        let current_cortical_areas = manager.get_cortical_area_count();
        let max_cortical_areas = 10000; // TODO: Make this configurable

        Ok(CapacityInfo {
            current_neurons,
            max_neurons,
            neuron_utilization_percent,
            current_synapses,
            max_synapses,
            synapse_utilization_percent,
            current_cortical_areas,
            max_cortical_areas,
        })
    }
}


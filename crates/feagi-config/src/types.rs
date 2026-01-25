// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Configuration type definitions
//!
//! This module defines all configuration structs that map to sections in
//! `feagi_configuration.toml`.

use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::path::PathBuf;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Root configuration structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct FeagiConfig {
    pub system: SystemConfig,
    pub genome: GenomeConfig,
    pub api: ApiConfig,
    pub agent: AgentConfig,
    pub ports: PortsConfig,
    pub zmq: ZmqConfig,
    pub websocket: WebSocketConfig,   // FEAGI 2.0: WebSocket transport
    pub transports: TransportsConfig, // FEAGI 2.0: Multi-transport coordination
    pub timeouts: TimeoutsConfig,
    pub agents: AgentsConfig,
    pub neural: NeuralConfig,
    pub plasticity: PlasticityConfig,
    pub burst_engine: BurstEngineConfig,
    pub connectome: ConnectomeConfig,
    pub resources: ResourcesConfig,
    pub logging: LoggingConfig,
    pub visualization: VisualizationConfig,
    pub compression: CompressionConfig,
    pub memory_processing: MemoryProcessingConfig,
    pub snapshot: SnapshotConfig,
}

/// System-level configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SystemConfig {
    pub max_cores: usize,
    pub debug: bool,
    pub log_level: String,
    #[cfg(feature = "std")]
    pub data_dir: PathBuf,
    #[cfg(not(feature = "std"))]
    pub data_dir: String,
    pub cpu_affinity: Vec<usize>,
    pub priority: i32,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            max_cores: 0, // 0 = auto-detect
            debug: true,
            log_level: "WARNING".to_string(),
            #[cfg(feature = "std")]
            data_dir: PathBuf::from(""),
            #[cfg(not(feature = "std"))]
            data_dir: String::new(),
            cpu_affinity: Vec::new(),
            priority: 0,
        }
    }
}

/// Genome loading and validation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GenomeConfig {
    pub auto_recovery_on_validation_failure: bool,
}

impl Default for GenomeConfig {
    fn default() -> Self {
        Self {
            auto_recovery_on_validation_failure: true,
        }
    }
}

/// REST API server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub reload: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8000,
            workers: 1,
            reload: false,
        }
    }
}

/// Agent registration and communication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AgentConfig {
    pub registration_port: u16,
    pub sensory_port: u16,
    pub motor_port: u16,
    pub host: String,
    /// Enable auto-creation of missing IPU/OPU cortical areas during agent registration
    pub auto_create_missing_cortical_areas: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            registration_port: 30001,
            sensory_port: 5555, // NOTE: This is agent config (different from ports.zmq_sensory_port)
            motor_port: 5564,
            host: "0.0.0.0".to_string(),
            auto_create_missing_cortical_areas: true,
        }
    }
}

/// ZMQ communication ports
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PortsConfig {
    pub zmq_req_rep_port: u16,
    pub zmq_pub_sub_port: u16,
    pub zmq_push_pull_port: u16,
    pub zmq_sensory_port: u16,
    pub zmq_visualization_port: u16,
    pub zmq_rest_port: u16,
    pub zmq_motor_port: u16,
}

impl Default for PortsConfig {
    fn default() -> Self {
        Self {
            zmq_req_rep_port: 5555,
            zmq_pub_sub_port: 5556,
            zmq_push_pull_port: 5557,
            zmq_sensory_port: 5558,
            zmq_visualization_port: 5562,
            zmq_rest_port: 5563,
            zmq_motor_port: 5564,
        }
    }
}

impl PortsConfig {
    /// Get all ports as a vector for conflict detection
    pub fn all_ports(&self) -> Vec<(&str, u16)> {
        vec![
            ("zmq_req_rep", self.zmq_req_rep_port),
            ("zmq_pub_sub", self.zmq_pub_sub_port),
            ("zmq_push_pull", self.zmq_push_pull_port),
            ("zmq_sensory", self.zmq_sensory_port),
            ("zmq_visualization", self.zmq_visualization_port),
            ("zmq_rest", self.zmq_rest_port),
            ("zmq_motor", self.zmq_motor_port),
        ]
    }
}

/// ZMQ-specific settings
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ZmqConfig {
    pub host: String,
    pub enabled: bool,
    pub polling_timeout: u64,
    pub message_buffer_size: usize,
    pub socket_connect_timeout: u64,
    pub socket_receive_timeout: u64,
    pub socket_send_timeout: u64,
    pub client_heartbeat_timeout: u64,
    pub inactive_client_timeout: u64,
    pub streams: ZmqStreamsConfig,
}

impl Default for ZmqConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            enabled: true,
            polling_timeout: 100,
            message_buffer_size: 100,
            socket_connect_timeout: 1000,
            socket_receive_timeout: 5000,
            socket_send_timeout: 5000,
            client_heartbeat_timeout: 30000,
            inactive_client_timeout: 60000,
            streams: ZmqStreamsConfig::default(),
        }
    }
}

/// ZMQ stream configurations
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ZmqStreamsConfig {
    pub visualization: VisualizationStreamConfig,
    pub sensory: SensoryStreamConfig,
    pub motor: MotorStreamConfig,
    pub rest: RestStreamConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct VisualizationStreamConfig {
    pub enabled: bool,
    pub auto_enable_on_subscribers: bool,
    pub subscriber_check_interval: f64,
}

impl Default for VisualizationStreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_enable_on_subscribers: true,
            subscriber_check_interval: 1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SensoryStreamConfig {
    pub enabled: bool,
    pub receive_high_water_mark: u32,
    pub linger_ms: u32,
    pub immediate: bool,
    pub poll_timeout_ms: u32,
    pub startup_drain_timeout_ms: u32,
}

impl Default for SensoryStreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            receive_high_water_mark: 1,
            linger_ms: 0,
            immediate: true,
            poll_timeout_ms: 10,
            startup_drain_timeout_ms: 500,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MotorStreamConfig {
    pub enabled: bool,
}

impl Default for MotorStreamConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RestStreamConfig {
    pub enabled: bool,
}

impl Default for RestStreamConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// WebSocket-specific settings (FEAGI 2.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub host: String,
    pub sensory_port: u16,
    pub motor_port: u16,
    pub visualization_port: u16,
    pub registration_port: u16,
    pub rest_api_port: u16,
    pub connection_timeout_ms: u64,
    pub ping_interval_ms: u64,
    pub ping_timeout_ms: u64,
    pub close_timeout_ms: u64,
    pub max_message_size: usize,
    pub max_connections: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            host: "0.0.0.0".to_string(),
            sensory_port: 9051,
            motor_port: 9052,
            visualization_port: 9050,
            registration_port: 9053,
            rest_api_port: 9054,
            connection_timeout_ms: 5000,
            ping_interval_ms: 60000,
            ping_timeout_ms: 60000,
            close_timeout_ms: 10000,
            max_message_size: 10485760, // 10MB
            max_connections: 100,
        }
    }
}

/// Multi-transport coordination settings (FEAGI 2.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TransportsConfig {
    pub available: Vec<String>,
    pub default: String,
    pub allow_mixed: bool,
}

impl Default for TransportsConfig {
    fn default() -> Self {
        Self {
            available: vec!["zmq".to_string()],
            default: "zmq".to_string(),
            allow_mixed: false,
        }
    }
}

/// System-wide timeout configurations
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TimeoutsConfig {
    pub graceful_shutdown: f64,
    pub service_startup: f64,
    pub thread_join: f64,
    pub process_join: f64,
    pub service_stop: f64,
    pub visualization_shutdown: f64,
    pub api_service_shutdown: f64,
    pub fq_sampler_shutdown: f64,
}

impl Default for TimeoutsConfig {
    fn default() -> Self {
        Self {
            graceful_shutdown: 8.0,
            service_startup: 3.0,
            thread_join: 2.0,
            process_join: 2.0,
            service_stop: 5.0,
            visualization_shutdown: 5.0,
            api_service_shutdown: 10.0,
            fq_sampler_shutdown: 2.0,
        }
    }
}

/// Agents configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AgentsConfig {
    pub default_host: String,
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            default_host: "127.0.0.1".to_string(),
        }
    }
}

/// Neural processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct NeuralConfig {
    /// Burst engine timestep in seconds (default: 0.1s = 10Hz)
    /// This value is overridden by genome's simulation_timestep when a genome is loaded
    pub burst_engine_timestep: f64,
    pub batch_size: usize,
    pub use_sparse_computation: bool,
    pub enable_plasticity: bool,
    pub hybrid: HybridConfig,
}

impl Default for NeuralConfig {
    fn default() -> Self {
        Self {
            burst_engine_timestep: 0.1,
            batch_size: 1000,
            use_sparse_computation: true,
            enable_plasticity: true,
            hybrid: HybridConfig::default(),
        }
    }
}

/// Hybrid CPU/GPU processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HybridConfig {
    pub enabled: bool,
    pub gpu_threshold: usize,
    pub keepalive_enabled: bool,
    pub keepalive_interval: f64,
    pub auto_tune_threshold: bool,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gpu_threshold: 1_000_000,
            keepalive_enabled: true,
            keepalive_interval: 30.0,
            auto_tune_threshold: false,
        }
    }
}

/// Plasticity system configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PlasticityConfig {
    pub queue_capacity: usize,
    pub max_ops_per_burst: usize,
    pub stdp: StdpConfig,
    pub memory: MemoryConfig,
}

impl Default for PlasticityConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 4096,
            max_ops_per_burst: 1024,
            stdp: StdpConfig::default(),
            memory: MemoryConfig::default(),
        }
    }
}

/// STDP (Spike-Time Dependent Plasticity) configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StdpConfig {
    pub lookback_steps: usize,
    pub tau_pre: f64,
    pub tau_post: f64,
    pub a_plus: f64,
    pub a_minus: f64,
}

impl Default for StdpConfig {
    fn default() -> Self {
        Self {
            lookback_steps: 20,
            tau_pre: 20.0,
            tau_post: 20.0,
            a_plus: 0.01,
            a_minus: 0.012,
        }
    }
}

/// Memory formation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MemoryConfig {
    pub lookback_steps: usize,
    pub pattern_duration: usize,
    pub min_activation_count: usize,
    pub default_temporal_depth: usize,
    pub pattern_cache_size: usize,
    pub array_capacity: usize,
    pub initial_lifespan: u32,
    pub lifespan_growth_rate: f32,
    pub longterm_threshold: u32,
    pub max_reactivations: u32,
    pub firing_threshold: f32,
    pub initial_membrane_potential: f32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            lookback_steps: 50,
            pattern_duration: 10,
            min_activation_count: 3,
            default_temporal_depth: 3,
            pattern_cache_size: 10000,
            array_capacity: 50000,
            initial_lifespan: 20,
            lifespan_growth_rate: 3.0,
            longterm_threshold: 100,
            max_reactivations: 1000,
            firing_threshold: 1.0,
            initial_membrane_potential: 0.0,
        }
    }
}

/// Burst Engine memory management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BurstEngineConfig {
    pub mode: String,
    pub fcl_capacity_multiplier: f64,
    pub fire_queue_capacity_multiplier: f64,
    pub memory_area_multiplier: f64,
    pub enable_preallocation: bool,
    pub enable_capacity_warnings: bool,
    pub sleep: BurstEngineSleepConfig,
}

impl Default for BurstEngineConfig {
    fn default() -> Self {
        Self {
            mode: "inference".to_string(),
            fcl_capacity_multiplier: 1.5,
            fire_queue_capacity_multiplier: 1.2,
            memory_area_multiplier: 2.0,
            enable_preallocation: true,
            enable_capacity_warnings: true,
            sleep: BurstEngineSleepConfig::default(),
        }
    }
}

/// Burst Engine sleep mode configuration (system-level)
/// Note: All sleep parameters (thresholds, frequencies) come from genome physiology
/// This only contains the master enable/disable flag
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BurstEngineSleepConfig {
    /// Master kill switch for sleep mode (all params come from genome)
    pub enabled: bool,
}

impl Default for BurstEngineSleepConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Connectome sizing and memory allocation (SINGLE SOURCE OF TRUTH)
/// These values define MAXIMUM capacity allocated at NPU initialization.
/// There is NO dynamic growth - these ARE the hard limits.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ConnectomeConfig {
    pub neuron_space: usize,
    pub synapse_space: usize,
    pub memory_neuron_space: usize,
    pub memory_processing_batch_size: usize,
    pub memory_pattern_cache_size: usize,
    pub memory_neuron_limit_per_area: usize,
}

impl Default for ConnectomeConfig {
    fn default() -> Self {
        Self {
            neuron_space: 100_000,
            synapse_space: 500_000,
            memory_neuron_space: 50_000,
            memory_processing_batch_size: 100,
            memory_pattern_cache_size: 10_000,
            memory_neuron_limit_per_area: 10_000,
        }
    }
}

/// Resource allocation settings
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ResourcesConfig {
    pub use_gpu: bool,
    pub gpu_memory_fraction: f64,
    pub enable_health_check: bool,
}

impl Default for ResourcesConfig {
    fn default() -> Self {
        Self {
            use_gpu: true,
            gpu_memory_fraction: 0.8,
            enable_health_check: true,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub global_log_level: String,
    pub print_debug_logs: bool,
    pub print_burst_info: bool,
    pub log_file: String,
    pub max_log_size: String,
    pub backup_count: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            global_log_level: "WARNING".to_string(),
            print_debug_logs: false,
            print_burst_info: true,
            log_file: "feagi.log".to_string(),
            max_log_size: "10MB".to_string(),
            backup_count: 5,
        }
    }
}

/// Visualization settings
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct VisualizationConfig {
    pub enabled: bool,
    pub update_interval: u64,
    /// Visualization transport selection.
    ///
    /// Allowed values:
    /// - "auto": honor agent request (chosen_transport / shm_path)
    /// - "websocket": never allocate/advertise SHM visualization paths
    /// - "shm": always allocate/advertise SHM visualization paths when applicable
    pub transport: String,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval: 50,
            transport: "auto".to_string(),
        }
    }
}

/// Data compression settings
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub min_size_threshold: usize,
    pub enable_stats: bool,
    pub algorithm: String,
    pub compress_visualization: bool,
    pub compress_motor: bool,
    pub compress_sensory: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_size_threshold: 100,
            enable_stats: true,
            algorithm: "lz4".to_string(),
            compress_visualization: true,
            compress_motor: true,
            compress_sensory: false,
        }
    }
}

/// Memory processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MemoryProcessingConfig {
    pub batch_size: usize,
    pub pattern_cache_size: usize,
    pub sleep_manager: SleepManagerConfig,
}

impl Default for MemoryProcessingConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            pattern_cache_size: 10_000,
            sleep_manager: SleepManagerConfig::default(),
        }
    }
}

/// Sleep manager configuration for memory system
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SleepManagerConfig {
    pub enabled: bool,
    pub fcl_low_activity_window_bursts: usize,
    pub fcl_low_activity_threshold: usize,
    pub monitor_interval_seconds: f64,
    pub gc_prune_inactive_after_bursts: usize,
}

impl Default for SleepManagerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fcl_low_activity_window_bursts: 50,
            fcl_low_activity_threshold: 5,
            monitor_interval_seconds: 2.0,
            gc_prune_inactive_after_bursts: 500,
        }
    }
}

/// Brain snapshot configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SnapshotConfig {
    pub output_dir: String,
    pub temp_dir: String,
    pub zip_compression: String,
    pub default_format: String,
    pub fc_compression: String,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            output_dir: "output/snapshots".to_string(),
            temp_dir: "output/snapshots/tmp".to_string(),
            zip_compression: "deflate".to_string(),
            default_format: "zip".to_string(),
            fc_compression: "store".to_string(),
        }
    }
}

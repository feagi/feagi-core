//! Monitoring: system monitoring, metrics, telemetry, and cortical activity sampling.
//! Converted from the old `monitoring` endpoints (`/v1/monitoring/*`).

use feagi_basis::feagi_basis_proc_common::define_request_message_category;

/// Aggregated system health values returned inside monitoring data.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MonitoringData {
    pub neuron_count: u64,
    pub cortical_area_count: u64,
    pub burst_count: u64,
    pub brain_readiness: bool,
    pub burst_engine_active: bool,
}

/// Firing statistics over a cortical activity monitoring window.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FiringStatistics {
    /// Total spikes seen over the window
    pub total_spikes: usize,
    /// Spikes per second over the window
    pub firing_rate_hz: f32,
    /// IDs of neurons that fired at least once
    pub active_neurons: Vec<u32>,
    /// Number of distinct neurons that fired
    pub active_neuron_count: usize,
    /// Neuron that fired most often, if any fired
    pub peak_firing_neuron: Option<u32>,
    /// Average membrane potential across all sampled spikes
    pub avg_membrane_potential: f32,
}

/// One spike recorded during cortical activity monitoring.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpikeRecord {
    pub burst: u64,
    pub neuron_id: u32,
    pub potential: f32,
}

define_request_message_category! {
    category_name: "Monitoring",
    base_path: "v1/monitoring",
    category_description: "System monitoring, metrics, telemetry, and real-time cortical activity sampling",
    read: {
        "Status": {
            path: "status",
            description: "Get monitoring system status including metrics collection and brain readiness",
            response: {
                enabled: bool, "Whether monitoring is enabled",
                metrics_collected: u32, "Number of metrics collected",
                brain_readiness: bool, "Whether the brain is ready",
                burst_engine_active: bool, "Whether the burst engine is running",
            },
        },
        "Metrics": {
            path: "metrics",
            description: "Get system metrics including burst frequency, neuron count, and brain readiness",
            response: {
                burst_frequency_hz: f64, "Current burst frequency in Hz",
                burst_count: u64, "Total bursts executed",
                neuron_count: u64, "Total neuron count",
                cortical_area_count: u64, "Total cortical area count",
                brain_readiness: bool, "Whether the brain is ready",
                burst_engine_active: bool, "Whether the burst engine is running",
            },
        },
        "Data": {
            path: "data",
            description: "Get detailed monitoring data with timestamps for analysis and debugging",
            alloc_only: true,
            response: {
                data: MonitoringData, "Aggregated system health values",
                timestamp: String, "RFC 3339 timestamp of the sample",
            },
        },
        "Performance": {
            path: "performance",
            description: "Get performance metrics including CPU and memory usage",
            response: {
                cpu_usage: f64, "CPU usage",
                memory_usage: f64, "Memory usage",
            },
        },
        "CorticalActivity": {
            path: "cortical_activity",
            description: "Monitor real-time neuron firing activity for a specific cortical area over a time window",
            alloc_only: true,
            payload: {
                area: String, "Cortical area ID (base64) to monitor",
                duration: Option<f32>, "Duration to monitor in seconds (default 1.0)",
            },
            response: {
                area_id: String, "Monitored cortical area ID",
                area_name: String, "Monitored cortical area name",
                duration_ms: f32, "Monitoring window length in milliseconds",
                burst_count_sampled: u64, "Number of bursts that elapsed during the window",
                firing_statistics: FiringStatistics, "Aggregate firing statistics",
                spike_history: Vec<SpikeRecord>, "Every spike sampled during the window",
            },
            error: {
                AreaNotFound: "No cortical area has that ID",
            },
        },
    },
}

// NOTES:
// All responses were untyped HashMap<String, Value>; field types inferred from json!() inserts and service types (integer widths guessed).
// Status: metrics_collected was a hardcoded 5.
// Performance: cpu_usage/memory_usage were hardcoded 0.0 placeholders; units unknown.
// CorticalActivity: duration had a serde default of 1.0, made Option<f32>.

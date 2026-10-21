//! System: health check, versions, configuration, user preferences, logs, beacon,
//! visualization sampler settings and other FEAGI instance level information.
//! Converted from the old `system` endpoints (reference only).

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

/// Fatigue information reported by the health check.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FatigueInfo {
    /// Fatigue index (0-100), maximum utilization across all fatigue criteria
    pub fatigue_index: Option<u8>,
    /// Whether fatigue is currently active (triggers fatigue neuron injection)
    pub fatigue_active: Option<bool>,
    /// Regular neuron utilization percentage (0-100)
    pub regular_neuron_util: Option<u8>,
    /// Memory neuron utilization percentage (0-100)
    pub memory_neuron_util: Option<u8>,
    /// Synapse utilization percentage (0-100)
    pub synapse_util: Option<u8>,
}

/// Per memory area statistics reported by the health check.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoryAreaStats {
    /// Current memory neuron count in the area
    pub neuron_count: u64,
    /// Total memory neurons created in the area
    pub created_total: u64,
    /// Total memory neurons deleted from the area
    pub deleted_total: u64,
    /// Time of the last update
    pub last_updated: u64,
}

/// Summary of a pending amalgamation reported by the health check.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmalgamationPending {
    /// ID of the pending amalgamation
    pub amalgamation_id: String,
    /// Title of the genome being amalgamated
    pub genome_title: String,
    /// Size of the incoming circuit
    pub circuit_size: [u32; 3],
}

/// Activity state of a system process.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProcessStatus {
    /// Whether the process is active
    pub active: bool,
    /// Whether the process is paused (burst engine only)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub paused: Option<bool>,
}

/// A single log record from the in-process ring buffer.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LogTailRecord {
    /// Unix timestamp in milliseconds
    pub timestamp_ms: i64,
    /// Severity level
    pub level: String,
    /// Tracing target
    pub target: String,
    /// Source file (omitted when empty)
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub file: String,
    /// Source line
    pub line: u32,
    /// Log message
    pub message: String,
    /// Structured fields attached to the record
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fields: Option<serde_json::Value>,
}

define_request_message_category! {
    category_name: "System",
    base_path: "v1/system",
    category_description: "FEAGI instance level information and settings: health, versions, configuration, preferences, logs, beacon and visualization sampling",
    read: {
        "HealthCheck": {
            path: "health_check",
            description: "Get comprehensive system health including burst engine status, neuron/synapse counts and genome availability",
            alloc_only: true,
            response: {
                burst_engine: bool, "Whether the burst engine is running",
                connected_agents: Option<i32>, "Number of connected agents",
                influxdb_availability: bool, "Whether InfluxDB is available",
                neuron_count_max: i64, "Neuron capacity",
                synapse_count_max: i64, "Synapse capacity",
                latest_changes_saved_externally: bool, "Whether the latest changes were saved externally",
                genome_availability: bool, "Whether a genome is loaded",
                genome_validity: Option<bool>, "Whether the loaded genome is valid",
                brain_readiness: bool, "Whether the brain is ready",
                genome_loading: bool, "True while a genome load/upload is in progress",
                genome_state: String, "Genome lifecycle: missing, loading, loaded, saving or error",
                feagi_session: Option<i64>, "Session timestamp identifying this FEAGI instance",
                fitness: Option<f64>, "Fitness score",
                cortical_area_count: Option<i32>, "Number of cortical areas",
                neuron_count: Option<i64>, "Total neuron count",
                memory_neuron_count: Option<i64>, "Memory neuron count",
                regular_neuron_count: Option<i64>, "Regular neuron count",
                synapse_count: Option<i64>, "Total synapse count",
                estimated_brain_size_in_MB: Option<f64>, "Estimated brain size in megabytes",
                genome_num: Option<i32>, "Genome number",
                genome_timestamp: Option<i64>, "Genome timestamp",
                simulation_timestep: Option<f64>, "Simulation timestep in seconds",
                memory_area_stats: Option<HashMap<String, MemoryAreaStats>>, "Memory area statistics keyed by cortical ID",
                amalgamation_pending: Option<AmalgamationPending>, "Pending amalgamation summary",
                brain_regions_hash: Option<u64>, "Hash of brain regions (hierarchy, membership, properties)",
                cortical_areas_hash: Option<u64>, "Hash of cortical areas and properties (excluding mappings)",
                brain_geometry_hash: Option<u64>, "Hash of brain geometry (positions, dimensions, 2D coordinates)",
                morphologies_hash: Option<u64>, "Hash of the morphology registry",
                cortical_mappings_hash: Option<u64>, "Hash of cortical mappings",
                agent_data_hash: Option<u64>, "Hash of agent data (IDs, capabilities, connection properties)",
                brain_regions_root: Option<String>, "Root brain region ID",
                fatigue: Option<FatigueInfo>, "Fatigue index, state and contributing utilizations",
            },
        },
        "CorticalAreaVisualizationSkipRate": {
            path: "cortical_area_visualization_skip_rate",
            description: "Get the visualization skip rate (how many frames to skip during visualization)",
            response: {
                skip_rate: i32, "Visualization skip rate",
            },
        },
        "CorticalAreaVisualizationSuppressionThreshold": {
            path: "cortical_area_visualization_suppression_threshold",
            description: "Get the threshold below which cortical areas are suppressed from visualization",
            response: {
                threshold: i32, "Suppression threshold",
            },
        },
        "Version": {
            path: "version",
            description: "Get the current FEAGI version string",
            alloc_only: true,
            response: {
                version: String, "FEAGI version",
            },
        },
        "Versions": {
            path: "versions",
            description: "Get detailed version information for all FEAGI crates and components",
            alloc_only: true,
            response: {
                versions: HashMap<String, String>, "Version keyed by crate/component name (includes rust and build_timestamp)",
            },
        },
        "Configuration": {
            path: "configuration",
            description: "Get system configuration including API settings and neuron/synapse capacity",
            alloc_only: true,
            response: {
                api_host: String, "API host",
                api_port: u16, "API port",
                max_neurons: u64, "Neuron capacity",
                max_synapses: u64, "Synapse capacity",
            },
        },
        "UserPreferences": {
            path: "user_preferences",
            description: "Get user preferences including advanced mode, UI magnification and auto-creation settings",
            response: {
                adv_mode: bool, "Advanced mode enabled",
                ui_magnification: f64, "UI magnification factor",
                auto_pns_area_creation: bool, "Automatically create PNS areas",
            },
        },
        "CorticalAreaTypes": {
            path: "cortical_area_types",
            description: "Get the list of available cortical area types",
            alloc_only: true,
            response: {
                types: Vec<String>, "Cortical area type names (Sensory, Motor, Custom, Memory, Core)",
            },
        },
        "FclStatus": {
            path: "fcl_status",
            description: "Get Fire Candidate List sampler status including frequency and consumer state",
            response: {
                available: bool, "Whether the sampler is available",
                frequency: f64, "Sampler frequency in Hz",
                consumer: u32, "Sampler consumer setting",
                enabled: bool, "Whether the sampler is enabled (consumer > 0)",
            },
        },
        "Processes": {
            path: "processes",
            description: "Get status of active system processes including the burst engine and API server",
            response: {
                burst_engine: ProcessStatus, "Burst engine process status",
                api_server: ProcessStatus, "API server process status",
            },
        },
        "UniqueLogs": {
            path: "unique_logs",
            description: "Get the collection of unique log messages",
            alloc_only: true,
            response: {
                logs: Vec<String>, "Unique log messages",
            },
        },
        "LogTail": {
            path: "log_tail",
            description: "Get a tail of recent log records from the in-process tracing ring buffer; filters are optional and combined",
            alloc_only: true,
            payload: {
                since_ts_ms: Option<i64>, "Only return records emitted at or after this Unix timestamp (ms)",
                level: Option<String>, "Minimum severity (TRACE/DEBUG/INFO/WARN/ERROR)",
                target_prefix: Option<String>, "Restrict to tracing targets starting with this prefix",
                limit: Option<u64>, "Maximum number of records to return (most recent win)",
            },
            response: {
                enabled: bool, "Whether the log ring buffer is enabled",
                capacity: u64, "Ring buffer capacity in records",
                records: Vec<LogTailRecord>, "Matching records, oldest first",
                returned: u64, "Number of records returned",
            },
        },
        "BeaconSubscribers": {
            path: "beacon/subscribers",
            description: "Get the list of beacon subscribers",
            alloc_only: true,
            response: {
                subscribers: Vec<String>, "Beacon subscribers",
            },
        },
        "GlobalActivityVisualization": {
            path: "global_activity_visualization",
            description: "Get global activity visualization configuration",
            response: {
                enabled: bool, "Whether global activity visualization is enabled",
                frequency_hz: f64, "Visualization frequency in Hz",
            },
        },
        "InfluxdbTest": {
            path: "db/influxdb/test",
            description: "Test connectivity to the InfluxDB time-series database",
            response: {
                connected: bool, "Whether InfluxDB is connected",
                available: bool, "Whether InfluxDB is available",
            },
        },
    },
    create: {
        "EnableVisualizationFqSampler": {
            path: "enable_visualization_fq_sampler",
            description: "Enable the Fire Queue sampler for visualization data streaming",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "DisableVisualizationFqSampler": {
            path: "disable_visualization_fq_sampler",
            description: "Disable the Fire Queue sampler to stop visualization data streaming",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "FclReset": {
            path: "fcl_reset",
            description: "Reset the Fire Candidate List, clearing all pending fire candidates",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "Logs": {
            path: "logs",
            description: "Configure logging settings such as log level and output destinations",
            alloc_only: true,
            payload: {
                config: HashMap<String, serde_json::Value>, "Logging configuration",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "BeaconSubscribe": {
            path: "beacon/subscribe",
            description: "Subscribe to the system beacon for event notifications",
            alloc_only: true,
            payload: {
                request: HashMap<String, String>, "Subscription details",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "CircuitLibraryPath": {
            path: "circuit_library_path",
            description: "Set the file system path of the circuit library",
            alloc_only: true,
            payload: {
                circuit_library_path: String, "New circuit library path",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "Register": {
            path: "register",
            description: "Register a system component or module with FEAGI",
            alloc_only: true,
            payload: {
                component: HashMap<String, serde_json::Value>, "Component registration details",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
    },
    edit: {
        "CorticalAreaVisualizationSkipRate": {
            path: "cortical_area_visualization_skip_rate",
            description: "Set the visualization skip rate to reduce visualization frequency",
            alloc_only: true,
            payload: {
                skip_rate: i32, "New visualization skip rate",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "CorticalAreaVisualizationSuppressionThreshold": {
            path: "cortical_area_visualization_suppression_threshold",
            description: "Set the threshold for suppressing low-activity cortical areas from visualization",
            alloc_only: true,
            payload: {
                threshold: i32, "New suppression threshold",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "UserPreferences": {
            path: "user_preferences",
            description: "Update user preferences for UI customization and behavior",
            alloc_only: true,
            payload: {
                adv_mode: Option<bool>, "Advanced mode enabled",
                ui_magnification: Option<f64>, "UI magnification factor",
                auto_pns_area_creation: Option<bool>, "Automatically create PNS areas",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "GlobalActivityVisualization": {
            path: "global_activity_visualization",
            description: "Configure global activity visualization settings and frequency",
            alloc_only: true,
            payload: {
                enabled: Option<bool>, "Enable or disable global activity visualization",
                frequency_hz: Option<f64>, "Visualization frequency in Hz",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
    },
    delete: {
        "BeaconUnsubscribe": {
            path: "beacon/unsubscribe",
            description: "Unsubscribe from the system beacon",
            alloc_only: true,
            payload: {
                request: HashMap<String, String>, "Subscription details",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
    },
}

// NOTES:
// - Bare responses wrapped: skip_rate, threshold (i32), version (String), versions (map), types / subscribers (Vec<String>).
// - Bare request bodies wrapped: skip_rate / threshold (old body was a bare i32).
// - Untyped map bodies wrapped in one field: Logs `config`, BeaconSubscribe/Unsubscribe `request`, Register `component` (old code ignored them).
// - CircuitLibraryPath `circuit_library_path` field is a guess (old body was an ignored HashMap<String, String>).
// - UserPreferences and GlobalActivityVisualization edit payloads guessed from the GET fields (old body was an ignored map).
// - Processes: old response was a map of process name -> status; typed as two fixed fields.
// - MemoryAreaStats field types (u64, last_updated) and AmalgamationPending.circuit_size [u32; 3] are guesses.
// - estimated_brain_size_in_MB keeps its non-snake-case JSON name.
// - Most handlers besides health_check, versions, configuration, fcl_status, processes, log_tail and the FQ sampler toggles were stubs.

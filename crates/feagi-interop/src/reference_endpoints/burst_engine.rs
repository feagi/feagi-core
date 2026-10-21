//! Burst engine: control, status, timing, Fire Candidate List (FCL), Fire Queue (FQ),
//! fire ledger and sampler configuration of the neural processing loop.
//! Converted from the old `burst_engine` endpoints (reference only).

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

/// Action for the burst engine control message.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BurstEngineAction {
    /// Start the burst engine
    Start,
    /// Pause the burst engine
    Pause,
    /// Stop the burst engine
    Stop,
    /// Resume the burst engine (same as start)
    Resume,
}

/// Detailed fire queue data for one cortical area.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FireQueueAreaDetail {
    /// Internal cortical index of the area
    pub cortical_idx: u32,
    /// Number of neurons that fired in this area
    pub fired_count: u64,
    /// IDs of the neurons that fired
    pub neuron_ids: Vec<u32>,
    /// X coordinates of the fired neurons
    pub coordinates_x: Vec<u32>,
    /// Y coordinates of the fired neurons
    pub coordinates_y: Vec<u32>,
    /// Z coordinates of the fired neurons
    pub coordinates_z: Vec<u32>,
    /// Membrane potentials of the fired neurons at firing time
    pub membrane_potentials: Vec<f32>,
}

define_request_message_category! {
    category_name: "BurstEngine",
    base_path: "v1/burst_engine",
    category_description: "Control and inspect the burst engine (neural processing loop): status, timing, FCL, fire queue, fire ledger and sampler settings",
    read: {
        "SimulationTimestep": {
            path: "simulation_timestep",
            description: "Get the current simulation timestep in seconds",
            response: {
                simulation_timestep: f64, "Simulation timestep in seconds (1 / burst frequency)",
            },
        },
        "Fcl": {
            path: "fcl",
            description: "Get the Fire Candidate List (FCL) content showing neurons ready to fire, grouped by cortical area (max 20 IDs per area)",
            alloc_only: true,
            response: {
                timestep: u64, "Current burst count",
                total_neurons: u64, "Total neuron IDs returned across all areas",
                cortical_areas: HashMap<String, Vec<u64>>, "Neuron IDs in the FCL keyed by cortical ID",
                default_window_size: u32, "Default fire ledger window size",
                active_cortical_count: u64, "Number of cortical areas with candidates",
            },
        },
        "FireQueue": {
            path: "fire_queue",
            description: "Get the Fire Queue (FQ) showing how many neurons fired per cortical area in the last sampled burst",
            alloc_only: true,
            response: {
                timestep: u64, "Current burst count",
                total_fired: u64, "Total neurons that fired",
                cortical_areas: HashMap<String, u64>, "Fired neuron count keyed by cortical ID",
            },
        },
        "FireQueueDetailed": {
            path: "fire_queue/detailed",
            description: "Get the detailed Fire Queue with per-area neuron IDs, coordinates and membrane potentials",
            alloc_only: true,
            response: {
                timestep: u64, "Current burst count",
                total_fired: u64, "Total neurons that fired",
                cortical_areas: HashMap<String, FireQueueAreaDetail>, "Fired neuron details keyed by cortical ID",
            },
        },
        "FclNeuron": {
            path: "fcl/neuron",
            description: "Get the current FCL candidate potential for a specific neuron",
            payload: {
                neuron_id: u64, "Neuron ID to look up in the current FCL snapshot",
            },
            response: {
                neuron_id: u64, "Neuron ID that was looked up",
                present: bool, "Whether the neuron is in the FCL",
                cortical_idx: Option<u32>, "Cortical index of the neuron, if present",
                candidate_potential: f32, "Candidate potential (0 if not present)",
            },
        },
        "FireQueueNeuron": {
            path: "fire_queue/neuron",
            description: "Check whether a specific neuron fired in the last burst (Fire Queue sample)",
            payload: {
                neuron_id: u64, "Neuron ID to check in the last Fire Queue sample",
            },
            response: {
                neuron_id: u64, "Neuron ID that was checked",
                fired: bool, "Whether the neuron fired",
                cortical_idx: Option<u32>, "Cortical index of the neuron, if it fired",
                membrane_potential_at_fire: Option<f32>, "Membrane potential when it fired",
            },
        },
        "FclStatus": {
            path: "fcl_status",
            description: "Get Fire Candidate List manager status",
            alloc_only: true,
            response: {
                available: bool, "Whether the FCL manager is available",
                error: Option<String>, "Error message if not available",
            },
        },
        "FireLedgerDefaultWindowSize": {
            path: "fire_ledger/default_window_size",
            description: "Get the default fire history window size",
            response: {
                window_size: i32, "Default window size in bursts",
            },
        },
        "FireLedgerAreasWindowConfig": {
            path: "fire_ledger/areas_window_config",
            description: "Get fire history window configuration for all cortical areas",
            alloc_only: true,
            response: {
                default_window_size: u32, "Default window size in bursts",
                areas: HashMap<String, u64>, "Window size keyed by cortical ID",
                total_configured_areas: u64, "Number of configured areas",
            },
        },
        "Stats": {
            path: "stats",
            description: "Get burst engine statistics including burst count and frequency",
            response: {
                burst_count: u64, "Total bursts since start",
                frequency_hz: f64, "Burst frequency in Hz",
                active: bool, "Whether the engine is running",
                paused: bool, "Whether the engine is paused",
            },
        },
        "Status": {
            path: "status",
            description: "Get burst engine status including active state, pause state and burst count",
            response: {
                active: bool, "Whether the engine is running",
                paused: bool, "Whether the engine is paused",
                burst_count: u64, "Total bursts since start",
                frequency_hz: f64, "Burst frequency in Hz",
            },
        },
        "FclSamplerConfig": {
            path: "fcl_sampler/config",
            description: "Get FCL/FQ sampler configuration",
            response: {
                frequency: f64, "Sampler frequency in Hz",
                consumer: u32, "Sampler consumer setting (0 = disabled)",
            },
        },
        "AreaFclSampleRate": {
            path: "fcl_sampler/area/:area_id<u32>/sample_rate",
            path_item_descriptions: {
                area_id: "Cortical area index (cortical_idx)",
            },
            description: "Get FCL sample rate for a specific cortical area",
            response: {
                sample_rate: f64, "Sample rate",
            },
        },
        "BurstCounter": {
            path: "burst_counter",
            description: "Get the total number of bursts executed since start",
            response: {
                burst_count: u64, "Total bursts since start",
            },
        },
        "Config": {
            path: "config",
            description: "Get burst engine configuration including frequency and timing settings",
            response: {
                burst_frequency_hz: f64, "Burst frequency in Hz",
                burst_interval_seconds: f64, "Seconds between bursts",
                target_frequency_hz: f64, "Target burst frequency in Hz",
                is_running: bool, "Whether the engine is running",
                is_paused: bool, "Whether the engine is paused",
            },
        },
        "FireLedgerAreaWindowSize": {
            path: "fire_ledger/area/:area_id<u32>/window_size",
            path_item_descriptions: {
                area_id: "Cortical area index (cortical_idx)",
            },
            description: "Get fire ledger window size for a specific cortical area",
            response: {
                window_size: i32, "Window size in bursts",
            },
            error: {
                AreaNotFound: "No fire ledger configuration exists for that cortical index",
            },
        },
        "FireLedgerHistory": {
            path: "fire_ledger/area/:area_id<String>/history",
            path_item_descriptions: {
                area_id: "Cortical area ID or index",
            },
            description: "Get fire ledger history for a specific cortical area",
            alloc_only: true,
            payload: {
                lookback_steps: Option<i32>, "Number of timesteps to retrieve",
            },
            response: {
                history: serde_json::Value, "Fire history (shape not defined by the old implementation)",
            },
            error: {
                InvalidAreaId: "The area ID could not be parsed",
                NotImplemented: "Fire ledger history retrieval is not implemented",
            },
        },
        "MembranePotentials": {
            path: "membrane_potentials",
            description: "Get membrane potentials for specific neurons",
            alloc_only: true,
            payload: {
                neuron_ids: String, "Comma-separated list of neuron IDs",
            },
            response: {
                potentials: HashMap<String, f32>, "Membrane potential keyed by neuron ID",
            },
        },
        "FrequencyStatus": {
            path: "frequency_status",
            description: "Get current burst frequency measurement status",
            response: {
                target_frequency_hz: f64, "Target burst frequency in Hz",
                actual_frequency_hz: f64, "Measured burst frequency in Hz",
                burst_count: u64, "Total bursts since start",
                is_measuring: bool, "Whether a measurement is in progress",
            },
        },
        "FrequencyHistory": {
            path: "frequency_history",
            description: "Get burst frequency measurement history",
            alloc_only: true,
            payload: {
                limit: Option<i32>, "Number of measurements to return (default 10)",
            },
            response: {
                measurements: Vec<serde_json::Value>, "Frequency measurements",
                limit: i32, "Limit that was applied",
            },
        },
    },
    create: {
        "SimulationTimestep": {
            path: "simulation_timestep",
            description: "Set the simulation timestep in seconds (converted to burst frequency)",
            alloc_only: true,
            payload: {
                simulation_timestep: f64, "New simulation timestep in seconds",
            },
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
        "Control": {
            path: "control",
            description: "Control the burst engine with an action: start, pause, stop or resume",
            alloc_only: true,
            payload: {
                action: BurstEngineAction, "Action to perform",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "FclSamplerConfig": {
            path: "fcl_sampler/config",
            description: "Update FCL/FQ sampler configuration; returns the updated configuration",
            payload: {
                frequency: Option<f64>, "New sampler frequency in Hz",
                consumer: Option<u32>, "New sampler consumer setting",
            },
            response: {
                frequency: f64, "Sampler frequency in Hz",
                consumer: u32, "Sampler consumer setting",
            },
        },
        "AreaFclSampleRate": {
            path: "fcl_sampler/area/:area_id<u32>/sample_rate",
            path_item_descriptions: {
                area_id: "Cortical area index (cortical_idx)",
            },
            description: "Set FCL sample rate for a specific cortical area",
            payload: {
                sample_rate: f64, "New sample rate (must be positive)",
            },
            response: {
                sample_rate: f64, "Sample rate that was set",
            },
        },
        "Start": {
            path: "start",
            description: "Start the burst engine to begin neural processing",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "Stop": {
            path: "stop",
            description: "Stop the burst engine and halt neural processing",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "Hold": {
            path: "hold",
            description: "Pause the burst engine temporarily",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "Resume": {
            path: "resume",
            description: "Resume the burst engine after a pause",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "MeasureFrequency": {
            path: "measure_frequency",
            description: "Start a burst frequency measurement",
            alloc_only: true,
            payload: {
                duration_seconds: Option<f64>, "Measurement duration in seconds (default 5)",
                sample_count: Option<i32>, "Number of samples (default 100)",
            },
            response: {
                status: String, "Measurement status",
                duration_seconds: f64, "Measurement duration in seconds",
                sample_count: i32, "Number of samples",
            },
        },
        "ForceConnectomeIntegration": {
            path: "force_connectome_integration",
            description: "Force connectome integration to rebuild neural connections immediately",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
                status: String, "Integration status",
            },
        },
    },
    edit: {
        "FireLedgerDefaultWindowSize": {
            path: "fire_ledger/default_window_size",
            description: "Set the default fire history window size for all cortical areas",
            alloc_only: true,
            payload: {
                window_size: Option<i32>, "New default window size (default 20, must be positive)",
            },
            response: {
                success: bool, "Whether the update succeeded",
                window_size: i32, "Window size that was set",
                message: String, "Confirmation message",
            },
        },
        "Config": {
            path: "config",
            description: "Update burst engine configuration; returns the updated configuration",
            payload: {
                burst_frequency_hz: Option<f64>, "New burst frequency in Hz (must be positive)",
            },
            response: {
                burst_frequency_hz: f64, "Burst frequency in Hz",
                burst_interval_seconds: f64, "Seconds between bursts",
                target_frequency_hz: f64, "Target burst frequency in Hz",
                is_running: bool, "Whether the engine is running",
                is_paused: bool, "Whether the engine is paused",
            },
        },
        "FireLedgerAreaWindowSize": {
            path: "fire_ledger/area/:area_id<u32>/window_size",
            path_item_descriptions: {
                area_id: "Cortical area index (cortical_idx)",
            },
            description: "Set fire ledger window size for a specific cortical area",
            payload: {
                window_size: i32, "New window size (must be positive)",
            },
            response: {
                success: bool, "Whether the update succeeded",
                cortical_idx: u32, "Cortical area index that was updated",
                window_size: i32, "Window size that was set",
            },
        },
        "MembranePotentials": {
            path: "membrane_potentials",
            description: "Update membrane potentials for specific neurons",
            alloc_only: true,
            payload: {
                potentials: HashMap<String, f32>, "New membrane potential keyed by neuron ID",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
    },
}

// NOTES:
// - Bare-value responses wrapped: SimulationTimestep (f64), BurstCounter (u64), FireLedgerDefaultWindowSize/AreaWindowSize (i32).
// - MembranePotentials: old GET returned a bare map (wrapped as `potentials`); old PUT took a bare map body (wrapped as `potentials`).
// - MembranePotentials GET: `neuron_ids` kept as a comma-separated String (old code never parsed it).
// - Edit FireLedgerAreaWindowSize response field `area_id` renamed to `cortical_idx` to avoid clashing with the path param.
// - FireLedgerHistory: old handler always errored (unimplemented); `history` response is an untyped Value.
// - FrequencyHistory `measurements` is Vec<Value>; old code always returned an empty list.
// - FclNeuron/FireQueueNeuron: `cortical_idx`/`membrane_potential_at_fire` only present when found, so Option.
// - Control `action` typed as an enum; old code took a String and rejected unknown actions.
// - Integer widths for counts (usize in old code) mapped to u64; FireQueueAreaDetail coordinate types guessed as u32.
// - Many old handlers were stubs (fcl_reset, fcl_status, measure_frequency, force_connectome_integration, membrane_potentials).

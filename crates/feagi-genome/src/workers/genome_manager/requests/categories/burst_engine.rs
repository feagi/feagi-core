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
    },
    edit: {
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

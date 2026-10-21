//! Simulation: simulation control and stimulation scripts.
//! Converted from the old `simulation` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

define_request_message_category! {
    category_name: "Simulation",
    base_path: "v1/simulation",
    category_description: "Simulation control and stimulation scripts",
    read: {
        "Status": {
            path: "status",
            description: "Get simulation status including active state and running scripts",
            response: {
                active: bool, "Whether the simulation is active",
                stimulation_running: bool, "Whether a stimulation script is running",
            },
        },
        "Stats": {
            path: "stats",
            description: "Get simulation statistics including total stimulations and active scripts",
            response: {
                total_stimulations: u64, "Total number of stimulations applied",
                active_scripts: u64, "Number of active stimulation scripts",
            },
        },
    },
    create: {
        "UploadString": {
            path: "upload/string",
            description: "Upload a stimulation script for neural activity simulation",
            alloc_only: true,
            payload: {
                stimulation_script: HashMap<String, serde_json::Value>, "The stimulation script",
            },
            response: {
                message: String, "Result message",
            },
        },
        "Reset": {
            path: "reset",
            description: "Reset simulation state to initial conditions",
            alloc_only: true,
            response: {
                message: String, "Result message",
            },
        },
        "Config": {
            path: "config",
            description: "Configure simulation parameters and behavior settings",
            alloc_only: true,
            payload: {
                config: HashMap<String, serde_json::Value>, "Simulation configuration to apply",
            },
            response: {
                message: String, "Result message",
            },
        },
    },
}

// NOTES:
// stimulation_script and config are untyped Value maps; the old handlers never parsed them.
// Config payload field `config` taken from SimulationControlRequest DTO; the old handler ignored the body.
// POST responses were bare HashMap<String, String> with one "message" key; mapped to `message`.
// Stats.active_scripts was usize in the DTO; changed to u64.

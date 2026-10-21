//! Input: sensory input sources, vision configuration, and a runtime tap of sensory input.
//! Converted from the old `input` endpoints (`/v1/input/*`).

use feagi_basis::feagi_basis_proc_common::define_request_message_category;

/// Single voxel sample in a sensory area.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SensorTapSample {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub potential: f32,
}

/// Per-cortical-area sensory activity captured by the sensor tap.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SensorTapArea {
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub neuron_count: usize,
    pub samples: Vec<SensorTapSample>,
}

define_request_message_category! {
    category_name: "Input",
    base_path: "v1/input",
    category_description: "Sensory input sources, their configuration, and inspection of sensory input consumed by the burst loop",
    read: {
        "Vision": {
            path: "vision",
            description: "Get vision input configuration and settings",
            alloc_only: true,
            response: {
                config: std::collections::HashMap<String, serde_json::Value>, "Vision input configuration",
            },
        },
        "Sources": {
            path: "sources",
            description: "Get list of available input sources (vision, audio, etc.)",
            alloc_only: true,
            response: {
                sources: Vec<String>, "Names of available input sources",
            },
        },
        "SensorSnapshotLast": {
            path: "sensor_snapshot/last",
            description: "Get the most recent sensory input decoded by the burst loop",
            alloc_only: true,
            payload: {
                cortical_id: Option<String>, "Filter areas to this base64 cortical ID",
            },
            response: {
                burst_num: u64, "Burst counter when the snapshot was captured; 0 if no sensory traffic received yet",
                timestamp_ms: i64, "Wall-clock millisecond timestamp when the snapshot was captured",
                has_data: bool, "True when at least one area was captured",
                total_areas: usize, "Total cortical areas with sensory input this burst",
                total_neurons: usize, "Total samples across all sensory areas this burst",
                areas: Vec<SensorTapArea>, "Per-area sensory samples, ordered as decoded by the burst loop",
            },
        },
    },
    create: {
        "Vision": {
            path: "vision",
            description: "Update vision input configuration",
            alloc_only: true,
            payload: {
                config: std::collections::HashMap<String, serde_json::Value>, "Vision input configuration",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "Configure": {
            path: "configure",
            description: "Configure input sources and their parameters",
            alloc_only: true,
            payload: {
                config: std::collections::HashMap<String, serde_json::Value>, "Input source configuration",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
    },
}

// NOTES:
// Vision (read): old handler returned an empty untyped map (TODO); wrapped as config: HashMap<String, Value>.
// Vision (create): old handler was unimplemented and ignored its untyped body; bare map now wrapped in config.
// Configure: old body was an ignored untyped map; payload guessed; old body was the bare map, now wrapped in config.
// Sources: old response was a bare Vec<String>; wrapped as sources field.
// Vision/Configure responses: old HashMap<String, String> {"message"}; wrapped as message field.
// SensorSnapshotLast: old handler body was commented out; fields from its DTO.

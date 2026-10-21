//! Outputs: output/motor target configuration and a runtime tap of motor activity.
//! Converted from the old `outputs` endpoints (`/v1/output/*`).

use feagi_basis::feagi_basis_proc_common::define_request_message_category;

/// Single voxel sample in a motor activity area.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MotorTapSample {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub potential: f32,
}

/// Per-cortical-area activity captured by the motor tap.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MotorTapArea {
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub neuron_count: usize,
    pub samples: Vec<MotorTapSample>,
}

/// Per-agent publish stats captured by the motor tap.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MotorTapAgent {
    pub agent_id: String,
    pub burst_num: u64,
    pub timestamp_ms: i64,
    pub byte_count: usize,
    pub published: bool,
    pub last_error: String,
    pub subscribed_cortical_ids: Vec<String>,
}

/// Read-only spike-cost / activity aggregate for a burst's tap snapshot.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActivitySummary {
    /// Number of areas that fired at least one neuron this burst (after any filter).
    pub active_area_count: usize,
    /// Total fired neurons across the (filtered) areas this burst.
    pub total_fired_neurons: usize,
    /// Largest single-area fired-neuron count this burst.
    pub peak_area_fired_neurons: usize,
    /// Mean fired neurons per active area (0.0 when none fired).
    pub mean_area_fired_neurons: f64,
    /// Mean potential across captured firing samples (0.0 when none).
    pub mean_sample_potential: f64,
    /// Peak potential across captured firing samples (0.0 when none).
    pub peak_sample_potential: f32,
}

define_request_message_category! {
    category_name: "Outputs",
    base_path: "v1/output",
    category_description: "Output/motor target configuration and inspection of motor output produced by the burst loop",
    read: {
        "Targets": {
            path: "targets",
            description: "Get available output targets from connected motor/output agents",
            alloc_only: true,
            response: {
                targets: Vec<String>, "IDs of agents with motor/output capabilities",
            },
        },
        "MotorSnapshotLast": {
            path: "motor_snapshot/last",
            description: "Get the most recent motor output produced by the burst loop, tapped before per-agent transport filtering",
            alloc_only: true,
            payload: {
                agent_id: Option<String>, "Filter the agents list to this agent ID",
                cortical_id: Option<String>, "Keep only this OPU (base64 cortical ID) in areas and recompute totals",
            },
            response: {
                burst_num: u64, "Burst counter when the snapshot was captured; 0 if no motor activity recorded yet",
                timestamp_ms: i64, "Wall-clock millisecond timestamp when the snapshot was captured",
                has_data: bool, "True when at least one area was captured",
                total_areas: usize, "Total motor cortical areas seen this burst",
                total_neurons: usize, "Total firing neurons across all motor areas this burst",
                activity_summary: ActivitySummary, "Spike-cost / activity-intensity aggregate over the (filtered) areas",
                areas: Vec<MotorTapArea>, "Per-area activity, ordered as captured by the burst loop",
                agents: Vec<MotorTapAgent>, "Per-agent publish summary; empty if no agent has published yet",
            },
        },
    },
    create: {
        "Configure": {
            path: "configure",
            description: "Configure output targets and motor agent connections",
            alloc_only: true,
            payload: {
                config: serde_json::Value, "Output configuration object (keyed by target)",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
    },
}

// NOTES:
// Targets: old response was a HashMap with a single "targets" key; typed as Vec<String>.
// Configure: "config" only validated as a JSON object, shape unknown -> serde_json::Value.
// Configure: old response was HashMap<String, String> {"message"}; wrapped as message field.
// MotorSnapshotLast: old handler body was commented out and returned not_implemented; fields from its DTO.

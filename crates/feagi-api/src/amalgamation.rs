// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
/*!
Genome amalgamation state (FEAGI-side).

This module implements the server-side portion of the BV/desktop amalgamation workflow:

- A client (e.g., Brain Hub) calls `POST /v1/genome/amalgamation_by_payload` to submit a genome JSON
  to be amalgamated into the currently running brain.
- FEAGI stores a single **pending** amalgamation request and surfaces it via
  `GET /v1/system/health_check` as `amalgamation_pending`.
- Brain Visualizer polls `health_check`, detects `amalgamation_pending`, and prompts the user for
  where/how to import. BV then calls `POST /v1/genome/amalgamation_destination` to confirm.
- FEAGI applies the confirmation and clears the pending state so BV can detect completion.

Design constraints:
- Deterministic: no implicit behavior changes across platforms.
- Minimal state: stored in-memory (per FEAGI session). Persistence is not currently required by BV.
- Schema compatibility: `amalgamation_pending` must include keys BV expects.
*/

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// BV-facing summary of a pending amalgamation (health_check contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmalgamationPendingSummary {
    pub amalgamation_id: String,
    pub genome_title: String,
    /// Dimensions of the imported circuit bounding box (x,y,z).
    pub circuit_size: [i32; 3],
}

/// Internal stored pending amalgamation request.
#[derive(Debug, Clone)]
pub struct AmalgamationPending {
    pub summary: AmalgamationPendingSummary,
    /// Original genome JSON payload (as string) for deterministic replay on confirmation.
    pub genome_json: String,
}

/// Historical record for completed/cancelled requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmalgamationHistoryEntry {
    pub amalgamation_id: String,
    pub genome_title: String,
    pub circuit_size: [i32; 3],
    pub status: String, // "pending" | "confirmed" | "cancelled" | "replaced"
    pub timestamp_ms: i64,
}

#[derive(Debug, Default, Clone)]
pub struct AmalgamationState {
    pub pending: Option<AmalgamationPending>,
    pub history: Vec<AmalgamationHistoryEntry>,
}

pub type SharedAmalgamationState = Arc<RwLock<AmalgamationState>>;

/// Create a fresh in-memory amalgamation state container.
pub fn new_shared_state() -> SharedAmalgamationState {
    Arc::new(RwLock::new(AmalgamationState::default()))
}

/// Compute a circuit bounding-box size (x,y,z) from a parsed RuntimeGenome.
///
/// Bounding box is computed over all cortical areas:
/// - min corner = min(position)
/// - max corner = max(position + dimensions)
/// - size = max - min (per axis)
///
/// If there are no cortical areas, size is [0,0,0].
pub fn compute_circuit_size_from_runtime_genome(
    genome: &feagi_evolutionary::RuntimeGenome,
) -> [i32; 3] {
    let mut any = false;
    let mut min_x: i32 = 0;
    let mut min_y: i32 = 0;
    let mut min_z: i32 = 0;
    let mut max_x: i32 = 0;
    let mut max_y: i32 = 0;
    let mut max_z: i32 = 0;

    for area in genome.cortical_areas.values() {
        let x0 = area.position.x;
        let y0 = area.position.y;
        let z0 = area.position.z;

        let x1 = x0.saturating_add(area.dimensions.width as i32);
        let y1 = y0.saturating_add(area.dimensions.height as i32);
        let z1 = z0.saturating_add(area.dimensions.depth as i32);

        if !any {
            any = true;
            min_x = x0;
            min_y = y0;
            min_z = z0;
            max_x = x1;
            max_y = y1;
            max_z = z1;
        } else {
            min_x = min_x.min(x0);
            min_y = min_y.min(y0);
            min_z = min_z.min(z0);
            max_x = max_x.max(x1);
            max_y = max_y.max(y1);
            max_z = max_z.max(z1);
        }
    }

    if !any {
        return [0, 0, 0];
    }

    [
        max_x.saturating_sub(min_x),
        max_y.saturating_sub(min_y),
        max_z.saturating_sub(min_z),
    ]
}

/// Convert a pending summary to the `health_check` JSON shape.
pub fn pending_summary_to_health_json(summary: &AmalgamationPendingSummary) -> Value {
    serde_json::json!({
        "amalgamation_id": summary.amalgamation_id,
        "genome_title": summary.genome_title,
        "circuit_size": summary.circuit_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_circuit_size_empty_genome_is_zero() {
        let genome =
            feagi_evolutionary::templates::create_minimal_genome("g".to_string(), "t".to_string());
        assert_eq!(compute_circuit_size_from_runtime_genome(&genome), [0, 0, 0]);
    }

    #[test]
    fn compute_circuit_size_single_area_matches_block_boundaries() {
        // One area at [1,1,1] with dims [2,3,4] -> size should be [2,3,4].
        let json = serde_json::json!({
            "genome_id": "test",
            "genome_title": "Test Genome",
            "genome_description": "",
            "version": "2.1",
            "blueprint": {
                "X19fcG93ZXI=": { // "___power" base64 (core-ish, but valid cortical id)
                    "cortical_name": "Area",
                    "block_boundaries": [2, 3, 4],
                    "relative_coordinate": [1, 1, 1],
                    "cortical_type": "CUSTOM"
                }
            },
            "brain_regions": {
                "root": {
                    "title": "Root",
                    "parent_region_id": null,
                    "coordinate_3d": [0, 0, 0],
                    "areas": ["X19fcG93ZXI="],
                    "regions": []
                }
            },
            "physiology": {
                "simulation_timestep": 0.025
            }
        })
        .to_string();

        let genome = feagi_evolutionary::load_genome_from_json(&json).expect("valid genome json");
        assert_eq!(compute_circuit_size_from_runtime_genome(&genome), [2, 3, 4]);
    }
}

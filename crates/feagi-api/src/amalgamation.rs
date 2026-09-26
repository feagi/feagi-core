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
    /// Footprint of the imported circuit's input, output, and conflict plates (x, y, z).
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

/// Plate layout constants from Brain Visualizer `UI_BrainMonitor_BrainRegion3D.gd`.
/// The import shadow uses the same plate row those constants draw.
const PLATE_PLACEHOLDER_WIDTH: i32 = 5;
const PLATE_PLACEHOLDER_DEPTH: i32 = 5;
const PLATE_AREA_BUFFER: i32 = 8;
const PLATE_SIDE_MARGIN: i32 = 2;
const PLATE_FRONT_BACK_MARGIN: i32 = 2;
const PLATE_HEIGHT: i32 = 1;
const PLATE_GAP: i32 = 1;

/// Width, depth, and thickness of one input, output, or conflict plate.
///
/// An empty plate is the placeholder. A plate with areas is wide enough for
/// their widths plus the gaps and side margins, and deep enough for the
/// deepest area plus the front and back margins. Plate thickness stays fixed.
fn plate_footprint(area_widths_and_depths: &[(i32, i32)]) -> (i32, i32) {
    if area_widths_and_depths.is_empty() {
        return (PLATE_PLACEHOLDER_WIDTH, PLATE_PLACEHOLDER_DEPTH);
    }
    let mut total_width: i32 = 0;
    let mut max_depth: i32 = 0;
    for (width, depth) in area_widths_and_depths {
        total_width = total_width.saturating_add(*width);
        max_depth = max_depth.max(*depth);
    }
    let count = area_widths_and_depths.len() as i32;
    let plate_width = total_width
        .saturating_add((count - 1).saturating_mul(PLATE_AREA_BUFFER))
        .saturating_add(PLATE_SIDE_MARGIN.saturating_mul(2));
    let plate_depth = max_depth.saturating_add(PLATE_FRONT_BACK_MARGIN.saturating_mul(2));
    (plate_width, plate_depth)
}

fn area_width_depth(
    genome: &feagi_evolutionary::RuntimeGenome,
    cortical_id_b64: &str,
) -> Option<(i32, i32)> {
    use feagi_structures::genomic::cortical_area::CorticalID;
    let cortical_id = CorticalID::try_from_base_64(cortical_id_b64).ok()?;
    let area = genome.cortical_areas.get(&cortical_id)?;
    Some((area.dimensions.width as i32, area.dimensions.depth as i32))
}

/// Import shadow size from the circuit's designated input, output, and conflict plates.
///
/// Internal areas that are not designated inputs or outputs do not change the size.
/// An area listed as both input and output is a conflict and is drawn on its own plate.
/// With no designated input or output, the size is the two empty plates side by side.
pub fn compute_circuit_size_from_runtime_genome(
    genome: &feagi_evolutionary::RuntimeGenome,
) -> [i32; 3] {
    let (designated_inputs, designated_outputs) =
        feagi_evolutionary::designated_io_lists_for_cloned_circuit(genome);
    let input_ids: std::collections::HashSet<&str> =
        designated_inputs.iter().map(String::as_str).collect();
    let output_ids: std::collections::HashSet<&str> =
        designated_outputs.iter().map(String::as_str).collect();

    let mut input_areas: Vec<(i32, i32)> = Vec::new();
    let mut output_areas: Vec<(i32, i32)> = Vec::new();
    let mut conflict_areas: Vec<(i32, i32)> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for cortical_id in designated_inputs.iter().chain(designated_outputs.iter()) {
        if !seen.insert(cortical_id.as_str()) {
            continue;
        }
        let in_inputs = input_ids.contains(cortical_id.as_str());
        let in_outputs = output_ids.contains(cortical_id.as_str());
        let Some(width_depth) = area_width_depth(genome, cortical_id) else {
            continue;
        };
        if in_inputs && in_outputs {
            conflict_areas.push(width_depth);
        } else if in_inputs {
            input_areas.push(width_depth);
        } else if in_outputs {
            output_areas.push(width_depth);
        }
    }

    let (input_width, input_depth) = plate_footprint(&input_areas);
    let (output_width, output_depth) = plate_footprint(&output_areas);
    let mut width = input_width
        .saturating_add(PLATE_GAP)
        .saturating_add(output_width);
    let mut depth = input_depth.max(output_depth);
    if !conflict_areas.is_empty() {
        let (conflict_width, conflict_depth) = plate_footprint(&conflict_areas);
        width = width
            .saturating_add(PLATE_GAP)
            .saturating_add(conflict_width);
        depth = depth.max(conflict_depth);
    }
    [width, PLATE_HEIGHT, depth]
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

    fn load_test_genome(json: serde_json::Value) -> feagi_evolutionary::RuntimeGenome {
        feagi_evolutionary::load_genome_from_json(&json.to_string()).expect("valid genome json")
    }

    #[test]
    fn compute_circuit_size_without_designated_io_is_empty_plates() {
        let genome =
            feagi_evolutionary::templates::create_minimal_genome("g".to_string(), "t".to_string());
        // Two empty plates (5 wide, 5 deep) with the 1-voxel gap between them.
        assert_eq!(
            compute_circuit_size_from_runtime_genome(&genome),
            [11, 1, 5]
        );
    }

    #[test]
    fn compute_circuit_size_ignores_undesignated_internal_areas() {
        let genome = load_test_genome(serde_json::json!({
            "genome_id": "test",
            "genome_title": "Test Genome",
            "version": "2.1",
            "blueprint": {
                "X19fcG93ZXI=": {
                    "cortical_name": "Huge internal",
                    "block_boundaries": [40, 8, 30],
                    "relative_coordinate": [100, 0, 100],
                    "cortical_type": "CUSTOM"
                }
            },
            "brain_regions": {
                "550e8400-e29b-41d4-a716-446655440000": {
                    "title": "Root Brain Region",
                    "parent_region_id": null,
                    "areas": ["X19fcG93ZXI="],
                    "regions": ["550e8400-e29b-41d4-a716-446655440001"]
                },
                "550e8400-e29b-41d4-a716-446655440001": {
                    "title": "Gate",
                    "parent_region_id": "550e8400-e29b-41d4-a716-446655440000",
                    "areas": ["X19fcG93ZXI="],
                    "designated_inputs": [],
                    "designated_outputs": []
                }
            },
            "physiology": { "simulation_timestep": 0.025 }
        }));
        assert_eq!(
            compute_circuit_size_from_runtime_genome(&genome),
            [11, 1, 5]
        );
    }

    #[test]
    fn compute_circuit_size_matches_input_and_output_plates() {
        // Input area width 2, depth 4. Output area width 1, depth 2.
        // Input plate: width 2+4=6, depth 4+4=8. Output plate: width 1+4=5, depth 2+4=6.
        // Row: 6 + 1 gap + 5 = 12 wide, deepest plate is 8.
        let genome = load_test_genome(serde_json::json!({
            "genome_id": "test",
            "genome_title": "Test Genome",
            "version": "2.1",
            "blueprint": {
                "X19fcG93ZXI=": {
                    "cortical_name": "Input",
                    "block_boundaries": [2, 3, 4],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "CUSTOM"
                },
                "X19fZGVhdGg=": {
                    "cortical_name": "Output",
                    "block_boundaries": [1, 1, 2],
                    "relative_coordinate": [50, 0, 50],
                    "cortical_type": "CUSTOM"
                },
                "X19fZmF0aWc=": {
                    "cortical_name": "Internal",
                    "block_boundaries": [40, 8, 30],
                    "relative_coordinate": [200, 0, 200],
                    "cortical_type": "CUSTOM"
                }
            },
            "brain_regions": {
                "550e8400-e29b-41d4-a716-446655440000": {
                    "title": "Root Brain Region",
                    "parent_region_id": null,
                    "areas": [],
                    "regions": ["550e8400-e29b-41d4-a716-446655440001"]
                },
                "550e8400-e29b-41d4-a716-446655440001": {
                    "title": "Gate",
                    "parent_region_id": "550e8400-e29b-41d4-a716-446655440000",
                    "areas": ["X19fcG93ZXI=", "X19fZGVhdGg=", "X19fZmF0aWc="],
                    "designated_inputs": ["X19fcG93ZXI="],
                    "designated_outputs": ["X19fZGVhdGg="]
                }
            },
            "physiology": { "simulation_timestep": 0.025 }
        }));
        assert_eq!(
            compute_circuit_size_from_runtime_genome(&genome),
            [12, 1, 8]
        );
    }

    #[test]
    fn compute_circuit_size_puts_shared_area_on_conflict_plate() {
        // The shared area leaves the input and output plates empty and adds a conflict plate.
        // Empty + gap + empty + gap + conflict(width 2+4=6, depth 4+4=8)
        // = 5+1+5+1+6 = 18 wide, depth 8.
        let genome = load_test_genome(serde_json::json!({
            "genome_id": "test",
            "genome_title": "Test Genome",
            "version": "2.1",
            "blueprint": {
                "X19fcG93ZXI=": {
                    "cortical_name": "Shared",
                    "block_boundaries": [2, 1, 4],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "CUSTOM"
                }
            },
            "brain_regions": {
                "550e8400-e29b-41d4-a716-446655440000": {
                    "title": "Root Brain Region",
                    "parent_region_id": null,
                    "areas": [],
                    "regions": ["550e8400-e29b-41d4-a716-446655440001"]
                },
                "550e8400-e29b-41d4-a716-446655440001": {
                    "title": "Gate",
                    "parent_region_id": "550e8400-e29b-41d4-a716-446655440000",
                    "areas": ["X19fcG93ZXI="],
                    "designated_inputs": ["X19fcG93ZXI="],
                    "designated_outputs": ["X19fcG93ZXI="]
                }
            },
            "physiology": { "simulation_timestep": 0.025 }
        }));
        assert_eq!(
            compute_circuit_size_from_runtime_genome(&genome),
            [18, 1, 8]
        );
    }
}

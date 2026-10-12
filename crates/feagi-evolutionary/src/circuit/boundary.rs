// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Circuit boundary resolution from region membership and cortical mappings.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::CircuitBoundary;

/// Interface role of a member area, derived from its cortical group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AreaRole {
    /// Receives activity from sensors (IPU); always a boundary input.
    Sensory,
    /// Drives actuators (OPU); always a boundary output.
    Motor,
    /// Any other area (custom, memory, core).
    Internal,
}

/// One directed cortical mapping between two areas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingEdge {
    /// Source area id.
    pub src_area_id: String,
    /// Destination area id.
    pub dst_area_id: String,
}

/// Computes the circuit boundary from member roles and the brain's mapping edges.
///
/// A member is an input when it is sensory or receives a mapping from a non-member, and an
/// output when it is motor or sends a mapping to a non-member. Self-mappings and mappings
/// between two non-members are ignored. A member may be both an input and an output.
pub fn resolve_boundary(
    members: &BTreeMap<String, AreaRole>,
    edges: &[MappingEdge],
) -> CircuitBoundary {
    let mut input_area_ids = BTreeSet::new();
    let mut output_area_ids = BTreeSet::new();
    for (area_id, role) in members {
        match role {
            AreaRole::Sensory => {
                input_area_ids.insert(area_id.clone());
            }
            AreaRole::Motor => {
                output_area_ids.insert(area_id.clone());
            }
            AreaRole::Internal => {}
        }
    }
    for edge in edges {
        let src_in = members.contains_key(&edge.src_area_id);
        let dst_in = members.contains_key(&edge.dst_area_id);
        match (src_in, dst_in) {
            (false, true) => {
                input_area_ids.insert(edge.dst_area_id.clone());
            }
            (true, false) => {
                output_area_ids.insert(edge.src_area_id.clone());
            }
            (true, true) | (false, false) => {}
        }
    }
    CircuitBoundary {
        input_area_ids,
        output_area_ids,
    }
}

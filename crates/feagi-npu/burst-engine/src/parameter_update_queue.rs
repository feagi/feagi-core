// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Asynchronous parameter update queue for cortical areas.

This queue allows non-blocking parameter updates that are applied between burst cycles,
preventing lock contention and ensuring zero impact on burst timing even at ultra-high
frequencies (100Hz+) with GPU acceleration.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// A single parameter update command
#[derive(Debug, Clone)]
pub struct ParameterUpdate {
    /// Cortical area index
    pub cortical_idx: u32,
    /// Cortical area ID (for logging)
    pub cortical_id: String,
    /// Parameter name
    pub parameter_name: String,
    /// New value
    pub value: Value,
    /// Optional cortical area dimensions (for spatial gradient updates)
    pub dimensions: Option<(u32, u32, u32)>, // (width, height, depth)
    /// Optional neurons_per_voxel (for spatial gradient updates)
    pub neurons_per_voxel: Option<u32>,
    /// Optional base_threshold (for spatial gradient updates)
    pub base_threshold: Option<f32>,
}

/// Thread-safe queue for parameter updates
///
/// ARCHITECTURE:
/// - API thread: Pushes updates (non-blocking, just mutex on queue)
/// - Burst thread: Consumes updates between bursts (when NPU is free)
///
/// PERFORMANCE:
/// - Queue operations: ~1-2µs (fast mutex, no contention)
/// - Zero impact on burst timing
/// - Works at any frequency (100Hz+, 1000Hz+)
pub struct ParameterUpdateQueue {
    queue: Arc<Mutex<VecDeque<ParameterUpdate>>>,
}

impl ParameterUpdateQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
        }
    }

    /// Push a parameter update (non-blocking, called from API thread)
    pub fn push(&self, update: ParameterUpdate) {
        self.queue.lock().unwrap().push_back(update);
    }

    /// Drain all pending updates (called from burst thread between bursts)
    pub fn drain_all(&self) -> Vec<ParameterUpdate> {
        self.queue.lock().unwrap().drain(..).collect()
    }

    /// Get queue size (for monitoring)
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }
}

impl Default for ParameterUpdateQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ParameterUpdateQueue {
    fn clone(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
        }
    }
}

/// Convert a JSON number to f32. Accepts integer or floating JSON numbers.
pub fn json_number_as_f32(value: &Value) -> Option<f32> {
    value.as_f64().map(|v| v as f32)
}

/// True when `name` is a firing-threshold increment field (vector or per-axis).
pub fn is_firing_threshold_increment_param(name: &str) -> bool {
    matches!(
        name,
        "neuron_fire_threshold_increment"
            | "firing_threshold_increment"
            | "firing_threshold_increment_x"
            | "firing_threshold_increment_y"
            | "firing_threshold_increment_z"
    )
}

/// True when `name` is the base firing-threshold field.
pub fn is_firing_threshold_param(name: &str) -> bool {
    matches!(name, "neuron_fire_threshold" | "firing_threshold")
}

/// Parse increment as `[x, y, z]`.
///
/// Accepted payloads:
/// - array of at least 3 numbers
/// - object `{x, y, z}`
/// - scalar number (applied to all three axes; matches create/persist)
pub fn parse_firing_threshold_increment(value: &Value) -> Option<[f32; 3]> {
    if let Some(arr) = value.as_array() {
        if arr.len() < 3 {
            return None;
        }
        return Some([
            json_number_as_f32(&arr[0])?,
            json_number_as_f32(&arr[1])?,
            json_number_as_f32(&arr[2])?,
        ]);
    }
    if let Some(obj) = value.as_object() {
        return Some([
            obj.get("x").and_then(json_number_as_f32)?,
            obj.get("y").and_then(json_number_as_f32)?,
            obj.get("z").and_then(json_number_as_f32)?,
        ]);
    }
    json_number_as_f32(value).map(|v| [v, v, v])
}

/// Merge increment changes onto the area's current `[x, y, z]` vector.
///
/// Vector keys overwrite all axes first; per-axis keys then override one component.
/// Returns `None` when no increment key is present or a present key cannot be parsed.
pub fn merge_firing_threshold_increment(
    current: [f32; 3],
    changes: &HashMap<String, Value>,
) -> Option<[f32; 3]> {
    let mut next = current;
    let mut touched = false;

    for key in [
        "neuron_fire_threshold_increment",
        "firing_threshold_increment",
    ] {
        if let Some(value) = changes.get(key) {
            let parsed = parse_firing_threshold_increment(value)?;
            next = parsed;
            touched = true;
        }
    }
    if let Some(value) = changes.get("firing_threshold_increment_x") {
        next[0] = json_number_as_f32(value)?;
        touched = true;
    }
    if let Some(value) = changes.get("firing_threshold_increment_y") {
        next[1] = json_number_as_f32(value)?;
        touched = true;
    }
    if let Some(value) = changes.get("firing_threshold_increment_z") {
        next[2] = json_number_as_f32(value)?;
        touched = true;
    }

    if touched {
        Some(next)
    } else {
        None
    }
}

/// Stored neuron thresholds must be rewritten when increment changes, or when
/// the base threshold changes on an area that already has a non-zero gradient.
pub fn should_rewrite_threshold_gradient(
    increment_changed: bool,
    threshold_changed: bool,
    increment: [f32; 3],
) -> bool {
    increment_changed || (threshold_changed && increment.iter().any(|v| *v != 0.0))
}

/// Extract `(base, [inc_x, inc_y, inc_z])` from a queued increment update.
pub fn increment_from_parameter_update(update: &ParameterUpdate) -> Option<(f32, [f32; 3])> {
    let increment = parse_firing_threshold_increment(&update.value)?;
    let base = update.base_threshold?;
    Some((base, increment))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_accepts_float_array() {
        assert_eq!(
            parse_firing_threshold_increment(&json!([0.015, 0.0, 1.5])),
            Some([0.015, 0.0, 1.5])
        );
    }

    #[test]
    fn parse_accepts_integer_array() {
        assert_eq!(
            parse_firing_threshold_increment(&json!([10, 0, 100])),
            Some([10.0, 0.0, 100.0])
        );
    }

    #[test]
    fn parse_accepts_object() {
        assert_eq!(
            parse_firing_threshold_increment(&json!({"x": 10, "y": 0, "z": 0})),
            Some([10.0, 0.0, 0.0])
        );
    }

    #[test]
    fn parse_scalar_applies_to_all_axes() {
        assert_eq!(
            parse_firing_threshold_increment(&json!(10)),
            Some([10.0, 10.0, 10.0])
        );
    }

    #[test]
    fn parse_rejects_short_array() {
        assert_eq!(parse_firing_threshold_increment(&json!([10, 0])), None);
    }

    #[test]
    fn parse_rejects_non_numeric() {
        assert_eq!(parse_firing_threshold_increment(&json!(["10", 0, 0])), None);
    }

    #[test]
    fn merge_per_axis_keeps_other_axes() {
        let current = [1.0, 2.0, 3.0];
        let mut changes = HashMap::new();
        changes.insert("firing_threshold_increment_x".to_string(), json!(10));
        assert_eq!(
            merge_firing_threshold_increment(current, &changes),
            Some([10.0, 2.0, 3.0])
        );
    }

    #[test]
    fn merge_vector_then_per_axis_override() {
        let current = [0.0, 0.0, 0.0];
        let mut changes = HashMap::new();
        changes.insert(
            "neuron_fire_threshold_increment".to_string(),
            json!([10, 0, 0]),
        );
        changes.insert("firing_threshold_increment_z".to_string(), json!(100));
        assert_eq!(
            merge_firing_threshold_increment(current, &changes),
            Some([10.0, 0.0, 100.0])
        );
    }

    #[test]
    fn merge_returns_none_when_increment_absent() {
        let mut changes = HashMap::new();
        changes.insert("neuron_fire_threshold".to_string(), json!(1.0));
        assert_eq!(
            merge_firing_threshold_increment([0.0, 0.0, 0.0], &changes),
            None
        );
    }

    #[test]
    fn rewrite_when_increment_changes_or_base_changes_with_gradient() {
        assert!(should_rewrite_threshold_gradient(
            true,
            false,
            [0.0, 0.0, 0.0]
        ));
        assert!(should_rewrite_threshold_gradient(
            false,
            true,
            [10.0, 0.0, 0.0]
        ));
        assert!(!should_rewrite_threshold_gradient(
            false,
            true,
            [0.0, 0.0, 0.0]
        ));
        assert!(!should_rewrite_threshold_gradient(
            false,
            false,
            [10.0, 0.0, 0.0]
        ));
    }

    #[test]
    fn increment_from_update_requires_base_and_vector() {
        let update = ParameterUpdate {
            cortical_idx: 1,
            cortical_id: "area".to_string(),
            parameter_name: "neuron_fire_threshold_increment".to_string(),
            value: json!([10, 0, 0]),
            dimensions: None,
            neurons_per_voxel: None,
            base_threshold: Some(0.01),
        };
        assert_eq!(
            increment_from_parameter_update(&update),
            Some((0.01, [10.0, 0.0, 0.0]))
        );

        let missing_base = ParameterUpdate {
            base_threshold: None,
            ..update.clone()
        };
        assert_eq!(increment_from_parameter_update(&missing_base), None);
    }
}

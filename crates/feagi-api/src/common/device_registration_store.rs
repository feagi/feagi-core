// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Compact device-registration store comparison for auto-create diagnostics.
//!
//! FEAGI's poll loop prefers descriptor-scoped registrations over the live
//! session payload. Session-only APIs (`capabilities/all`) hide that split, so
//! leftover `SegmentedVision` in the descriptor store can recreate `isvi`
//! areas after the connected agent has already switched to simple `Vision`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap, HashSet};

#[cfg(feature = "feagi-agent")]
use feagi_agent::{AgentCapabilities, AgentDescriptor};
#[cfg(feature = "feagi-agent")]
use feagi_io::AgentID;

const INPUT_UNITS_KEY: &str = "input_units_and_encoder_properties";
const OUTPUT_UNITS_KEY: &str = "output_units_and_decoder_properties";
const VISION_KEY: &str = "Vision";
const SEGMENTED_VISION_KEY: &str = "SegmentedVision";

/// Compact I/O inventory extracted from a device-registration JSON blob.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RegistrationIoSummary {
    /// Sensory unit type keys (`Vision`, `SegmentedVision`, `Servo`, ...).
    pub input_unit_keys: Vec<String>,
    /// Motor unit type keys (`PositionalServo`, `RotaryMotor`, ...).
    pub output_unit_keys: Vec<String>,
    /// `cortical_unit_index` values under `Vision`.
    pub vision_groups: Vec<u16>,
    /// Friendly names for simple-vision groups, aligned with `vision_groups`.
    pub vision_names: Vec<String>,
    /// `cortical_unit_index` values under `SegmentedVision`.
    pub segmented_vision_groups: Vec<u16>,
    /// Friendly names for segmented-vision groups, aligned with
    /// `segmented_vision_groups`.
    pub segmented_vision_names: Vec<String>,
}

/// One agent row in the registration-store comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeviceRegistrationStoreRow {
    /// Live session id (base64), when a connection is currently registered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Session id last bound onto this descriptor (survives deregister).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor_bound_session_id: Option<String>,
    pub manufacturer: String,
    pub agent_name: String,
    pub agent_version: u32,
    /// Which store the auto-create poll uses: `descriptor` if present, else `session`.
    pub poll_source: String,
    pub live_session: bool,
    pub descriptor_present: bool,
    pub session_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<RegistrationIoSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<RegistrationIoSummary>,
    /// True when compact session and descriptor inventories differ, or when an
    /// orphan descriptor remains with no live session.
    pub mismatch: bool,
    pub keys_only_in_descriptor: Vec<String>,
    pub keys_only_in_session: Vec<String>,
    /// Descriptor still advertises `SegmentedVision` while the live session does not.
    pub segmented_vision_only_in_descriptor: bool,
}

/// Compact comparison of session vs descriptor device-registration stores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeviceRegistrationStoreResponse {
    pub count: usize,
    pub mismatch_count: usize,
    /// How many rows have `segmented_vision_only_in_descriptor`.
    pub segmented_vision_descriptor_stale_count: usize,
    pub agents: Vec<DeviceRegistrationStoreRow>,
}

/// Extract unit-type keys and vision group indexes from a registration JSON object.
pub fn summarize_device_registrations(value: &Value) -> RegistrationIoSummary {
    let inputs = value.get(INPUT_UNITS_KEY).and_then(Value::as_object);
    let outputs = value.get(OUTPUT_UNITS_KEY).and_then(Value::as_object);
    let input_unit_keys = object_keys(inputs);
    let output_unit_keys = object_keys(outputs);
    let (vision_groups, vision_names) = vision_groups_from_inputs(inputs, VISION_KEY);
    let (segmented_vision_groups, segmented_vision_names) =
        vision_groups_from_inputs(inputs, SEGMENTED_VISION_KEY);
    RegistrationIoSummary {
        input_unit_keys,
        output_unit_keys,
        vision_groups,
        vision_names,
        segmented_vision_groups,
        segmented_vision_names,
    }
}

fn object_keys(object: Option<&serde_json::Map<String, Value>>) -> Vec<String> {
    let Some(object) = object else {
        return Vec::new();
    };
    let mut keys: Vec<String> = object.keys().cloned().collect();
    keys.sort();
    keys
}

fn vision_groups_from_inputs(
    inputs: Option<&serde_json::Map<String, Value>>,
    key: &str,
) -> (Vec<u16>, Vec<String>) {
    let Some(inputs) = inputs else {
        return (Vec::new(), Vec::new());
    };
    let Some(entries) = inputs.get(key).and_then(Value::as_array) else {
        return (Vec::new(), Vec::new());
    };
    let mut groups: Vec<(u16, String)> = Vec::new();
    for entry in entries {
        let Some(pair) = entry.as_array() else {
            continue;
        };
        let Some(unit_def) = pair.first().and_then(Value::as_object) else {
            continue;
        };
        let Some(index) = unit_def
            .get("cortical_unit_index")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
        else {
            continue;
        };
        let name = unit_def
            .get("friendly_name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        groups.push((index, name));
    }
    groups.sort_by_key(|(index, _)| *index);
    let mut indexes = Vec::with_capacity(groups.len());
    let mut names = Vec::with_capacity(groups.len());
    for (index, name) in groups {
        indexes.push(index);
        names.push(name);
    }
    (indexes, names)
}

fn union_io_keys(summary: &RegistrationIoSummary) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    keys.extend(summary.input_unit_keys.iter().cloned());
    keys.extend(summary.output_unit_keys.iter().cloned());
    keys
}

fn key_diff(
    session: Option<&RegistrationIoSummary>,
    descriptor: Option<&RegistrationIoSummary>,
) -> (Vec<String>, Vec<String>, bool) {
    let session_keys = session.map(union_io_keys).unwrap_or_default();
    let descriptor_keys = descriptor.map(union_io_keys).unwrap_or_default();
    let keys_only_in_descriptor: Vec<String> =
        descriptor_keys.difference(&session_keys).cloned().collect();
    let keys_only_in_session: Vec<String> =
        session_keys.difference(&descriptor_keys).cloned().collect();
    let segmented_vision_only_in_descriptor = descriptor
        .map(|summary| !summary.segmented_vision_groups.is_empty())
        .unwrap_or(false)
        && session
            .map(|summary| summary.segmented_vision_groups.is_empty())
            .unwrap_or(true);
    (
        keys_only_in_descriptor,
        keys_only_in_session,
        segmented_vision_only_in_descriptor,
    )
}

fn summaries_differ(
    session: Option<&RegistrationIoSummary>,
    descriptor: Option<&RegistrationIoSummary>,
) -> bool {
    match (session, descriptor) {
        (Some(session_summary), Some(descriptor_summary)) => session_summary != descriptor_summary,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (None, None) => false,
    }
}

fn poll_source(descriptor_present: bool) -> &'static str {
    if descriptor_present {
        "descriptor"
    } else {
        "session"
    }
}

#[allow(clippy::too_many_arguments)]
fn build_row(
    session_id: Option<String>,
    descriptor_bound_session_id: Option<String>,
    manufacturer: String,
    agent_name: String,
    agent_version: u32,
    live_session: bool,
    session_value: Option<&Value>,
    descriptor_value: Option<&Value>,
) -> DeviceRegistrationStoreRow {
    let session = session_value.map(summarize_device_registrations);
    let descriptor = descriptor_value.map(summarize_device_registrations);
    let descriptor_present = descriptor_value.is_some();
    let session_present = session_value.is_some();
    let (keys_only_in_descriptor, keys_only_in_session, segmented_vision_only_in_descriptor) =
        key_diff(session.as_ref(), descriptor.as_ref());
    let mismatch = summaries_differ(session.as_ref(), descriptor.as_ref());
    DeviceRegistrationStoreRow {
        session_id,
        descriptor_bound_session_id,
        manufacturer,
        agent_name,
        agent_version,
        poll_source: poll_source(descriptor_present).to_string(),
        live_session,
        descriptor_present,
        session_present,
        session,
        descriptor,
        mismatch,
        keys_only_in_descriptor,
        keys_only_in_session,
        segmented_vision_only_in_descriptor,
    }
}

/// Build the compact store comparison the auto-create poll actually uses.
#[cfg(feature = "feagi-agent")]
pub fn build_device_registration_store_compare(
    live_agents: &HashMap<AgentID, (AgentDescriptor, Vec<AgentCapabilities>)>,
    by_descriptor: &HashMap<AgentDescriptor, Value>,
    by_agent: &HashMap<AgentID, Value>,
    descriptor_session_ids: &HashMap<AgentDescriptor, String>,
) -> DeviceRegistrationStoreResponse {
    let mut agents: Vec<DeviceRegistrationStoreRow> = Vec::new();
    let mut used_descriptors: HashSet<AgentDescriptor> = HashSet::new();
    let mut used_sessions: HashSet<AgentID> = HashSet::new();

    for (session_id, (descriptor, _)) in live_agents {
        used_descriptors.insert(descriptor.clone());
        used_sessions.insert(*session_id);
        agents.push(build_row(
            Some(session_id.to_base64()),
            descriptor_session_ids.get(descriptor).cloned(),
            descriptor.manufacturer().to_string(),
            descriptor.agent_name().to_string(),
            descriptor.agent_version(),
            true,
            by_agent.get(session_id),
            by_descriptor.get(descriptor),
        ));
    }

    for (descriptor, value) in by_descriptor {
        if used_descriptors.contains(descriptor) {
            continue;
        }
        agents.push(build_row(
            None,
            descriptor_session_ids.get(descriptor).cloned(),
            descriptor.manufacturer().to_string(),
            descriptor.agent_name().to_string(),
            descriptor.agent_version(),
            false,
            None,
            Some(value),
        ));
    }

    for (session_id, value) in by_agent {
        if used_sessions.contains(session_id) {
            continue;
        }
        agents.push(build_row(
            Some(session_id.to_base64()),
            None,
            String::new(),
            String::new(),
            0,
            false,
            Some(value),
            None,
        ));
    }

    agents.sort_by(|left, right| {
        left.agent_name
            .cmp(&right.agent_name)
            .then(left.session_id.cmp(&right.session_id))
    });
    let mismatch_count = agents.iter().filter(|row| row.mismatch).count();
    let segmented_vision_descriptor_stale_count = agents
        .iter()
        .filter(|row| row.segmented_vision_only_in_descriptor)
        .count();
    DeviceRegistrationStoreResponse {
        count: agents.len(),
        mismatch_count,
        segmented_vision_descriptor_stale_count,
        agents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn simple_vision_regs() -> Value {
        json!({
            "input_units_and_encoder_properties": {
                "Vision": [[
                    {
                        "friendly_name": "scene_cam",
                        "cortical_unit_index": 0
                    },
                    {"CartesianPlane": {}}
                ]],
                "Servo": []
            },
            "output_units_and_decoder_properties": {
                "PositionalServo": []
            }
        })
    }

    fn segmented_vision_regs() -> Value {
        json!({
            "input_units_and_encoder_properties": {
                "SegmentedVision": [[
                    {
                        "friendly_name": "hand_bottom",
                        "cortical_unit_index": 0
                    },
                    {}
                ]]
            },
            "output_units_and_decoder_properties": {
                "PositionalServo": []
            }
        })
    }

    #[test]
    fn summarize_extracts_vision_and_segmented_groups() {
        let simple = summarize_device_registrations(&simple_vision_regs());
        assert_eq!(simple.input_unit_keys, vec!["Servo", "Vision"]);
        assert_eq!(simple.vision_groups, vec![0]);
        assert_eq!(simple.vision_names, vec!["scene_cam"]);
        assert!(simple.segmented_vision_groups.is_empty());

        let segmented = summarize_device_registrations(&segmented_vision_regs());
        assert_eq!(segmented.input_unit_keys, vec!["SegmentedVision"]);
        assert_eq!(segmented.segmented_vision_groups, vec![0]);
        assert_eq!(segmented.segmented_vision_names, vec!["hand_bottom"]);
    }

    #[test]
    fn key_diff_flags_stale_segmented_vision_on_descriptor() {
        let session = summarize_device_registrations(&simple_vision_regs());
        let descriptor = summarize_device_registrations(&segmented_vision_regs());
        let (only_descriptor, only_session, stale_segmented) =
            key_diff(Some(&session), Some(&descriptor));
        assert!(only_descriptor.contains(&"SegmentedVision".to_string()));
        assert!(only_session.contains(&"Vision".to_string()));
        assert!(stale_segmented);
        assert!(summaries_differ(Some(&session), Some(&descriptor)));
    }

    #[cfg(feature = "feagi-agent")]
    #[test]
    fn compare_marks_orphan_descriptor_as_mismatch() {
        let descriptor = AgentDescriptor::new("neuraville", "Lite6", 1).expect("descriptor");
        let mut by_descriptor = HashMap::new();
        by_descriptor.insert(descriptor.clone(), segmented_vision_regs());
        let mut descriptor_session_ids = HashMap::new();
        descriptor_session_ids.insert(descriptor, "old-session".to_string());
        let live: HashMap<AgentID, (AgentDescriptor, Vec<AgentCapabilities>)> = HashMap::new();
        let by_agent: HashMap<AgentID, Value> = HashMap::new();
        let result = build_device_registration_store_compare(
            &live,
            &by_descriptor,
            &by_agent,
            &descriptor_session_ids,
        );
        assert_eq!(result.count, 1);
        assert_eq!(result.mismatch_count, 1);
        assert_eq!(result.segmented_vision_descriptor_stale_count, 1);
        assert_eq!(result.agents[0].poll_source, "descriptor");
        assert!(!result.agents[0].live_session);
        assert!(result.agents[0].segmented_vision_only_in_descriptor);
    }
}

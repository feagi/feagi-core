// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Before/after state captured around a change, read through the service traits.
//!
//! A snapshot is `None` when the element does not exist at that moment (before a
//! create, after a delete).

use crate::traits::ConnectomeService;
use serde_json::{Map, Value};

const MAPPING_KEY: &str = "cortical_mapping_dst";

/// Full property map of one cortical area.
pub async fn area_snapshot(reads: &dyn ConnectomeService, cortical_id: &str) -> Option<Value> {
    reads
        .get_cortical_area_properties(cortical_id)
        .await
        .ok()
        .map(|properties| Value::Object(properties.into_iter().collect()))
}

/// Property maps of several areas keyed by cortical id; `None` when none exist.
pub async fn areas_snapshot(reads: &dyn ConnectomeService, cortical_ids: &[&str]) -> Option<Value> {
    let mut areas = Map::new();
    for id in cortical_ids {
        if let Some(snapshot) = area_snapshot(reads, id).await {
            areas.insert((*id).to_string(), snapshot);
        }
    }
    (!areas.is_empty()).then_some(Value::Object(areas))
}

/// Area properties plus every other area's mapping rules that target it, so a
/// delete can be fully reverted.
pub async fn area_with_incoming_snapshot(
    reads: &dyn ConnectomeService,
    cortical_id: &str,
) -> Option<Value> {
    let area = area_snapshot(reads, cortical_id).await?;
    let mut incoming = Map::new();
    if let Ok(ids) = reads.get_cortical_area_ids().await {
        for src_id in ids.iter().filter(|id| id.as_str() != cortical_id) {
            if let Some(rules) = mapping_rules(reads, src_id, cortical_id).await {
                incoming.insert(src_id.clone(), rules);
            }
        }
    }
    let mut snapshot = Map::new();
    snapshot.insert("area".to_string(), area);
    snapshot.insert("incoming_mappings".to_string(), Value::Object(incoming));
    Some(Value::Object(snapshot))
}

/// Mapping rules from `src_id` to `dst_id`; `None` when no mapping exists.
pub async fn mapping_rules(
    reads: &dyn ConnectomeService,
    src_id: &str,
    dst_id: &str,
) -> Option<Value> {
    let properties = reads.get_cortical_area_properties(src_id).await.ok()?;
    properties
        .get(MAPPING_KEY)
        .and_then(|mappings| mappings.get(dst_id))
        .filter(|rules| rules.as_array().is_none_or(|list| !list.is_empty()))
        .cloned()
}

/// Brain region as returned by the service.
pub async fn region_snapshot(reads: &dyn ConnectomeService, region_id: &str) -> Option<Value> {
    let region = reads.get_brain_region(region_id).await.ok()?;
    serde_json::to_value(region).ok()
}

/// Morphology definition.
pub async fn morphology_snapshot(
    reads: &dyn ConnectomeService,
    morphology_id: &str,
) -> Option<Value> {
    let mut morphologies = reads.get_morphologies().await.ok()?;
    let morphology = morphologies.remove(morphology_id)?;
    serde_json::to_value(morphology).ok()
}

/// Classifier record.
pub async fn classifier_snapshot(
    reads: &dyn ConnectomeService,
    classifier_id: &str,
) -> Option<Value> {
    let classifier = reads.get_classifier(classifier_id).await.ok()?;
    serde_json::to_value(classifier).ok()
}

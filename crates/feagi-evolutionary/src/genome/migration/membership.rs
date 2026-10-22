// Copyright 2026 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Brain-region membership fold: legacy `areas` -> `cortical_areas`.
//!
//! New genome and connectome saves write only `cortical_areas`. Documents
//! that still have `areas`, or both keys, are converted here. When both
//! lists are present, `cortical_areas` keeps its order and ids that appear
//! only in `areas` are appended.
//!
//! A malformed (non-array) membership value cannot be merged, so the fold
//! discards it. Those outcomes are reported separately so callers can
//! surface them instead of losing membership silently.

use serde_json::{Map, Value};

/// What folding one region's membership did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipFold {
    /// No legacy `areas` key; the region was not touched.
    Unchanged,
    /// `areas` was merged into `cortical_areas` and removed.
    Merged,
    /// `areas` was not an array. It was removed without adding members.
    DroppedNonArrayAreas,
    /// `cortical_areas` was not an array. It was replaced by the `areas` list.
    ReplacedNonArrayCorticalAreas,
}

impl MembershipFold {
    /// True when the region object changed.
    pub fn changed(self) -> bool {
        self != Self::Unchanged
    }

    /// Diagnostic for outcomes that discarded a membership value.
    pub fn data_loss_note(self, region_id: &str) -> Option<String> {
        match self {
            Self::Unchanged | Self::Merged => None,
            Self::DroppedNonArrayAreas => Some(format!(
                "brain region {region_id}: dropped legacy areas because it was not an array"
            )),
            Self::ReplacedNonArrayCorticalAreas => Some(format!(
                "brain region {region_id}: replaced non-array cortical_areas with the legacy areas list"
            )),
        }
    }
}

/// Outcome of folding every brain region in a genome.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GenomeMembershipFold {
    /// Number of region objects that changed.
    pub folded: usize,
    /// One note per region whose membership value was discarded.
    pub data_loss: Vec<String>,
}

/// Move a brain region's legacy `areas` list onto `cortical_areas`.
pub fn fold_region_membership_into_cortical_areas(
    region: &mut Map<String, Value>,
) -> MembershipFold {
    let Some(legacy) = region.remove("areas") else {
        return MembershipFold::Unchanged;
    };
    let Value::Array(legacy_ids) = legacy else {
        return MembershipFold::DroppedNonArrayAreas;
    };
    let membership = region
        .entry("cortical_areas".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    let Some(canonical) = membership.as_array_mut() else {
        *membership = Value::Array(legacy_ids);
        return MembershipFold::ReplacedNonArrayCorticalAreas;
    };
    for id in legacy_ids {
        if !canonical.iter().any(|existing| existing == &id) {
            canonical.push(id);
        }
    }
    MembershipFold::Merged
}

/// Fold every brain region's legacy `areas` key into `cortical_areas`.
pub fn fold_genome_region_membership(genome: &mut Value) -> GenomeMembershipFold {
    let mut outcome = GenomeMembershipFold::default();
    let Some(regions) = genome
        .get_mut("brain_regions")
        .and_then(|value| value.as_object_mut())
    else {
        return outcome;
    };
    for (region_id, region) in regions.iter_mut() {
        let Some(region) = region.as_object_mut() else {
            continue;
        };
        let fold = fold_region_membership_into_cortical_areas(region);
        if fold.changed() {
            outcome.folded += 1;
        }
        if let Some(note) = fold.data_loss_note(region_id) {
            outcome.data_loss.push(note);
        }
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fold_keeps_cortical_areas_order_and_appends_legacy_only_ids() {
        let mut region = Map::new();
        region.insert("areas".to_string(), json!(["_power", "_death", "_power"]));
        region.insert("cortical_areas".to_string(), json!(["_death", "_death"]));

        assert_eq!(
            fold_region_membership_into_cortical_areas(&mut region),
            MembershipFold::Merged
        );
        assert!(region.get("areas").is_none());
        assert_eq!(
            region.get("cortical_areas"),
            Some(&json!(["_death", "_death", "_power"]))
        );
    }

    #[test]
    fn fold_moves_an_areas_only_list() {
        let mut region = Map::new();
        region.insert("areas".to_string(), json!(["_power"]));

        assert_eq!(
            fold_region_membership_into_cortical_areas(&mut region),
            MembershipFold::Merged
        );
        assert!(region.get("areas").is_none());
        assert_eq!(region.get("cortical_areas"), Some(&json!(["_power"])));
    }

    #[test]
    fn fold_leaves_a_cortical_areas_only_region_unchanged() {
        let mut region = Map::new();
        region.insert("cortical_areas".to_string(), json!(["_death"]));

        let fold = fold_region_membership_into_cortical_areas(&mut region);
        assert_eq!(fold, MembershipFold::Unchanged);
        assert!(!fold.changed());
        assert_eq!(region.get("cortical_areas"), Some(&json!(["_death"])));
    }

    #[test]
    fn fold_drops_a_non_array_areas_value_and_reports_it() {
        let mut region = Map::new();
        region.insert("areas".to_string(), json!("_power"));
        region.insert("cortical_areas".to_string(), json!(["_death"]));

        let fold = fold_region_membership_into_cortical_areas(&mut region);
        assert_eq!(fold, MembershipFold::DroppedNonArrayAreas);
        assert!(fold.changed());
        assert!(fold.data_loss_note("root").unwrap().contains("root"));
        assert!(region.get("areas").is_none());
        assert_eq!(region.get("cortical_areas"), Some(&json!(["_death"])));
    }

    #[test]
    fn fold_replaces_a_non_array_cortical_areas_value_and_reports_it() {
        let mut region = Map::new();
        region.insert("areas".to_string(), json!(["_power"]));
        region.insert("cortical_areas".to_string(), json!({"bad": true}));

        let fold = fold_region_membership_into_cortical_areas(&mut region);
        assert_eq!(fold, MembershipFold::ReplacedNonArrayCorticalAreas);
        assert!(fold
            .data_loss_note("root")
            .unwrap()
            .contains("cortical_areas"));
        assert!(region.get("areas").is_none());
        assert_eq!(region.get("cortical_areas"), Some(&json!(["_power"])));
    }

    #[test]
    fn fold_drops_an_empty_areas_list_without_adding_members() {
        let mut region = Map::new();
        region.insert("areas".to_string(), json!([]));
        region.insert("cortical_areas".to_string(), json!(["_death"]));

        let fold = fold_region_membership_into_cortical_areas(&mut region);
        assert_eq!(fold, MembershipFold::Merged);
        assert!(fold.data_loss_note("root").is_none());
        assert!(region.get("areas").is_none());
        assert_eq!(region.get("cortical_areas"), Some(&json!(["_death"])));
    }

    #[test]
    fn fold_genome_skips_missing_regions_and_non_objects() {
        assert_eq!(
            fold_genome_region_membership(&mut json!({})),
            GenomeMembershipFold::default()
        );
        let mut genome = json!({
            "brain_regions": {
                "root": { "areas": [] },
                "note": "not a region"
            }
        });
        let outcome = fold_genome_region_membership(&mut genome);
        assert_eq!(outcome.folded, 1);
        assert!(outcome.data_loss.is_empty());
        assert!(genome["brain_regions"]["root"].get("areas").is_none());
        assert_eq!(genome["brain_regions"]["root"]["cortical_areas"], json!([]));
        assert_eq!(genome["brain_regions"]["note"], json!("not a region"));
    }

    #[test]
    fn fold_genome_collects_one_data_loss_note_per_lossy_region() {
        let mut genome = json!({
            "brain_regions": {
                "clean": { "areas": ["_power"] },
                "bad_areas": { "areas": 7, "cortical_areas": ["_death"] },
                "bad_canonical": { "areas": ["_power"], "cortical_areas": "x" }
            }
        });
        let outcome = fold_genome_region_membership(&mut genome);
        assert_eq!(outcome.folded, 3);
        assert_eq!(outcome.data_loss.len(), 2);
        assert!(outcome.data_loss.iter().any(|n| n.contains("bad_areas")));
        assert!(outcome
            .data_loss
            .iter()
            .any(|n| n.contains("bad_canonical")));
    }
}

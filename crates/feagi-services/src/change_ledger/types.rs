// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Change ledger data model.
//!
//! Every successful structural genome mutation becomes one [`GenomeChange`]. The
//! operation payload mirrors the service call that produced it, so an entry can be
//! replayed verbatim on another FEAGI instance through the same service layer.

use crate::types::{CreateBrainRegionParams, CreateCorticalAreaParams};
use feagi_structures::genomic::classifiers::Classifier;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Where a recorded change came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOrigin {
    /// Made against this instance (API, agent registration, MCP, BV).
    Local,
    /// Applied from another instance via the replay API.
    Replayed,
}

/// Kind of genome element a change touches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeTargetKind {
    CorticalArea,
    CorticalMapping,
    BrainRegion,
    Morphology,
    Classifier,
    Genome,
    Connectome,
}

/// One genome element touched by a change.
///
/// For `CorticalMapping`, `id` is the source area and `related_id` the destination.
/// For a morphology rename, `id` is the old name and `related_id` the new name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeTarget {
    pub kind: ChangeTargetKind,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_id: Option<String>,
}

impl ChangeTarget {
    /// Target with a single identifier.
    pub fn new(kind: ChangeTargetKind, id: impl Into<String>) -> Self {
        Self {
            kind,
            id: id.into(),
            related_id: None,
        }
    }

    /// Target identified by a pair (mapping source/destination, rename old/new).
    pub fn pair(kind: ChangeTargetKind, id: impl Into<String>, related: impl Into<String>) -> Self {
        Self {
            kind,
            id: id.into(),
            related_id: Some(related.into()),
        }
    }
}

/// Mutation payload; one variant per mutating service call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GenomeChangeOperation {
    CreateCorticalAreas {
        params: Vec<CreateCorticalAreaParams>,
    },
    UpdateCorticalArea {
        cortical_id: String,
        changes: HashMap<String, Value>,
    },
    DeleteCorticalArea {
        cortical_id: String,
    },
    CreateBrainRegion {
        params: Box<CreateBrainRegionParams>,
    },
    UpdateBrainRegion {
        region_id: String,
        properties: HashMap<String, Value>,
    },
    DeleteBrainRegion {
        region_id: String,
    },
    CreateMorphology {
        morphology_id: String,
        morphology: Box<feagi_evolutionary::Morphology>,
    },
    UpdateMorphology {
        morphology_id: String,
        morphology: Box<feagi_evolutionary::Morphology>,
    },
    DeleteMorphology {
        morphology_id: String,
    },
    RenameMorphology {
        old_id: String,
        new_id: String,
    },
    /// An empty `mapping_data` deletes the mapping.
    UpdateCorticalMapping {
        src_area_id: String,
        dst_area_id: String,
        mapping_data: Vec<Value>,
    },
    UpsertClassifier {
        classifier: Box<Classifier>,
    },
    DeleteClassifier {
        classifier_id: String,
    },
    RekeyMemoryTwinSource {
        memory_area_id: String,
        old_src_area_id: String,
        new_src_area_id: String,
    },
    /// Marker: a whole genome replaced the live brain. Not replayable.
    GenomeLoaded {
        genome_id: String,
        genome_title: String,
    },
    /// Marker: the connectome was cleared. Not replayable.
    ConnectomeReset,
    /// Marker: a connectome snapshot replaced the live brain. Not replayable.
    ConnectomeImported,
}

impl GenomeChangeOperation {
    /// Whether the operation can be applied on another instance.
    ///
    /// Markers describe whole-brain replacement; peers converge on those by loading
    /// the same genome, not by replay.
    pub fn is_replayable(&self) -> bool {
        !matches!(
            self,
            Self::GenomeLoaded { .. } | Self::ConnectomeReset | Self::ConnectomeImported
        )
    }

    /// Genome elements this operation touches.
    pub fn targets(&self) -> Vec<ChangeTarget> {
        use ChangeTargetKind as K;
        match self {
            Self::CreateCorticalAreas { params } => params
                .iter()
                .map(|p| ChangeTarget::new(K::CorticalArea, p.cortical_id.clone()))
                .collect(),
            Self::UpdateCorticalArea { cortical_id, .. }
            | Self::DeleteCorticalArea { cortical_id } => {
                vec![ChangeTarget::new(K::CorticalArea, cortical_id.clone())]
            }
            Self::CreateBrainRegion { params } => {
                vec![ChangeTarget::new(K::BrainRegion, params.region_id.clone())]
            }
            Self::UpdateBrainRegion { region_id, .. } | Self::DeleteBrainRegion { region_id } => {
                vec![ChangeTarget::new(K::BrainRegion, region_id.clone())]
            }
            Self::CreateMorphology { morphology_id, .. }
            | Self::UpdateMorphology { morphology_id, .. }
            | Self::DeleteMorphology { morphology_id } => {
                vec![ChangeTarget::new(K::Morphology, morphology_id.clone())]
            }
            Self::RenameMorphology { old_id, new_id } => {
                vec![ChangeTarget::pair(
                    K::Morphology,
                    old_id.clone(),
                    new_id.clone(),
                )]
            }
            Self::UpdateCorticalMapping {
                src_area_id,
                dst_area_id,
                ..
            } => vec![ChangeTarget::pair(
                K::CorticalMapping,
                src_area_id.clone(),
                dst_area_id.clone(),
            )],
            Self::UpsertClassifier { classifier } => vec![ChangeTarget::new(
                K::Classifier,
                classifier.classifier_id.clone(),
            )],
            Self::DeleteClassifier { classifier_id } => {
                vec![ChangeTarget::new(K::Classifier, classifier_id.clone())]
            }
            Self::RekeyMemoryTwinSource {
                memory_area_id,
                old_src_area_id,
                new_src_area_id,
            } => vec![
                ChangeTarget::new(K::CorticalArea, memory_area_id.clone()),
                ChangeTarget::pair(
                    K::CorticalMapping,
                    old_src_area_id.clone(),
                    new_src_area_id.clone(),
                ),
            ],
            Self::GenomeLoaded { genome_id, .. } => {
                vec![ChangeTarget::new(K::Genome, genome_id.clone())]
            }
            Self::ConnectomeReset | Self::ConnectomeImported => {
                vec![ChangeTarget::new(K::Connectome, String::new())]
            }
        }
    }
}

/// One recorded genome change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeChange {
    /// Monotonic position in this process's ledger (polling cursor). Local to the
    /// process; not meaningful in a persisted genome or on another instance.
    pub sequence: u64,
    /// Globally unique, time-ordered identifier. Preserved across replay, so it is
    /// the idempotency key on every instance.
    pub change_id: String,
    /// Changes produced by one request (for example a clone) share a group id.
    pub group_id: String,
    /// When the change was originally made (Unix ms, UTC).
    pub timestamp_ms: i64,
    /// When this instance recorded it (Unix ms, UTC).
    pub recorded_at_ms: i64,
    /// Who made the change, when the caller identified itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub origin: ChangeOrigin,
    pub replayable: bool,
    pub targets: Vec<ChangeTarget>,
    pub operation: GenomeChangeOperation,
    /// State of the targets before the change, when it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    /// State of the targets after the change, when it still exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
}

/// A page of ledger entries returned by [`super::ChangeLedger::changes_since`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesPage {
    pub changes: Vec<GenomeChange>,
    /// Highest sequence recorded so far (0 when the ledger is empty).
    pub latest_sequence: u64,
    /// Lowest sequence still held in memory, when any entry is held.
    pub oldest_available_sequence: Option<u64>,
    /// True when entries after the requested cursor were evicted before this read.
    /// The caller has missed changes and must resynchronize from the full genome.
    pub gap: bool,
    /// True when more entries exist after the last one returned.
    pub has_more: bool,
}

/// A change submitted for replay. Extra fields (such as `sequence` or `origin`
/// from another instance's ledger) are ignored, so a [`GenomeChange`] read from a
/// peer can be submitted as-is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyGenomeChangeRequest {
    pub change_id: String,
    pub group_id: String,
    pub timestamp_ms: i64,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub operation: GenomeChangeOperation,
}

/// Result of a replay request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApplyGenomeChangeOutcome {
    /// Applied and recorded at `sequence`.
    Applied { sequence: u64 },
    /// The change id was already present; nothing was done.
    AlreadyApplied,
}

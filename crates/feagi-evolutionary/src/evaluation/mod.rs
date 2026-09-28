// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! `GenomeEvaluation` v1 — one evaluated individual: a genome, its fitness, and its lineage.
//!
//! A completed Trainer protocol produces one record (ADR-016 in
//! `docs/FEAGI_TRAINER_ADR_SET.md`). The record pins the content hash of the genome that was
//! evaluated (the genotype), references the protocol's Trainer scorecards by id, carries the
//! comparability key that decides which evaluations may be ranked against each other, the
//! validation-split fitness, and the lineage used by evolutionary operators.
//!
//! Inheritance is genome-only: lineage links genome hashes, never trained connectomes. The
//! record performs no I/O; hosts (Composer for feagi-desktop) store it.
//!
//! @cursor:ffi-safe — plain data, serde-only, no runtime reflection.

use std::collections::BTreeSet;

use feagi_dataset_contracts::{
    BackendKind, ConnectomeHash, ContentHash, DatasetAssetId, EvaluationProtocolVersion, PluginRef,
    ScorecardId, SplitId,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(test)]
mod tests;

/// Wire/format version of the `GenomeEvaluation` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Prefix of every content hash minted for genome snapshots and run configurations.
const SHA256_PREFIX: &str = "sha256:";
/// Hex digits in a SHA-256 digest.
const SHA256_HEX_LEN: usize = 64;

/// Identifies one `GenomeEvaluation` record.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EvaluationId(pub String);

/// Identifies the experiment (the environment genomes compete in).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExperimentId(pub String);

/// Direction in which a fitness metric improves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitnessObjective {
    /// Higher values are better (e.g. accuracy).
    Maximize,
    /// Lower values are better (e.g. error rate).
    Minimize,
}

/// Which metric, on which split, is the fitness, and which way it improves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FitnessSpec {
    /// Metric key as it appears in the scorecard `metrics` map.
    pub metric: String,
    /// Split the fitness is read from (validation; never the held-out test split).
    pub split_id: SplitId,
    /// Improvement direction.
    pub objective: FitnessObjective,
}

/// Everything that must match for two evaluations to be ranked against each other.
///
/// Two evaluations are comparable exactly when their keys are equal (`==`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparabilityKey {
    /// Experiment the genome was evaluated in.
    pub experiment_id: ExperimentId,
    /// Dataset asset scored against.
    pub dataset_asset_id: DatasetAssetId,
    /// Dataset version string.
    pub dataset_version: String,
    /// Content hash binding the evaluation to exact dataset bytes and labels.
    pub dataset_content_hash: ContentHash,
    /// Evaluation protocol semantics version.
    pub evaluation_protocol_version: EvaluationProtocolVersion,
    /// Metric pack that computed the metrics.
    pub metric_pack: PluginRef,
    /// Reward policy used during training.
    pub reward_policy: PluginRef,
    /// Fitness metric, split, and objective.
    pub fitness: FitnessSpec,
    /// Hash of the run settings (sample caps, class remap, bindings, burst frequency).
    pub run_config_hash: ContentHash,
    /// Genome schema version the genome was authored under.
    pub genome_schema_version: u32,
    /// feagi-core version of the runtime that ran the protocol.
    pub feagi_core_version: String,
    /// Execution backend.
    pub backend: BackendKind,
}

/// Confidence interval of a repeated (N-seed) fitness estimate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound.
    pub low: f64,
    /// Upper bound.
    pub high: f64,
    /// Confidence level in the open interval (0, 1), e.g. 0.95.
    pub level: f64,
}

/// Fitness value with its repeat provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitnessEstimate {
    /// Point estimate (the mean when `n > 1`).
    pub value: f64,
    /// Number of fresh-development repeats the estimate is over.
    pub n: u32,
    /// Required when `n > 1`; absent when `n == 1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<ConfidenceInterval>,
}

/// Whether the protocol produced a fitness value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FitnessOutcome {
    /// The fitness split ran to completion.
    Scored(FitnessEstimate),
    /// The protocol had no phase on the fitness split.
    NoFitnessSplit,
    /// The fitness split was skipped or ended early.
    Incomplete,
}

/// How a genome came to exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenomeOrigin {
    /// Authored or edited by an operator.
    Manual,
    /// Brought in from outside the experiment (e.g. Brain Hub).
    Imported,
    /// Produced by mutating exactly one parent genome.
    Mutation,
    /// Produced by recombining two or more parent genomes.
    Crossover,
}

/// Genome-only ancestry of the evaluated genome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    /// How the genome was produced.
    pub origin: GenomeOrigin,
    /// Generation number; 0 for manual and imported genomes.
    pub generation: u32,
    /// Content hashes of parent genomes (never connectomes).
    pub parents: Vec<ContentHash>,
}

/// One evaluated individual (ADR-016).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenomeEvaluation {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Identity of this record.
    pub evaluation_id: EvaluationId,
    /// Content hash of the genome snapshot taken at protocol start.
    pub genome_hash: ContentHash,
    /// Comparability key.
    pub key: ComparabilityKey,
    /// Scorecards produced by the protocol's phases.
    pub scorecard_ids: Vec<ScorecardId>,
    /// Fitness outcome on `key.fitness.split_id`.
    pub fitness: FitnessOutcome,
    /// Genome-only ancestry.
    pub lineage: Lineage,
    /// Starting connectome, present only when the operator pinned this run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_connectome: Option<ConnectomeHash>,
}

/// Contract violation found by [`GenomeEvaluation::validate`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvaluationError {
    /// `schema_version` is not [`SCHEMA_VERSION`].
    #[error("unsupported schema_version {found}; expected {expected}")]
    SchemaVersion {
        /// Version on the record.
        found: u32,
        /// Version this crate reads.
        expected: u32,
    },
    /// A required string field is empty or whitespace.
    #[error("field `{0}` must not be empty")]
    EmptyField(&'static str),
    /// A content hash is not `sha256:` followed by 64 lowercase hex digits.
    #[error("field `{field}` is not a sha256 content hash: {value}")]
    InvalidHash {
        /// Field holding the hash.
        field: &'static str,
        /// Offending value.
        value: String,
    },
    /// No scorecard ids were recorded.
    #[error("scorecard_ids must not be empty")]
    NoScorecards,
    /// A scorecard id appears more than once.
    #[error("duplicate scorecard id {0}")]
    DuplicateScorecard(String),
    /// The fitness estimate breaks an invariant.
    #[error("invalid fitness: {0}")]
    InvalidFitness(String),
    /// The lineage breaks an invariant.
    #[error("invalid lineage: {0}")]
    InvalidLineage(String),
}

impl GenomeEvaluation {
    /// Returns the fitness estimate when this individual may take part in selection.
    ///
    /// Only fully scored evaluations are selectable; `NoFitnessSplit` and `Incomplete`
    /// records stay in history but are not plotted as fitness points or used as parents.
    pub fn selectable_fitness(&self) -> Option<&FitnessEstimate> {
        match &self.fitness {
            FitnessOutcome::Scored(estimate) => Some(estimate),
            FitnessOutcome::NoFitnessSplit | FitnessOutcome::Incomplete => None,
        }
    }

    /// Checks every contract invariant; hosts call this before storing a record.
    ///
    /// # Errors
    /// Returns the first [`EvaluationError`] found.
    pub fn validate(&self) -> Result<(), EvaluationError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(EvaluationError::SchemaVersion {
                found: self.schema_version,
                expected: SCHEMA_VERSION,
            });
        }
        require_text("evaluation_id", &self.evaluation_id.0)?;
        require_sha256("genome_hash", &self.genome_hash)?;
        validate_key(&self.key)?;
        validate_scorecards(&self.scorecard_ids)?;
        if let FitnessOutcome::Scored(estimate) = &self.fitness {
            validate_estimate(estimate)?;
        }
        validate_lineage(&self.lineage, &self.genome_hash)?;
        if let Some(connectome) = &self.pinned_connectome {
            require_text("pinned_connectome", &connectome.0)?;
        }
        Ok(())
    }
}

fn require_text(field: &'static str, value: &str) -> Result<(), EvaluationError> {
    if value.trim().is_empty() {
        return Err(EvaluationError::EmptyField(field));
    }
    Ok(())
}

fn require_sha256(field: &'static str, hash: &ContentHash) -> Result<(), EvaluationError> {
    let digest_ok = hash.0.strip_prefix(SHA256_PREFIX).is_some_and(|hex| {
        hex.len() == SHA256_HEX_LEN
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    });
    if !digest_ok {
        return Err(EvaluationError::InvalidHash {
            field,
            value: hash.0.clone(),
        });
    }
    Ok(())
}

fn validate_key(key: &ComparabilityKey) -> Result<(), EvaluationError> {
    require_text("key.experiment_id", &key.experiment_id.0)?;
    require_text("key.dataset_asset_id", &key.dataset_asset_id.0)?;
    require_text("key.dataset_version", &key.dataset_version)?;
    require_text("key.dataset_content_hash", &key.dataset_content_hash.0)?;
    require_text(
        "key.evaluation_protocol_version",
        &key.evaluation_protocol_version.0,
    )?;
    require_text("key.metric_pack.id", &key.metric_pack.id.0)?;
    require_text("key.metric_pack.version", &key.metric_pack.version)?;
    require_text("key.reward_policy.id", &key.reward_policy.id.0)?;
    require_text("key.reward_policy.version", &key.reward_policy.version)?;
    require_text("key.fitness.metric", &key.fitness.metric)?;
    require_text("key.fitness.split_id", &key.fitness.split_id.0)?;
    require_sha256("key.run_config_hash", &key.run_config_hash)?;
    require_text("key.feagi_core_version", &key.feagi_core_version)?;
    Ok(())
}

fn validate_scorecards(ids: &[ScorecardId]) -> Result<(), EvaluationError> {
    if ids.is_empty() {
        return Err(EvaluationError::NoScorecards);
    }
    let mut seen = BTreeSet::new();
    for id in ids {
        require_text("scorecard_ids[]", &id.0)?;
        if !seen.insert(id.0.as_str()) {
            return Err(EvaluationError::DuplicateScorecard(id.0.clone()));
        }
    }
    Ok(())
}

fn validate_estimate(estimate: &FitnessEstimate) -> Result<(), EvaluationError> {
    if !estimate.value.is_finite() {
        return Err(EvaluationError::InvalidFitness(
            "value must be finite".to_string(),
        ));
    }
    match (estimate.n, &estimate.interval) {
        (0, _) => Err(EvaluationError::InvalidFitness(
            "n must be at least 1".to_string(),
        )),
        (1, None) => Ok(()),
        (1, Some(_)) => Err(EvaluationError::InvalidFitness(
            "a single run carries no interval".to_string(),
        )),
        (_, None) => Err(EvaluationError::InvalidFitness(
            "a repeated estimate (n > 1) requires an interval".to_string(),
        )),
        (_, Some(interval)) => validate_interval(estimate.value, interval),
    }
}

fn validate_interval(value: f64, interval: &ConfidenceInterval) -> Result<(), EvaluationError> {
    let finite = interval.low.is_finite() && interval.high.is_finite();
    if !finite || interval.low > value || value > interval.high {
        return Err(EvaluationError::InvalidFitness(
            "interval must be finite and contain the value".to_string(),
        ));
    }
    if !(interval.level > 0.0 && interval.level < 1.0) {
        return Err(EvaluationError::InvalidFitness(
            "interval level must be in (0, 1)".to_string(),
        ));
    }
    Ok(())
}

fn validate_lineage(lineage: &Lineage, genome_hash: &ContentHash) -> Result<(), EvaluationError> {
    for parent in &lineage.parents {
        require_sha256("lineage.parents[]", parent)?;
    }
    let unique: BTreeSet<&str> = lineage.parents.iter().map(|p| p.0.as_str()).collect();
    if unique.len() != lineage.parents.len() {
        return Err(EvaluationError::InvalidLineage(
            "parents must be distinct".to_string(),
        ));
    }
    if unique.contains(genome_hash.0.as_str()) {
        return Err(EvaluationError::InvalidLineage(
            "a genome cannot be its own parent".to_string(),
        ));
    }
    let parent_count = lineage.parents.len();
    match lineage.origin {
        GenomeOrigin::Manual | GenomeOrigin::Imported => {
            if parent_count != 0 || lineage.generation != 0 {
                return Err(EvaluationError::InvalidLineage(
                    "manual and imported genomes are generation 0 with no parents".to_string(),
                ));
            }
        }
        GenomeOrigin::Mutation => {
            if parent_count != 1 || lineage.generation == 0 {
                return Err(EvaluationError::InvalidLineage(
                    "a mutation has exactly one parent and generation >= 1".to_string(),
                ));
            }
        }
        GenomeOrigin::Crossover => {
            if parent_count < 2 || lineage.generation == 0 {
                return Err(EvaluationError::InvalidLineage(
                    "a crossover has at least two parents and generation >= 1".to_string(),
                ));
            }
        }
    }
    Ok(())
}

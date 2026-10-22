// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! `CircuitObjectiveSpec` v1 — a measurable, circuit-level objective for one brain region.
//!
//! A circuit is a brain region including its descendant regions. The spec pins the region's
//! membership and boundary at definition time, the stimulus protocol, the fitness metric, the
//! resource constraints, and the parameters an optimizer may change. It is the portable contract
//! shared by local and cloud evaluation (`docs/CIRCUIT_OPTIMIZER_ARCHITECTURE_AND_DESIGN.md`,
//! Section 5). Every metric carries a fixed improvement direction, so a metric cannot be paired
//! with the wrong objective.
//!
//! The spec performs no I/O; hosts build it from live session data and store it.
//!
//! @cursor:ffi-safe — plain data, serde-only, no runtime reflection.

mod boundary;

pub use boundary::{resolve_boundary, AreaRole, MappingEdge};

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evaluation::FitnessObjective;

#[cfg(test)]
mod tests;

/// Wire/format version of the `CircuitObjectiveSpec` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Voxel coordinate `[x, y, z]` inside a cortical area.
pub type Voxel = [u32; 3];

/// Areas through which the circuit exchanges activity with the rest of the brain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBoundary {
    /// Member areas that receive activity from outside the circuit (or from sensors).
    pub input_area_ids: BTreeSet<String>,
    /// Member areas that send activity outside the circuit (or to actuators).
    pub output_area_ids: BTreeSet<String>,
}

/// Which voxels of an area a stimulus drives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VoxelSelection {
    /// Every voxel of the area.
    All,
    /// An explicit, non-empty set of voxels.
    Listed {
        /// Voxels to drive.
        voxels: Vec<Voxel>,
    },
}

/// One stimulus applied to a boundary input area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stimulus {
    /// Stimulus class. Several stimuli may share a label to drive multiple areas at once.
    pub label: String,
    /// Boundary input area receiving the stimulus.
    pub area_id: String,
    /// Voxels driven in that area.
    pub voxels: VoxelSelection,
}

/// How the circuit is driven during one evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StimulusProtocol {
    /// Stimuli; may be empty only for metrics that observe spontaneous activity.
    pub stimuli: Vec<Stimulus>,
    /// Bursts observed per trial.
    pub bursts_per_trial: u32,
    /// Trials per stimulus label.
    pub trials_per_stimulus: u32,
    /// Reset the circuit's neural state before each trial.
    pub reset_between_trials: bool,
}

/// Fitness metric for the circuit. Each variant has a fixed improvement direction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CircuitMetric {
    /// Bursts from stimulus onset to first firing in a boundary output area (minimize).
    ResponseLatency {
        /// Boundary output area observed.
        output_area_id: String,
    },
    /// How distinctly a boundary output area responds to different stimulus labels (maximize).
    Selectivity {
        /// Boundary output area observed.
        output_area_id: String,
    },
    /// Distance of a member area's firing rate from a target band (minimize).
    FiringRateBand {
        /// Member area observed.
        area_id: String,
        /// Lower edge of the band, in hertz.
        min_hz: f64,
        /// Upper edge of the band, in hertz.
        max_hz: f64,
    },
    /// Distance of a boundary output area's active-neuron fraction from a target (minimize).
    ActiveFraction {
        /// Boundary output area observed.
        output_area_id: String,
        /// Target fraction of neurons that fire, in `[0, 1]`.
        target: f64,
    },
    /// Match between a boundary output area's firing and a target pattern per label (maximize).
    OutputPatternMatch {
        /// Boundary output area observed.
        output_area_id: String,
        /// Target voxels per stimulus label; every stimulus label needs a non-empty target.
        targets: BTreeMap<String, Vec<Voxel>>,
    },
}

impl CircuitMetric {
    /// Direction in which this metric improves.
    pub fn objective(&self) -> FitnessObjective {
        match self {
            Self::Selectivity { .. } | Self::OutputPatternMatch { .. } => {
                FitnessObjective::Maximize
            }
            Self::ResponseLatency { .. }
            | Self::FiringRateBand { .. }
            | Self::ActiveFraction { .. } => FitnessObjective::Minimize,
        }
    }

    /// Minimum number of distinct stimulus labels the metric needs.
    fn required_labels(&self) -> usize {
        match self {
            Self::Selectivity { .. } => 2,
            Self::ResponseLatency { .. } | Self::OutputPatternMatch { .. } => 1,
            Self::FiringRateBand { .. } | Self::ActiveFraction { .. } => 0,
        }
    }
}

/// Resource limits a candidate circuit must respect.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CircuitConstraints {
    /// Maximum neurons in the circuit; `None` means unconstrained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_neuron_count: Option<u64>,
    /// Maximum synapses in the circuit; `None` means unconstrained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_synapse_count: Option<u64>,
}

/// Neural parameter an optimizer may change in a member area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitParameter {
    /// Membrane potential at which a neuron fires.
    FiringThreshold,
    /// Fraction of membrane potential lost per burst.
    LeakCoefficient,
    /// Bursts a neuron stays silent after firing (integer).
    RefractoryPeriod,
    /// Bursts a neuron is silenced after reaching its consecutive fire limit (integer).
    SnoozePeriod,
    /// Consecutive bursts a neuron may fire before snoozing (integer).
    ConsecutiveFireCount,
    /// Postsynaptic potential delivered per synapse.
    PostsynapticPotential,
    /// Upper limit of the postsynaptic potential.
    PostsynapticPotentialMax,
    /// Probability-like excitability scaling of firing.
    NeuronExcitability,
}

impl CircuitParameter {
    /// Whether the parameter takes whole, non-negative values.
    pub fn is_integer(self) -> bool {
        matches!(
            self,
            Self::RefractoryPeriod | Self::SnoozePeriod | Self::ConsecutiveFireCount
        )
    }
}

/// Allowed range of one parameter in one member area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterRange {
    /// Member area the parameter belongs to.
    pub area_id: String,
    /// Parameter that may change.
    pub parameter: CircuitParameter,
    /// Inclusive lower bound.
    pub min: f64,
    /// Inclusive upper bound.
    pub max: f64,
}

/// A measurable objective for one circuit (brain region).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitObjectiveSpec {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Brain region the circuit is rooted at.
    pub region_id: String,
    /// Cortical areas in the region and its descendants when the spec was defined.
    pub member_area_ids: BTreeSet<String>,
    /// Boundary of the circuit.
    pub boundary: CircuitBoundary,
    /// How the circuit is driven.
    pub stimulus: StimulusProtocol,
    /// Fitness metric.
    pub metric: CircuitMetric,
    /// Resource limits.
    pub constraints: CircuitConstraints,
    /// Parameters an optimizer may change; empty for analysis-only objectives.
    pub search_space: Vec<ParameterRange>,
    /// Keep mappings that cross the circuit boundary unchanged.
    pub interface_frozen: bool,
}

/// Contract violation found by [`CircuitObjectiveSpec::validate`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CircuitSpecError {
    /// `schema_version` is not [`SCHEMA_VERSION`].
    #[error("unsupported schema_version {found}; expected {expected}")]
    SchemaVersion {
        /// Version on the spec.
        found: u32,
        /// Version this crate reads.
        expected: u32,
    },
    /// A required string field is empty or whitespace.
    #[error("field `{0}` must not be empty")]
    EmptyField(&'static str),
    /// The circuit has no member areas.
    #[error("the circuit has no member areas")]
    NoMembers,
    /// The boundary breaks an invariant.
    #[error("invalid boundary: {0}")]
    InvalidBoundary(String),
    /// An area is referenced where it is not allowed.
    #[error("area `{area_id}` is not {expected}")]
    AreaNotAllowed {
        /// Offending area.
        area_id: String,
        /// Set the area had to belong to.
        expected: &'static str,
    },
    /// The stimulus protocol breaks an invariant.
    #[error("invalid stimulus protocol: {0}")]
    InvalidStimulus(String),
    /// The metric breaks an invariant.
    #[error("invalid metric: {0}")]
    InvalidMetric(String),
    /// A constraint breaks an invariant.
    #[error("invalid constraint: {0}")]
    InvalidConstraint(String),
    /// The search space breaks an invariant.
    #[error("invalid search space: {0}")]
    InvalidSearchSpace(String),
}

impl CircuitObjectiveSpec {
    /// Direction in which the spec's fitness improves.
    pub fn objective(&self) -> FitnessObjective {
        self.metric.objective()
    }

    /// Checks every contract invariant; hosts call this before storing or running a spec.
    ///
    /// # Errors
    /// Returns the first [`CircuitSpecError`] found.
    pub fn validate(&self) -> Result<(), CircuitSpecError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(CircuitSpecError::SchemaVersion {
                found: self.schema_version,
                expected: SCHEMA_VERSION,
            });
        }
        require_text("region_id", &self.region_id)?;
        validate_members(&self.member_area_ids)?;
        validate_boundary(&self.boundary, &self.member_area_ids)?;
        validate_stimulus(&self.stimulus, &self.boundary)?;
        validate_metric(self)?;
        validate_constraints(&self.constraints)?;
        validate_search_space(&self.search_space, &self.member_area_ids)?;
        Ok(())
    }
}

fn require_text(field: &'static str, value: &str) -> Result<(), CircuitSpecError> {
    if value.trim().is_empty() {
        return Err(CircuitSpecError::EmptyField(field));
    }
    Ok(())
}

fn require_in(
    area_id: &str,
    set: &BTreeSet<String>,
    expected: &'static str,
) -> Result<(), CircuitSpecError> {
    if !set.contains(area_id) {
        return Err(CircuitSpecError::AreaNotAllowed {
            area_id: area_id.to_string(),
            expected,
        });
    }
    Ok(())
}

fn validate_members(members: &BTreeSet<String>) -> Result<(), CircuitSpecError> {
    if members.is_empty() {
        return Err(CircuitSpecError::NoMembers);
    }
    for area_id in members {
        require_text("member_area_ids[]", area_id)?;
    }
    Ok(())
}

fn validate_boundary(
    boundary: &CircuitBoundary,
    members: &BTreeSet<String>,
) -> Result<(), CircuitSpecError> {
    if boundary.input_area_ids.is_empty() {
        return Err(CircuitSpecError::InvalidBoundary(
            "at least one input area is required".to_string(),
        ));
    }
    if boundary.output_area_ids.is_empty() {
        return Err(CircuitSpecError::InvalidBoundary(
            "at least one output area is required".to_string(),
        ));
    }
    for area_id in &boundary.input_area_ids {
        require_in(area_id, members, "a member area")?;
    }
    for area_id in &boundary.output_area_ids {
        require_in(area_id, members, "a member area")?;
    }
    Ok(())
}

fn validate_voxels(voxels: &VoxelSelection, context: &str) -> Result<(), CircuitSpecError> {
    if let VoxelSelection::Listed { voxels } = voxels {
        if voxels.is_empty() {
            return Err(CircuitSpecError::InvalidStimulus(format!(
                "{context}: a listed voxel selection must not be empty"
            )));
        }
        let unique: BTreeSet<&Voxel> = voxels.iter().collect();
        if unique.len() != voxels.len() {
            return Err(CircuitSpecError::InvalidStimulus(format!(
                "{context}: voxels must be distinct"
            )));
        }
    }
    Ok(())
}

fn validate_stimulus(
    protocol: &StimulusProtocol,
    boundary: &CircuitBoundary,
) -> Result<(), CircuitSpecError> {
    if protocol.bursts_per_trial == 0 {
        return Err(CircuitSpecError::InvalidStimulus(
            "bursts_per_trial must be at least 1".to_string(),
        ));
    }
    if protocol.trials_per_stimulus == 0 {
        return Err(CircuitSpecError::InvalidStimulus(
            "trials_per_stimulus must be at least 1".to_string(),
        ));
    }
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    for stimulus in &protocol.stimuli {
        require_text("stimulus.stimuli[].label", &stimulus.label)?;
        require_in(
            &stimulus.area_id,
            &boundary.input_area_ids,
            "a boundary input area",
        )?;
        validate_voxels(&stimulus.voxels, &stimulus.label)?;
        if !seen.insert((stimulus.label.as_str(), stimulus.area_id.as_str())) {
            return Err(CircuitSpecError::InvalidStimulus(format!(
                "label `{}` drives area `{}` more than once",
                stimulus.label, stimulus.area_id
            )));
        }
    }
    Ok(())
}

fn stimulus_labels(protocol: &StimulusProtocol) -> BTreeSet<&str> {
    protocol.stimuli.iter().map(|s| s.label.as_str()).collect()
}

fn validate_metric(spec: &CircuitObjectiveSpec) -> Result<(), CircuitSpecError> {
    let metric = &spec.metric;
    let labels = stimulus_labels(&spec.stimulus);
    let required = metric.required_labels();
    if labels.len() < required {
        return Err(CircuitSpecError::InvalidMetric(format!(
            "this metric needs at least {required} distinct stimulus label(s); found {}",
            labels.len()
        )));
    }
    let outputs = &spec.boundary.output_area_ids;
    match metric {
        CircuitMetric::ResponseLatency { output_area_id }
        | CircuitMetric::Selectivity { output_area_id } => {
            require_in(output_area_id, outputs, "a boundary output area")
        }
        CircuitMetric::FiringRateBand {
            area_id,
            min_hz,
            max_hz,
        } => {
            require_in(area_id, &spec.member_area_ids, "a member area")?;
            let valid = min_hz.is_finite() && max_hz.is_finite() && *min_hz >= 0.0;
            if !valid || min_hz > max_hz {
                return Err(CircuitSpecError::InvalidMetric(
                    "firing rate band must be finite with 0 <= min_hz <= max_hz".to_string(),
                ));
            }
            Ok(())
        }
        CircuitMetric::ActiveFraction {
            output_area_id,
            target,
        } => {
            require_in(output_area_id, outputs, "a boundary output area")?;
            if !(0.0..=1.0).contains(target) {
                return Err(CircuitSpecError::InvalidMetric(
                    "active fraction target must be in [0, 1]".to_string(),
                ));
            }
            Ok(())
        }
        CircuitMetric::OutputPatternMatch {
            output_area_id,
            targets,
        } => {
            require_in(output_area_id, outputs, "a boundary output area")?;
            validate_pattern_targets(targets, &labels)
        }
    }
}

fn validate_pattern_targets(
    targets: &BTreeMap<String, Vec<Voxel>>,
    labels: &BTreeSet<&str>,
) -> Result<(), CircuitSpecError> {
    let target_labels: BTreeSet<&str> = targets.keys().map(String::as_str).collect();
    if &target_labels != labels {
        return Err(CircuitSpecError::InvalidMetric(
            "output pattern targets must cover exactly the stimulus labels".to_string(),
        ));
    }
    for (label, voxels) in targets {
        if voxels.is_empty() {
            return Err(CircuitSpecError::InvalidMetric(format!(
                "target pattern for `{label}` must not be empty"
            )));
        }
        let unique: BTreeSet<&Voxel> = voxels.iter().collect();
        if unique.len() != voxels.len() {
            return Err(CircuitSpecError::InvalidMetric(format!(
                "target pattern for `{label}` has duplicate voxels"
            )));
        }
    }
    Ok(())
}

fn validate_constraints(constraints: &CircuitConstraints) -> Result<(), CircuitSpecError> {
    if constraints.max_neuron_count == Some(0) {
        return Err(CircuitSpecError::InvalidConstraint(
            "max_neuron_count must be at least 1 when set".to_string(),
        ));
    }
    if constraints.max_synapse_count == Some(0) {
        return Err(CircuitSpecError::InvalidConstraint(
            "max_synapse_count must be at least 1 when set".to_string(),
        ));
    }
    Ok(())
}

fn validate_search_space(
    ranges: &[ParameterRange],
    members: &BTreeSet<String>,
) -> Result<(), CircuitSpecError> {
    let mut seen: BTreeSet<(&str, CircuitParameter)> = BTreeSet::new();
    for range in ranges {
        require_in(&range.area_id, members, "a member area")?;
        if !seen.insert((range.area_id.as_str(), range.parameter)) {
            return Err(CircuitSpecError::InvalidSearchSpace(format!(
                "{:?} in `{}` appears more than once",
                range.parameter, range.area_id
            )));
        }
        if !range.min.is_finite() || !range.max.is_finite() || range.min > range.max {
            return Err(CircuitSpecError::InvalidSearchSpace(format!(
                "{:?} in `{}` needs finite bounds with min <= max",
                range.parameter, range.area_id
            )));
        }
        let whole = range.min.fract() == 0.0 && range.max.fract() == 0.0;
        if range.parameter.is_integer() && (!whole || range.min < 0.0) {
            return Err(CircuitSpecError::InvalidSearchSpace(format!(
                "{:?} in `{}` takes whole, non-negative bounds",
                range.parameter, range.area_id
            )));
        }
    }
    Ok(())
}

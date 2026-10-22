// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the `CircuitObjectiveSpec` v1 contract and boundary resolution.

use super::*;

fn ids(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|v| v.to_string()).collect()
}

fn stimulus(label: &str, area_id: &str) -> Stimulus {
    Stimulus {
        label: label.to_string(),
        area_id: area_id.to_string(),
        voxels: VoxelSelection::All,
    }
}

/// Region `r1` with members in → hid → out; `in` is the input, `out` the output.
fn spec() -> CircuitObjectiveSpec {
    CircuitObjectiveSpec {
        schema_version: SCHEMA_VERSION,
        region_id: "r1".to_string(),
        member_area_ids: ids(&["in", "hid", "out"]),
        boundary: CircuitBoundary {
            input_area_ids: ids(&["in"]),
            output_area_ids: ids(&["out"]),
        },
        stimulus: StimulusProtocol {
            stimuli: vec![stimulus("a", "in"), stimulus("b", "in")],
            bursts_per_trial: 10,
            trials_per_stimulus: 3,
            reset_between_trials: true,
        },
        metric: CircuitMetric::Selectivity {
            output_area_id: "out".to_string(),
        },
        constraints: CircuitConstraints::default(),
        search_space: vec![ParameterRange {
            area_id: "hid".to_string(),
            parameter: CircuitParameter::FiringThreshold,
            min: 0.5,
            max: 2.0,
        }],
        interface_frozen: true,
    }
}

#[test]
fn valid_spec_passes() {
    assert_eq!(spec().validate(), Ok(()));
}

#[test]
fn schema_version_is_enforced() {
    let mut s = spec();
    s.schema_version = SCHEMA_VERSION + 1;
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::SchemaVersion { .. })
    ));
}

#[test]
fn empty_region_id_is_rejected() {
    let mut s = spec();
    s.region_id = "  ".to_string();
    assert_eq!(s.validate(), Err(CircuitSpecError::EmptyField("region_id")));
}

#[test]
fn circuit_without_members_is_rejected() {
    let mut s = spec();
    s.member_area_ids.clear();
    assert_eq!(s.validate(), Err(CircuitSpecError::NoMembers));
}

#[test]
fn boundary_needs_inputs_and_outputs() {
    let mut s = spec();
    s.boundary.input_area_ids.clear();
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidBoundary(_))
    ));
    let mut s = spec();
    s.boundary.output_area_ids.clear();
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidBoundary(_))
    ));
}

#[test]
fn boundary_areas_must_be_members() {
    let mut s = spec();
    s.boundary.output_area_ids.insert("elsewhere".to_string());
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::AreaNotAllowed { .. })
    ));
}

#[test]
fn stimulus_must_target_a_boundary_input() {
    let mut s = spec();
    s.stimulus.stimuli[0].area_id = "hid".to_string();
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::AreaNotAllowed { .. })
    ));
}

#[test]
fn trial_counts_must_be_positive() {
    let mut s = spec();
    s.stimulus.bursts_per_trial = 0;
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidStimulus(_))
    ));
    let mut s = spec();
    s.stimulus.trials_per_stimulus = 0;
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidStimulus(_))
    ));
}

#[test]
fn listed_voxels_must_be_non_empty_and_distinct() {
    let mut s = spec();
    s.stimulus.stimuli[0].voxels = VoxelSelection::Listed { voxels: vec![] };
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidStimulus(_))
    ));
    let mut s = spec();
    s.stimulus.stimuli[0].voxels = VoxelSelection::Listed {
        voxels: vec![[0, 0, 0], [0, 0, 0]],
    };
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidStimulus(_))
    ));
}

#[test]
fn same_label_cannot_drive_an_area_twice() {
    let mut s = spec();
    s.stimulus.stimuli.push(stimulus("a", "in"));
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidStimulus(_))
    ));
}

#[test]
fn selectivity_needs_two_labels() {
    let mut s = spec();
    s.stimulus.stimuli.truncate(1);
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidMetric(_))
    ));
}

#[test]
fn latency_needs_a_stimulus() {
    let mut s = spec();
    s.metric = CircuitMetric::ResponseLatency {
        output_area_id: "out".to_string(),
    };
    s.stimulus.stimuli.clear();
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidMetric(_))
    ));
}

#[test]
fn passive_metrics_allow_no_stimuli() {
    let mut s = spec();
    s.stimulus.stimuli.clear();
    s.metric = CircuitMetric::FiringRateBand {
        area_id: "hid".to_string(),
        min_hz: 1.0,
        max_hz: 5.0,
    };
    assert_eq!(s.validate(), Ok(()));
    s.metric = CircuitMetric::ActiveFraction {
        output_area_id: "out".to_string(),
        target: 0.1,
    };
    assert_eq!(s.validate(), Ok(()));
}

#[test]
fn output_metrics_must_observe_a_boundary_output() {
    let mut s = spec();
    s.metric = CircuitMetric::Selectivity {
        output_area_id: "hid".to_string(),
    };
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::AreaNotAllowed { .. })
    ));
}

#[test]
fn firing_rate_band_bounds_are_checked() {
    let mut s = spec();
    for (min_hz, max_hz) in [(5.0, 1.0), (-1.0, 1.0), (f64::NAN, 1.0)] {
        s.metric = CircuitMetric::FiringRateBand {
            area_id: "hid".to_string(),
            min_hz,
            max_hz,
        };
        assert!(matches!(
            s.validate(),
            Err(CircuitSpecError::InvalidMetric(_))
        ));
    }
}

#[test]
fn active_fraction_target_is_a_fraction() {
    let mut s = spec();
    for target in [-0.1, 1.1, f64::NAN] {
        s.metric = CircuitMetric::ActiveFraction {
            output_area_id: "out".to_string(),
            target,
        };
        assert!(matches!(
            s.validate(),
            Err(CircuitSpecError::InvalidMetric(_))
        ));
    }
}

#[test]
fn pattern_targets_must_cover_every_label() {
    let mut s = spec();
    let mut targets = BTreeMap::new();
    targets.insert("a".to_string(), vec![[0, 0, 0]]);
    s.metric = CircuitMetric::OutputPatternMatch {
        output_area_id: "out".to_string(),
        targets: targets.clone(),
    };
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidMetric(_))
    ));
    targets.insert("b".to_string(), vec![[1, 0, 0]]);
    s.metric = CircuitMetric::OutputPatternMatch {
        output_area_id: "out".to_string(),
        targets: targets.clone(),
    };
    assert_eq!(s.validate(), Ok(()));
    targets.insert("b".to_string(), vec![]);
    s.metric = CircuitMetric::OutputPatternMatch {
        output_area_id: "out".to_string(),
        targets,
    };
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidMetric(_))
    ));
}

#[test]
fn zero_constraints_are_rejected() {
    let mut s = spec();
    s.constraints.max_neuron_count = Some(0);
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidConstraint(_))
    ));
    let mut s = spec();
    s.constraints.max_synapse_count = Some(0);
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidConstraint(_))
    ));
}

#[test]
fn search_space_rejects_non_members_duplicates_and_bad_bounds() {
    let mut s = spec();
    s.search_space[0].area_id = "elsewhere".to_string();
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::AreaNotAllowed { .. })
    ));

    let mut s = spec();
    s.search_space.push(s.search_space[0].clone());
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidSearchSpace(_))
    ));

    let mut s = spec();
    s.search_space[0].min = 3.0;
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidSearchSpace(_))
    ));
}

#[test]
fn integer_parameters_need_whole_non_negative_bounds() {
    let mut s = spec();
    s.search_space = vec![ParameterRange {
        area_id: "hid".to_string(),
        parameter: CircuitParameter::RefractoryPeriod,
        min: 0.5,
        max: 3.0,
    }];
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidSearchSpace(_))
    ));
    s.search_space[0].min = -1.0;
    assert!(matches!(
        s.validate(),
        Err(CircuitSpecError::InvalidSearchSpace(_))
    ));
    s.search_space[0].min = 0.0;
    assert_eq!(s.validate(), Ok(()));
}

#[test]
fn metric_directions_are_fixed() {
    let mut s = spec();
    assert_eq!(s.objective(), FitnessObjective::Maximize);
    s.metric = CircuitMetric::ResponseLatency {
        output_area_id: "out".to_string(),
    };
    assert_eq!(s.objective(), FitnessObjective::Minimize);
}

#[test]
fn spec_round_trips_through_json() {
    let s = spec();
    let json = serde_json::to_string(&s).expect("serialize");
    assert!(json.contains("\"kind\":\"selectivity\""));
    let back: CircuitObjectiveSpec = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, s);
}

fn edge(src: &str, dst: &str) -> MappingEdge {
    MappingEdge {
        src_area_id: src.to_string(),
        dst_area_id: dst.to_string(),
    }
}

fn members(entries: &[(&str, AreaRole)]) -> BTreeMap<String, AreaRole> {
    entries
        .iter()
        .map(|(id, role)| (id.to_string(), *role))
        .collect()
}

#[test]
fn boundary_follows_mappings_that_cross_the_region() {
    let m = members(&[
        ("in", AreaRole::Internal),
        ("hid", AreaRole::Internal),
        ("out", AreaRole::Internal),
    ]);
    let edges = [
        edge("ext_src", "in"),
        edge("in", "hid"),
        edge("hid", "out"),
        edge("out", "ext_dst"),
        edge("ext_src", "ext_dst"),
        edge("hid", "hid"),
    ];
    let b = resolve_boundary(&m, &edges);
    assert_eq!(b.input_area_ids, ids(&["in"]));
    assert_eq!(b.output_area_ids, ids(&["out"]));
}

#[test]
fn sensory_and_motor_members_are_always_boundary() {
    let m = members(&[
        ("ipu", AreaRole::Sensory),
        ("hid", AreaRole::Internal),
        ("opu", AreaRole::Motor),
    ]);
    let edges = [edge("ipu", "hid"), edge("hid", "opu")];
    let b = resolve_boundary(&m, &edges);
    assert_eq!(b.input_area_ids, ids(&["ipu"]));
    assert_eq!(b.output_area_ids, ids(&["opu"]));
}

#[test]
fn an_area_can_be_both_input_and_output() {
    let m = members(&[("relay", AreaRole::Internal)]);
    let edges = [edge("ext_a", "relay"), edge("relay", "ext_b")];
    let b = resolve_boundary(&m, &edges);
    assert_eq!(b.input_area_ids, ids(&["relay"]));
    assert_eq!(b.output_area_ids, ids(&["relay"]));
}

#[test]
fn closed_circuit_has_no_boundary() {
    let m = members(&[("a", AreaRole::Internal), ("b", AreaRole::Internal)]);
    let b = resolve_boundary(&m, &[edge("a", "b"), edge("b", "a")]);
    assert!(b.input_area_ids.is_empty());
    assert!(b.output_area_ids.is_empty());
}

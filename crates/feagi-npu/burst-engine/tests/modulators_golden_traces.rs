// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of synaptic modulators (gate, gain, reward, learning rate) for a teaching
//! visual.
//!
//! Each trace drives the real `ModulatorEngine` without an NPU: `set_bindings`,
//! `allocate_group`, `commit_burst` with a `FireQueue` holding the driver firings, then reads
//! `transmission_factors`, `reward_for_group` and `learning_factor`. The transmission factor
//! scales a real spike: `SynapticPropagationEngine::propagate_delayed` with the synapse
//! stamped with the modulation group. By default (plain `cargo test`) the test regenerates the
//! traces in memory and asserts they equal
//! `tests/fixtures/modulators/golden-traces.json`. `feagi_core_commit` is ignored by the
//! comparison.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-npu/burst-engine/tests/fixtures/modulators:<feagi-react-core-pro>/src/feagi-academy/visuals/modulators/fixtures \
//! cargo test -p feagi-npu-burst-engine --test modulators_golden_traces
//! ```
//!
//! `FEAGI_WRITE_VISUAL_FIXTURES` is a `:`-separated list of directories; `golden-traces.json`
//! is written into each. The commit is a note only: the working tree may hold uncommitted
//! changes.
//!
//! Row semantics. `burst` is 0-based. `driver_fired[i]` says whether modulator i's driver
//! fires at this burst (the schedule is input data). The factors in a row are the ones in
//! force DURING this burst, i.e. produced by the commit at the end of the previous burst; the
//! commit of this burst's firings shows up in the next row (one-burst lag). Before burst 0 no
//! burst has been committed: a gate group starts closed (factor 0), other groups start at
//! their silent value. `transmission_factor` is the group's gain x gate factor (1 when the
//! group has neither). `reward` is `reward_for_group` (sum of reward signals, 0 when none).
//! `learning_factor` is `learning_factor` (product of 1 + signal, 1 when none).
//! `contribution` is the PSP that one spike of the source (weight 1, PSP `source_psp`,
//! excitatory) delivers to the target at this burst, after the transmission factor.
//!
//! NOT driven by Rust code here: the effective reward used by the weight commit is
//! `reward * learning_factor` (`npu.rs`, `apply_stdp_updates_for_burst`, the
//! `reward_for_group(..) * learning_factor(..)` product); and the engine's choice to push the
//! factors into the propagation engine after each commit (`npu.rs`, Phase 3.6) is mirrored by
//! this test's per-burst `set_transmission_factors` / `clear_transmission_factors` call.

use ahash::AHashMap;
use feagi_npu_burst_engine::fire_structures::{FireQueue, FiringNeuron, FIRE_KIND_STDP_ELIGIBLE};
use feagi_npu_burst_engine::modulator_engine::{ModulatorBinding, ModulatorEngine};
use feagi_npu_burst_engine::SynapticPropagationEngine;
use feagi_npu_neural::types::NeuronId;
use feagi_npu_runtime::{StdNeuronArray, StdSynapseArray, SynapseStorage};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::ModulatorKind;
use serde::{Deserialize, Serialize};
use std::path::Path;

const BURSTS: u64 = 16;
const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/modulators";
const SOURCE: u32 = 1;
const TARGET: u32 = 100;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Modulator {
    /// `synaptic.gate`, `synaptic.transmission_gain`, `synaptic.reward` or
    /// `synaptic.learning_rate`.
    kind: String,
    magnitude_percent: f32,
    graded: bool,
    full_scale_potential: f32,
    /// Membrane potential of the driver neuron when it fires (matters only when graded).
    driver_potential: f32,
    /// Bursts at which the driver fires.
    fire_bursts: Vec<u64>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    source_psp: f32,
    modulators: Vec<Modulator>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    burst: u64,
    driver_fired: Vec<bool>,
    transmission_factor: f32,
    reward: f32,
    learning_factor: f32,
    contribution: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Trace {
    name: String,
    settings: Settings,
    rows: Vec<Row>,
}

#[derive(Serialize, Deserialize)]
struct Golden {
    source_crates: Vec<String>,
    functions: Vec<String>,
    feagi_core_commit: String,
    traces: Vec<Trace>,
}

fn kind_of(name: &str) -> ModulatorKind {
    match name {
        "synaptic.gate" => ModulatorKind::Gate,
        "synaptic.transmission_gain" => ModulatorKind::TransmissionGain,
        "synaptic.reward" => ModulatorKind::Reward,
        "synaptic.learning_rate" => ModulatorKind::LearningRate,
        other => panic!("unsupported modulator kind {other}"),
    }
}

fn area(last: u8) -> CorticalID {
    let mut bytes = *b"chom0001";
    bytes[7] = last;
    CorticalID::try_from_bytes(&bytes).unwrap()
}

fn run_trace(name: &str, s: Settings) -> Trace {
    // Driver i lives in cortical area index i + 1.
    let bindings: Vec<ModulatorBinding> = s
        .modulators
        .iter()
        .enumerate()
        .map(|(i, m)| ModulatorBinding {
            instance_id: format!("mod{i}"),
            kind: kind_of(&m.kind),
            magnitude_percent: m.magnitude_percent,
            graded: m.graded,
            full_scale_potential: m.full_scale_potential,
            driver_cortical_idx: i as u32 + 1,
        })
        .collect();
    let ids: Vec<String> = bindings.iter().map(|b| b.instance_id.clone()).collect();
    let mut engine = ModulatorEngine::default();
    engine.set_bindings(bindings);
    let group = engine.allocate_group(&ids);
    assert!(group > 0, "modulators must form a synaptic group");

    // One synapse SOURCE -> TARGET stamped with the group.
    let mut synapses = StdSynapseArray::new(4);
    synapses
        .add_synapse(SOURCE, TARGET, 1.0, s.source_psp, 0, 0, 1)
        .unwrap();
    synapses.modulation_groups = vec![group];
    synapses.modulation_groups_allocated = true;
    let mut propagation = SynapticPropagationEngine::new();
    let mut mapping = AHashMap::new();
    mapping.insert(NeuronId(SOURCE), area(1));
    mapping.insert(NeuronId(TARGET), area(2));
    propagation.set_neuron_mapping(mapping);
    propagation.build_synapse_index(&synapses);

    let mut storage = StdNeuronArray::<f32>::new(4);
    let mps = AHashMap::new();
    let mut rows = Vec::new();
    for burst in 0..BURSTS {
        // Factors in force during this burst (produced by the previous commit).
        let factors = engine.transmission_factors();
        if engine.transmission_active() {
            propagation.set_transmission_factors(factors.clone());
        } else {
            propagation.clear_transmission_factors();
        }
        let delayed = propagation
            .propagate_delayed(&[NeuronId(SOURCE)], &synapses, &mps)
            .unwrap();
        let contribution: f32 = delayed
            .fcl_by_delay
            .values()
            .flat_map(|by_area| by_area.values())
            .flatten()
            .filter(|(nid, _)| *nid == NeuronId(TARGET))
            .map(|(_, c)| c.0)
            .sum();
        let reward = engine.reward_for_group(group);
        let learning_factor = engine.learning_factor(group);
        let transmission_factor = factors.get(group as usize).copied().unwrap_or(1.0);

        // This burst's driver firings are committed at its end.
        let mut fq = FireQueue::new();
        let mut driver_fired = Vec::new();
        for (i, m) in s.modulators.iter().enumerate() {
            let fires = m.fire_bursts.contains(&burst);
            driver_fired.push(fires);
            if fires {
                fq.add_neuron(FiringNeuron {
                    neuron_id: NeuronId(1000 + i as u32),
                    membrane_potential: m.driver_potential,
                    cortical_idx: i as u32 + 1,
                    x: 0,
                    y: 0,
                    z: 0,
                    fire_kind: FIRE_KIND_STDP_ELIGIBLE,
                });
            }
        }
        rows.push(Row {
            burst,
            driver_fired,
            transmission_factor,
            reward,
            learning_factor,
            contribution,
        });
        engine.commit_burst(&mut storage, &fq);
    }
    Trace {
        name: name.to_string(),
        settings: s,
        rows,
    }
}

fn modulator(kind: &str, magnitude_percent: f32, fire_bursts: Vec<u64>) -> Modulator {
    Modulator {
        kind: kind.to_string(),
        magnitude_percent,
        graded: false,
        full_scale_potential: 1.0,
        driver_potential: 1.0,
        fire_bursts,
    }
}

fn generate_traces() -> Vec<Trace> {
    let always: Vec<u64> = (0..BURSTS).collect();
    vec![
        // Gate +100: closed until the driver fires, then fully open one burst later.
        run_trace(
            "gate_open_for_three_bursts",
            Settings {
                source_psp: 0.5,
                modulators: vec![modulator("synaptic.gate", 100.0, vec![5, 6, 7])],
            },
        ),
        // Gain +50 always on, gate +100 open only at bursts 4..=8: a closed gate zeroes the gain.
        run_trace(
            "gain_times_gate",
            Settings {
                source_psp: 0.5,
                modulators: vec![
                    modulator("synaptic.transmission_gain", 50.0, always.clone()),
                    modulator("synaptic.gate", 100.0, vec![4, 5, 6, 7, 8]),
                ],
            },
        ),
        // Graded gate, +100 at full scale: the open amount follows the driver's potential.
        run_trace(
            "graded_gate_follows_driver_potential",
            Settings {
                source_psp: 0.5,
                modulators: vec![Modulator {
                    kind: "synaptic.gate".to_string(),
                    magnitude_percent: 100.0,
                    graded: true,
                    full_scale_potential: 1.0,
                    driver_potential: 0.25,
                    fire_bursts: vec![3, 4, 5],
                }],
            },
        ),
        // Reward +100 and punishment -100 sum; learning rate +50 multiplies the commit.
        run_trace(
            "reward_punishment_learning_rate",
            Settings {
                source_psp: 0.5,
                modulators: vec![
                    modulator("synaptic.reward", 100.0, vec![3, 9]),
                    modulator("synaptic.reward", -100.0, vec![9, 12]),
                    modulator("synaptic.learning_rate", 50.0, vec![9, 12]),
                ],
            },
        ),
    ]
}

fn golden(commit: &str, traces: Vec<Trace>) -> Golden {
    Golden {
        source_crates: vec![
            "feagi-npu-burst-engine".to_string(),
            "feagi-structures".to_string(),
        ],
        functions: vec![
            "ModulatorEngine::commit_burst".to_string(),
            "ModulatorEngine::transmission_factors".to_string(),
            "ModulatorEngine::reward_for_group".to_string(),
            "ModulatorEngine::learning_factor".to_string(),
            "SynapticPropagationEngine::propagate_delayed".to_string(),
        ],
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn modulators_golden_traces_match_checked_in_json() {
    let traces = generate_traces();

    if let Ok(dirs) = std::env::var("FEAGI_WRITE_VISUAL_FIXTURES") {
        let commit = std::env::var("FEAGI_CORE_COMMIT").unwrap_or_else(|_| "unknown".to_string());
        let text = serde_json::to_string_pretty(&golden(&commit, traces.clone())).unwrap() + "\n";
        for dir in dirs.split(':').filter(|d| !d.is_empty()) {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(Path::new(dir).join(FILE_NAME), &text).unwrap();
        }
        return;
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIR)
        .join(FILE_NAME);
    let text = std::fs::read_to_string(&path).expect("fixture missing; see module docs");
    let checked_in: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(
        checked_in.traces, traces,
        "modulators drifted from the golden traces"
    );

    // Sanity: the traces show what they claim.
    let row = |t: usize, b: usize| traces[t].rows[b].clone();
    // Gate: closed before the first commit, open one burst after the driver fires.
    assert_eq!(row(0, 0).contribution, 0.0);
    assert_eq!(row(0, 5).contribution, 0.0);
    assert!((row(0, 6).contribution - 0.5).abs() < 1e-6);
    assert_eq!(row(0, 9).contribution, 0.0);
    // Gain x gate: open gate gives 1.5x, closed gate zeroes it.
    assert!((row(1, 6).contribution - 0.75).abs() < 1e-6);
    assert_eq!(row(1, 12).contribution, 0.0);
    // Graded gate: 0.25 of full scale.
    assert!((row(2, 4).contribution - 0.125).abs() < 1e-6);
    // Reward lands one burst after the driver fires; reward and punishment cancel at burst 9.
    assert!((row(3, 4).reward - 1.0).abs() < 1e-6);
    assert!(row(3, 10).reward.abs() < 1e-6);
    assert!((row(3, 13).reward + 1.0).abs() < 1e-6);
    assert!((row(3, 13).learning_factor - 1.5).abs() < 1e-6);
}

#[test]
fn drift_is_detected_negative_case() {
    let traces = generate_traces();
    let mut altered = traces.clone();
    altered[0].rows[6].contribution += 1e-4;
    assert_ne!(altered, traces);
    // A different magnitude must change the trace.
    let other = run_trace(
        "x",
        Settings {
            source_psp: 0.5,
            modulators: vec![modulator("synaptic.gate", 50.0, vec![5, 6, 7])],
        },
    );
    assert_ne!(other.rows, traces[0].rows);
}

#[test]
fn json_round_trips_f32_exactly() {
    let traces = generate_traces();
    let text = serde_json::to_string(&golden("x", traces.clone())).unwrap();
    let back: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(back.traces, traces);
}

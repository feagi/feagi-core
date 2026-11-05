// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of a spike crossing a mapping, for a teaching visual.
//!
//! Each trace drives the real propagation path without an NPU: a `StdSynapseArray` holding the
//! synapses, `SynapticPropagationEngine::propagate_delayed` (PSP source, weight x PSP x sign,
//! fan-out division, delay), `SynapticArrivalSchedule` (arrival burst = spike burst + delay) and
//! `update_neuron_lif` for the target neuron. By default (plain `cargo test`) the test
//! regenerates the traces in memory and asserts they equal
//! `tests/fixtures/spike-across-mapping/golden-traces.json`. `feagi_core_commit` is ignored by
//! the comparison.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-npu/burst-engine/tests/fixtures/spike-across-mapping:<feagi-react-core-pro>/src/feagi-academy/visuals/spike-across-mapping/fixtures \
//! cargo test -p feagi-npu-burst-engine --test spike_across_mapping_golden_traces
//! ```
//!
//! `FEAGI_WRITE_VISUAL_FIXTURES` is a `:`-separated list of directories; `golden-traces.json`
//! is written into each. The commit is a note only: the working tree may hold uncommitted
//! changes.
//!
//! NOT driven by Rust code here: the conversion of a mapping rule's
//! `postSynapticCurrent_multiplier` m into a synapse `(weight, type)`. That is
//! `ConnectomeManager::resolve_synapse_params_for_rule` (feagi-brain-development,
//! `connectome_manager.rs`, private, needs a ConnectomeManager): weight = |m|, type =
//! Inhibitory when m < 0, else Excitatory. This test mirrors that rule as input data
//! (`weight`, `synapse_type` in each source). The PSP stored on the synapse is the source
//! area's `postsynaptic_current` (`source_psp`), also given as input.
//!
//! Row semantics. `burst` is 0-based. Per burst, in this order: (1) the contributions that
//! arrive at this burst (scheduled earlier) are summed per target; `input` is the sum for the
//! tracked target (0 when none). (2) The tracked target is stepped with `update_neuron_lif`
//! only when `input != 0` (lazy leak, as in the neuron-firing visual); with no input its
//! `membrane_potential` is `decayed_charge(stored V, leak, bursts since last update)` and its
//! stored state is untouched. (3) Each source that fires at this burst (`source_fired[i]`) is
//! propagated; its contributions are scheduled for `burst + delay_bursts`. `membrane_potential`
//! is V of the target after step 2 (0 when it fired and reset); `fired` is the return of
//! `update_neuron_lif`. `arrivals[i]` is the contribution of source i that arrived at this
//! burst for the tracked target (0 when none).

use ahash::AHashMap;
use feagi_npu_burst_engine::{SynapticArrivalSchedule, SynapticPropagationEngine};
use feagi_npu_neural::types::{FireCandidateList, NeuronId};
use feagi_npu_neural::{decayed_charge, update_neuron_lif};
use feagi_npu_runtime::{StdSynapseArray, SynapseStorage};
use feagi_structures::genomic::cortical_area::CorticalID;
use serde::{Deserialize, Serialize};
use std::path::Path;

const BURSTS: u64 = 24;
const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/spike-across-mapping";
const TARGET: u32 = 100;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Source {
    /// Synapse weight on the tracked target (= |PSC multiplier|).
    weight: f32,
    /// 0 excitatory, 1 inhibitory (= sign of the PSC multiplier).
    synapse_type: u8,
    /// Source area `postsynaptic_current`, stored on the synapse.
    source_psp: f32,
    /// Source area `mp_driven_psp`: the spike delivers the source's membrane potential.
    mp_driven: bool,
    /// Membrane potential of the source when it fires (used only when `mp_driven`).
    source_mp: f32,
    /// Source area `psp_uniform_distribution`.
    uniform_psp: bool,
    /// Total outgoing synapses of the source (1 = only the tracked target).
    outgoing_synapses: u32,
    delay_bursts: u8,
    /// The source fires at bursts `first_spike`, `first_spike + period`, ...
    first_spike: u64,
    period: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    sources: Vec<Source>,
    target_threshold: f32,
    target_leak: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    burst: u64,
    source_fired: Vec<bool>,
    arrivals: Vec<f32>,
    input: f32,
    membrane_potential: f32,
    fired: bool,
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

fn area_id(i: usize) -> CorticalID {
    let mut bytes = *b"chom0001";
    bytes[7] = b'1' + i as u8;
    CorticalID::try_from_bytes(&bytes).unwrap()
}

fn dst_area_id() -> CorticalID {
    CorticalID::try_from_bytes(b"chom0009").unwrap()
}

fn run_trace(name: &str, s: Settings) -> Trace {
    let n = s.sources.len();
    let mut synapses = StdSynapseArray::new(64);
    let mut engine = SynapticPropagationEngine::new();
    let mut mapping = AHashMap::new();
    mapping.insert(NeuronId(TARGET), dst_area_id());
    let mut mps = AHashMap::new();
    // Source i is neuron i + 1. Its first synapse goes to the tracked target; the others go
    // to sibling targets that this trace does not track (they only matter for fan-out).
    for (i, src) in s.sources.iter().enumerate() {
        let sid = i as u32 + 1;
        mapping.insert(NeuronId(sid), area_id(i));
        mps.insert(NeuronId(sid), src.source_mp);
        engine.set_mp_driven_psp_flag(area_id(i), src.mp_driven);
        engine.set_psp_uniform_distribution_flag(area_id(i), src.uniform_psp);
        synapses
            .add_synapse(
                sid,
                TARGET,
                src.weight,
                src.source_psp,
                src.synapse_type,
                0,
                src.delay_bursts,
            )
            .unwrap();
        for extra in 1..src.outgoing_synapses {
            let sibling = TARGET + 1 + (i as u32) * 100 + extra;
            mapping.insert(NeuronId(sibling), dst_area_id());
            synapses
                .add_synapse(
                    sid,
                    sibling,
                    src.weight,
                    src.source_psp,
                    src.synapse_type,
                    0,
                    src.delay_bursts,
                )
                .unwrap();
        }
    }
    engine.set_neuron_mapping(mapping);
    engine.build_synapse_index(&synapses);

    let mut schedule = SynapticArrivalSchedule::default();
    let mut potential = 0.0f32;
    let mut fraction = 0.0f32;
    let mut last_update = 0u64;
    let mut rows = Vec::new();
    for burst in 0..BURSTS {
        // (1) arrivals scheduled for this burst (summed over sources by the schedule).
        let mut fcl = FireCandidateList::new();
        let mut assoc = AHashMap::new();
        schedule.drain_into_phase1(burst, &mut fcl, &mut assoc);
        let input = fcl.get(NeuronId(TARGET)).unwrap_or(0.0);

        // (2) target neuron step.
        let (v, fired) = if input != 0.0 {
            let fired = update_neuron_lif(
                &mut potential,
                &mut fraction,
                s.target_threshold,
                0.0,
                s.target_leak,
                0.0,
                input,
                burst - last_update,
            );
            last_update = burst;
            (potential + fraction, fired)
        } else {
            (
                decayed_charge(potential + fraction, s.target_leak, burst - last_update),
                false,
            )
        };

        // (3) sources firing at this burst propagate; arrivals are scheduled.
        let mut source_fired = vec![false; n];
        for (i, src) in s.sources.iter().enumerate() {
            if burst >= src.first_spike && (burst - src.first_spike) % src.period == 0 {
                source_fired[i] = true;
                let delayed = engine
                    .propagate_delayed(&[NeuronId(i as u32 + 1)], &synapses, &mps)
                    .unwrap();
                schedule.schedule_from_delayed_propagation(
                    burst,
                    &delayed.fcl_by_delay,
                    &delayed.memory_associative_by_delay,
                );
            }
        }
        rows.push((burst, source_fired, input, v, fired));
    }

    // Per-source arrivals: the schedule merges sources, so run each source alone through the
    // same propagation path to record what it delivers to the tracked target, then place it at
    // spike burst + delay.
    let mut arrivals_by_source: Vec<AHashMap<u64, f32>> = Vec::new();
    for (i, src) in s.sources.iter().enumerate() {
        let mut per = AHashMap::new();
        let delayed = engine
            .propagate_delayed(&[NeuronId(i as u32 + 1)], &synapses, &mps)
            .unwrap();
        let mut contribution = 0.0f32;
        for (delay, by_area) in &delayed.fcl_by_delay {
            assert_eq!(*delay as u8, src.delay_bursts);
            for targets in by_area.values() {
                for &(nid, c) in targets {
                    if nid == NeuronId(TARGET) {
                        contribution += c.0;
                    }
                }
            }
        }
        let mut b = src.first_spike;
        while b < BURSTS {
            per.insert(b + u64::from(src.delay_bursts), contribution);
            b += src.period;
        }
        arrivals_by_source.push(per);
    }

    let rows = rows
        .into_iter()
        .map(|(burst, source_fired, input, v, fired)| Row {
            burst,
            source_fired,
            arrivals: arrivals_by_source
                .iter()
                .map(|m| m.get(&burst).copied().unwrap_or(0.0))
                .collect(),
            input,
            membrane_potential: v,
            fired,
        })
        .collect();
    Trace {
        name: name.to_string(),
        settings: s,
        rows,
    }
}

#[allow(clippy::too_many_arguments)]
fn source(
    psc_multiplier: f32,
    source_psp: f32,
    mp_driven: bool,
    source_mp: f32,
    uniform_psp: bool,
    outgoing_synapses: u32,
    delay_bursts: u8,
    first_spike: u64,
    period: u64,
) -> Source {
    // Mirrors ConnectomeManager::resolve_synapse_params_for_rule (see module docs).
    let (weight, synapse_type) = if psc_multiplier < 0.0 {
        (psc_multiplier.abs(), 1u8)
    } else {
        (psc_multiplier, 0u8)
    };
    Source {
        weight,
        synapse_type,
        source_psp,
        mp_driven,
        source_mp,
        uniform_psp,
        outgoing_synapses,
        delay_bursts,
        first_spike,
        period,
    }
}

fn settings(sources: Vec<Source>, target_threshold: f32, target_leak: f32) -> Settings {
    Settings {
        sources,
        target_threshold,
        target_leak,
    }
}

fn generate_traces() -> Vec<Trace> {
    vec![
        // Fixed PSP 1.0, PSC multiplier 0.4: each spike adds 0.4 one burst later.
        run_trace(
            "excitatory_psc_0_4",
            settings(
                vec![source(0.4, 1.0, false, 0.0, true, 1, 1, 0, 2)],
                1.0,
                0.1,
            ),
        ),
        // Negative multiplier: weight |m|, inhibitory type, so the spike removes charge.
        run_trace(
            "excitatory_then_inhibitory_veto",
            settings(
                vec![
                    source(0.6, 1.0, false, 0.0, true, 1, 1, 0, 2),
                    source(-0.8, 1.0, false, 0.0, true, 1, 2, 4, 4),
                ],
                1.0,
                0.1,
            ),
        ),
        // Fan-out of 4 with uniform PSP off: the contribution is divided by 4.
        run_trace(
            "fan_out_4_divided",
            settings(
                vec![source(1.0, 1.0, false, 0.0, false, 4, 1, 0, 2)],
                1.0,
                0.0,
            ),
        ),
        // Same fan-out with uniform PSP on: every synapse gets the full amount.
        run_trace(
            "fan_out_4_uniform",
            settings(
                vec![source(1.0, 1.0, false, 0.0, true, 4, 1, 0, 2)],
                1.0,
                0.0,
            ),
        ),
        // mp_driven PSP: the spike carries the source's own potential (0.7), not the fixed PSP.
        run_trace(
            "mp_driven_carries_source_potential",
            settings(
                vec![source(1.0, 0.1, true, 0.7, true, 1, 1, 0, 3)],
                1.0,
                0.0,
            ),
        ),
        // Synaptic delay 3: the effect lands 3 bursts after the spike.
        run_trace(
            "delay_3_bursts",
            settings(
                vec![source(0.6, 1.0, false, 0.0, true, 1, 3, 0, 3)],
                1.0,
                0.1,
            ),
        ),
        // Spike charge visual, step 1: multiplier 0.4, PSP 1, uniform on, 4 outgoing synapses.
        run_trace(
            "spike_charge_uniform_fan_out_4_excitatory_0_4",
            settings(
                vec![source(0.4, 1.0, false, 0.0, true, 4, 1, 0, 2)],
                1.0,
                0.1,
            ),
        ),
        // Spike charge visual, step 2: same, uniform off, so 0.4 is divided by 4.
        run_trace(
            "spike_charge_fan_out_4_divided_0_4",
            settings(
                vec![source(0.4, 1.0, false, 0.0, false, 4, 1, 0, 2)],
                1.0,
                0.1,
            ),
        ),
        // Spike charge visual, step 3: multiplier -0.4, uniform on, 4 outgoing synapses.
        run_trace(
            "spike_charge_uniform_fan_out_4_inhibitory_0_4",
            settings(
                vec![source(-0.4, 1.0, false, 0.0, true, 4, 1, 0, 2)],
                1.0,
                0.1,
            ),
        ),
    ]
}

fn golden(commit: &str, traces: Vec<Trace>) -> Golden {
    Golden {
        source_crates: vec![
            "feagi-npu-burst-engine".to_string(),
            "feagi-npu-neural".to_string(),
            "feagi-npu-runtime".to_string(),
        ],
        functions: vec![
            "SynapticPropagationEngine::propagate_delayed".to_string(),
            "SynapticArrivalSchedule::schedule_from_delayed_propagation".to_string(),
            "SynapticArrivalSchedule::drain_into_phase1".to_string(),
            "feagi_npu_neural::update_neuron_lif".to_string(),
        ],
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn spike_across_mapping_golden_traces_match_checked_in_json() {
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
        "spike propagation drifted from the golden traces"
    );

    // Sanity: the traces show what they claim.
    let first_input = |t: &Trace| t.rows.iter().find(|r| r.input != 0.0).unwrap().clone();
    assert!((first_input(&traces[0]).input - 0.4).abs() < 1e-6);
    assert_eq!(
        first_input(&traces[0]).burst,
        1,
        "delay 1 lands one burst later"
    );
    assert!(
        traces[1].rows.iter().any(|r| r.input < 0.0),
        "inhibitory source must deliver negative input"
    );
    assert!((first_input(&traces[2]).input - 0.25).abs() < 1e-6);
    assert!((first_input(&traces[3]).input - 1.0).abs() < 1e-6);
    assert!((first_input(&traces[4]).input - 0.7).abs() < 1e-6);
    assert_eq!(first_input(&traces[5]).burst, 3);
    assert!((first_input(&traces[6]).input - 0.4).abs() < 1e-6);
    assert!((first_input(&traces[7]).input - 0.1).abs() < 1e-6);
    assert!((first_input(&traces[8]).input + 0.4).abs() < 1e-6);
}

#[test]
fn drift_is_detected_negative_case() {
    let traces = generate_traces();
    let mut altered = traces.clone();
    altered[1].rows[10].membrane_potential += 1e-4;
    assert_ne!(altered, traces);
    // Dropping the fan-out division must change the trace.
    let uniform = run_trace(
        "x",
        settings(
            vec![source(1.0, 1.0, false, 0.0, true, 4, 1, 0, 2)],
            1.0,
            0.0,
        ),
    );
    assert_ne!(uniform.rows, traces[2].rows);
}

#[test]
fn json_round_trips_f32_exactly() {
    let traces = generate_traces();
    let text = serde_json::to_string(&golden("x", traces.clone())).unwrap();
    let back: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(back.traces, traces);
}

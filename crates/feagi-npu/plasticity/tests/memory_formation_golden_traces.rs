// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of how a memory neuron forms and ages, for a teaching visual.
//!
//! Each trace drives the real `MemoryNeuronArray` (create, reactivate, age, long-term
//! conversion, reward promotion) in the order the plasticity service runs them each burst, and
//! the real `ModulatorEngine` for `memory.formation` suppression. No NPU is involved. By
//! default (plain `cargo test`) the test regenerates the traces in memory and asserts they
//! equal `tests/fixtures/memory-formation/golden-traces.json`. `feagi_core_commit` is ignored
//! by the comparison.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-npu/plasticity/tests/fixtures/memory-formation:<feagi-react-core-pro>/src/feagi-academy/visuals/memory-formation/fixtures \
//! cargo test -p feagi-npu-plasticity --test memory_formation_golden_traces
//! ```
//!
//! `FEAGI_WRITE_VISUAL_FIXTURES` is a `:`-separated list of directories; `golden-traces.json`
//! is written into each. The commit is a note only: the working tree may hold uncommitted
//! changes.
//!
//! Per-burst order (mirrors `PlasticityService` in `service.rs`, burst handler): (1)
//! `check_longterm_conversion_by_area` BEFORE aging, then reward promotion
//! (`promote_to_longterm_by_id`); (2) for each pattern seen this burst: if the area holds an
//! active neuron for it, `reactivate_memory_neuron`; else if `memory.formation` currently
//! suppresses the area, skip; else `create_memory_neuron`; (3) `age_memory_neurons`; (4) the
//! modulator engine commits this burst's driver firings (the suppression takes effect from the
//! next burst).
//!
//! NOT driven by Rust code here: which patterns a memory area sees each burst (the pattern
//! detector hashes the upstream firing; the test takes the pattern ids as input data), and
//! which neurons a reward promotes (the burst engine marks them when a reward-gated
//! associative synapse is potentiated; the test takes `(burst, pattern)` as input data).
//!
//! Row semantics. `burst` is 0-based. `presented` lists the patterns seen at this burst.
//! `events` lists what happened to each, in the order it ran, as `(pattern, kind)` where kind
//! is `promoted_reward`, `promoted_long_term`, `reactivated`, `created`, `suppressed` or
//! `expired`. `neurons` is the state at the end of the burst for every pattern that has a
//! live neuron: `lifespan` is `lifespan_current` (bursts left before expiry; frozen once
//! long-term), `long_term` the flag, `activations` the activation count. A pattern with no
//! live neuron is absent. `suppressing` is whether `memory.formation` was blocking creation
//! during this burst.

use ahash::AHashMap;
use feagi_npu_burst_engine::fire_structures::{FireQueue, FiringNeuron, FIRE_KIND_STDP_ELIGIBLE};
use feagi_npu_burst_engine::modulator_engine::{ModulatorBinding, ModulatorEngine};
use feagi_npu_neural::types::NeuronId;
use feagi_npu_plasticity::{MemoryNeuronArray, MemoryNeuronLifecycleConfig};
use feagi_npu_runtime::StdNeuronArray;
use feagi_structures::genomic::ModulatorKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const BURSTS: u64 = 48;
const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/memory-formation";
const AREA: u32 = 5;
const DRIVER_AREA: u32 = 6;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    initial_lifespan: u32,
    lifespan_growth_rate: f32,
    longterm_threshold: u32,
    /// `(burst, pattern)` pairs: pattern seen at that burst.
    presentations: Vec<(u64, u64)>,
    /// `(burst, pattern)` pairs: reward promotes that pattern's neuron at that burst.
    reward_promotions: Vec<(u64, u64)>,
    /// Bursts at which a `memory.formation` -100% driver fires (0 = no such modulator).
    formation_driver_bursts: Vec<u64>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Event {
    pattern: u64,
    kind: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct NeuronState {
    pattern: u64,
    lifespan: u32,
    long_term: bool,
    activations: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    burst: u64,
    presented: Vec<u64>,
    suppressing: bool,
    events: Vec<Event>,
    neurons: Vec<NeuronState>,
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

fn ev(pattern: u64, kind: &str) -> Event {
    Event {
        pattern,
        kind: kind.to_string(),
    }
}

fn run_trace(name: &str, s: Settings) -> Trace {
    let config = MemoryNeuronLifecycleConfig {
        initial_lifespan: s.initial_lifespan,
        lifespan_growth_rate: s.lifespan_growth_rate,
        longterm_threshold: s.longterm_threshold,
        ..Default::default()
    };
    let mut configs: HashMap<u32, MemoryNeuronLifecycleConfig> = HashMap::new();
    configs.insert(AREA, config);
    let mut array = MemoryNeuronArray::new(64);

    let mut modulators = ModulatorEngine::default();
    let mut storage = StdNeuronArray::<f32>::new(4);
    if !s.formation_driver_bursts.is_empty() {
        modulators.set_bindings(vec![ModulatorBinding {
            instance_id: "no_new_memory".to_string(),
            kind: ModulatorKind::MemoryFormation,
            magnitude_percent: -100.0,
            graded: false,
            full_scale_potential: 1.0,
            driver_cortical_idx: DRIVER_AREA,
        }]);
        let mut subs = AHashMap::new();
        subs.insert(AREA, vec!["no_new_memory".to_string()]);
        modulators.set_area_subscriptions(subs);
    }

    let mut patterns: Vec<u64> = s.presentations.iter().map(|&(_, p)| p).collect();
    patterns.sort_unstable();
    patterns.dedup();

    let mut rows = Vec::new();
    for burst in 0..BURSTS {
        let mut events = Vec::new();
        let suppressing = modulators.memory_formation_suppressed(AREA);

        // (1) long-term conversion before aging, then reward promotion.
        let before_lt: Vec<(u64, bool)> = patterns
            .iter()
            .filter_map(|&p| {
                let idx = array.find_neuron_by_pattern(AREA, &p)?;
                let id = array.get_neuron_id(idx)?;
                Some((p, array.get_memory_neuron_detail(id)?.is_longterm_memory))
            })
            .collect();
        array.check_longterm_conversion_by_area(&configs, s.longterm_threshold);
        for &(p, was_lt) in &before_lt {
            let idx = array.find_neuron_by_pattern(AREA, &p).unwrap();
            let id = array.get_neuron_id(idx).unwrap();
            if !was_lt
                && array
                    .get_memory_neuron_detail(id)
                    .unwrap()
                    .is_longterm_memory
            {
                events.push(ev(p, "promoted_long_term"));
            }
        }
        for &(b, p) in &s.reward_promotions {
            if b == burst {
                if let Some(idx) = array.find_neuron_by_pattern(AREA, &p) {
                    let id = array.get_neuron_id(idx).unwrap();
                    if array.promote_to_longterm_by_id(id).is_some() {
                        events.push(ev(p, "promoted_reward"));
                    }
                }
            }
        }

        // (2) patterns seen this burst.
        let presented: Vec<u64> = s
            .presentations
            .iter()
            .filter(|&&(b, _)| b == burst)
            .map(|&(_, p)| p)
            .collect();
        for &p in &presented {
            if let Some(idx) = array.find_neuron_by_pattern(AREA, &p) {
                assert!(array.reactivate_memory_neuron(idx, burst));
                events.push(ev(p, "reactivated"));
            } else if suppressing {
                events.push(ev(p, "suppressed"));
            } else {
                let idx = array.create_memory_neuron(p, AREA, burst, &config);
                assert!(idx.is_some(), "pool has room in this trace");
                events.push(ev(p, "created"));
            }
        }

        // (3) age.
        let alive_before: Vec<(u64, usize)> = patterns
            .iter()
            .filter_map(|&p| Some((p, array.find_neuron_by_pattern(AREA, &p)?)))
            .collect();
        let died = array.age_memory_neurons(burst);
        for (p, idx) in alive_before {
            if died.contains(&idx) {
                events.push(ev(p, "expired"));
            }
        }
        array.drain_removed_neurons();

        // End-of-burst state.
        let neurons: Vec<NeuronState> = patterns
            .iter()
            .filter_map(|&p| {
                let idx = array.find_neuron_by_pattern(AREA, &p)?;
                let d = array.get_memory_neuron_detail(array.get_neuron_id(idx)?)?;
                Some(NeuronState {
                    pattern: p,
                    lifespan: d.lifespan_current,
                    long_term: d.is_longterm_memory,
                    activations: d.activation_count,
                })
            })
            .collect();

        // (4) commit this burst's modulator driver firings.
        let mut fq = FireQueue::new();
        if s.formation_driver_bursts.contains(&burst) {
            fq.add_neuron(FiringNeuron {
                neuron_id: NeuronId(1000),
                membrane_potential: 1.0,
                cortical_idx: DRIVER_AREA,
                x: 0,
                y: 0,
                z: 0,
                fire_kind: FIRE_KIND_STDP_ELIGIBLE,
            });
        }
        modulators.commit_burst(&mut storage, &fq);

        rows.push(Row {
            burst,
            presented,
            suppressing,
            events,
            neurons,
        });
    }
    Trace {
        name: name.to_string(),
        settings: s,
        rows,
    }
}

fn settings(
    initial_lifespan: u32,
    growth: f32,
    longterm_threshold: u32,
    presentations: Vec<(u64, u64)>,
) -> Settings {
    Settings {
        initial_lifespan,
        lifespan_growth_rate: growth,
        longterm_threshold,
        presentations,
        reward_promotions: Vec::new(),
        formation_driver_bursts: Vec::new(),
    }
}

fn generate_traces() -> Vec<Trace> {
    // Pattern 1 held every burst for 20 bursts.
    let held: Vec<(u64, u64)> = (0..20).map(|b| (b, 1)).collect();
    // Pattern 1 returns every 6 bursts, or every 2.
    let gap6: Vec<(u64, u64)> = (0..8).map(|k| (k * 6, 1)).collect();
    let gap2: Vec<(u64, u64)> = (0..24).map(|k| (k * 2, 1)).collect();
    let mut rewarded = settings(9, 1.0, 100, vec![(0, 1), (0, 2)]);
    rewarded.reward_promotions = vec![(3, 1)];
    let mut suppressed = settings(9, 1.0, 100, vec![(2, 1), (6, 1), (6, 2), (14, 2)]);
    suppressed.formation_driver_bursts = (4..=8).collect();
    vec![
        // Seen once: lives `initial_lifespan` bursts unseen, then expires.
        run_trace("seen_once_expires", settings(9, 1.0, 100, vec![(0, 1)])),
        // Returns too late: each return adds only 1, the gap costs 5, so it expires and a later
        // sighting creates a brand-new neuron.
        run_trace(
            "returns_every_6_bursts_too_slow",
            settings(9, 1.0, 100, gap6),
        ),
        // Held every burst: lifespan grows by 3 per burst until it reaches the long-term threshold.
        run_trace("held_becomes_long_term", settings(9, 3.0, 30, held)),
        // Reward at burst 3 makes pattern 1 long-term at once; unrewarded pattern 2 expires.
        run_trace("reward_makes_it_long_term", rewarded),
        // memory.formation -100% blocks NEW neurons (pattern 2 at burst 6) but not a known one.
        run_trace("formation_blocked_then_allowed", suppressed),
        // Returns every 2 bursts: each gap costs 1 and each return adds 1, so it stays at 9.
        run_trace(
            "returns_every_2_bursts_stays_alive",
            settings(9, 1.0, 100, gap2),
        ),
    ]
}

fn golden(commit: &str, traces: Vec<Trace>) -> Golden {
    Golden {
        source_crates: vec![
            "feagi-npu-plasticity".to_string(),
            "feagi-npu-burst-engine".to_string(),
        ],
        functions: vec![
            "MemoryNeuronArray::create_memory_neuron".to_string(),
            "MemoryNeuronArray::reactivate_memory_neuron".to_string(),
            "MemoryNeuronArray::age_memory_neurons".to_string(),
            "MemoryNeuronArray::check_longterm_conversion_by_area".to_string(),
            "MemoryNeuronArray::promote_to_longterm_by_id".to_string(),
            "ModulatorEngine::memory_formation_suppressed".to_string(),
        ],
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn memory_formation_golden_traces_match_checked_in_json() {
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
        "memory formation drifted from the golden traces"
    );

    // Sanity: the traces show what they claim.
    let kinds = |t: usize, k: &str| -> Vec<(u64, u64)> {
        traces[t]
            .rows
            .iter()
            .flat_map(|r| {
                r.events
                    .iter()
                    .filter(|e| e.kind == k)
                    .map(move |e| (r.burst, e.pattern))
            })
            .collect()
    };
    assert_eq!(kinds(0, "created"), vec![(0, 1)]);
    assert_eq!(kinds(0, "expired").len(), 1, "a pattern seen once expires");
    assert!(!kinds(1, "expired").is_empty(), "a too-slow return expires");
    assert!(kinds(5, "expired").is_empty(), "a fast return survives");
    assert_eq!(kinds(5, "created"), vec![(0, 1)]);
    assert!(!kinds(2, "promoted_long_term").is_empty());
    assert_eq!(kinds(3, "promoted_reward"), vec![(3, 1)]);
    assert_eq!(kinds(3, "expired").len(), 1, "unrewarded pattern 2 expires");
    assert_eq!(kinds(4, "suppressed"), vec![(6, 2)]);
    assert!(kinds(4, "created").contains(&(14, 2)));
}

#[test]
fn drift_is_detected_negative_case() {
    let traces = generate_traces();
    let mut altered = traces.clone();
    altered[2].rows[10].neurons[0].lifespan += 1;
    assert_ne!(altered, traces);
    // A different growth rate must change the trace.
    let held: Vec<(u64, u64)> = (0..20).map(|b| (b, 1)).collect();
    let other = run_trace("x", settings(9, 2.0, 30, held));
    assert_ne!(other.rows, traces[2].rows);
}

#[test]
fn json_round_trips_exactly() {
    let traces = generate_traces();
    let text = serde_json::to_string(&golden("x", traces.clone())).unwrap();
    let back: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(back.traces, traces);
}

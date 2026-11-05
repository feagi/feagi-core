// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of neuron pacing (refractory period, consecutive fire limit, snooze period)
//! for a teaching visual.
//!
//! Each trace drives the real engine rule on a `StdNeuronArray` with one neuron: every burst it
//! calls the public `process_neural_dynamics` (`neural_dynamics.rs`) with a fire candidate list
//! holding the burst's input, then `finish_burst_refractory_period` with the set of neurons that
//! fired, exactly as the burst loop does. Fired flags come from the returned fire queue. By
//! default (plain `cargo test`) the test regenerates the traces in memory and asserts they equal
//! `tests/fixtures/neuron-pacing/golden-traces.json`. `feagi_core_commit` is ignored.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-npu/burst-engine/tests/fixtures/neuron-pacing:<feagi-react-core-pro>/src/feagi-academy/visuals/neuron-pacing/fixtures \
//! cargo test -p feagi-npu-burst-engine --test neuron_pacing_golden_traces
//! ```
//!
//! `FEAGI_WRITE_VISUAL_FIXTURES` is a `:`-separated list of directories; `golden-traces.json`
//! is written into each. The commit is a note only.
//!
//! Setup: one neuron, fire threshold 1.0, no leak, excitability 1 (always fires when allowed),
//! membrane charge accumulation on. `drive` is the candidate potential of the burst: 2.0 is
//! "strong input held on" (always crosses the threshold), 0.25 is a weak burst that does not.
//! Row semantics, per burst: `fired` is membership in the fire queue; `blocked_by` is `null`
//! when it fired, `"rest"` when the refractory countdown was above 0 at the start of the burst,
//! `"limit"` when the threshold was crossed but the consecutive fire limit blocked the fire,
//! `"input"` when the charge stayed below the threshold. `countdown` and `fire_count` are the
//! neuron's refractory countdown and consecutive fire count after
//! `finish_burst_refractory_period`; `membrane_potential` is the stored charge at that point.

use ahash::AHashSet;
use feagi_npu_burst_engine::neural_dynamics::{
    finish_burst_refractory_period, process_neural_dynamics,
};
use feagi_npu_neural::types::{FireCandidateList, NeuronId};
use feagi_npu_runtime::{NeuronStorage, StdNeuronArray};
use serde::{Deserialize, Serialize};
use std::path::Path;

const BURSTS: u64 = 16;
const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/neuron-pacing";
const THRESHOLD: f32 = 1.0;
const STRONG: f32 = 2.0;
const WEAK: f32 = 0.25;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    refractory_period: u16,
    /// 0 means no limit.
    consecutive_fire_limit: u16,
    snooze_period: u16,
    /// Bursts (0-based) that carry a weak drive instead of the strong one.
    weak_bursts: Vec<u64>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    burst: u64,
    drive: f32,
    fired: bool,
    blocked_by: Option<String>,
    countdown: u16,
    fire_count: u16,
    membrane_potential: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Trace {
    name: String,
    settings: Settings,
    rows: Vec<Row>,
}

#[derive(Serialize, Deserialize)]
struct Golden {
    source_crate: String,
    functions: Vec<String>,
    feagi_core_commit: String,
    traces: Vec<Trace>,
}

fn run_trace(name: &str, s: Settings) -> Trace {
    let mut storage = StdNeuronArray::<f32>::new(4);
    let idx = NeuronStorage::add_neuron(
        &mut storage,
        THRESHOLD,
        f32::MAX, // threshold limit: no upper bound
        0.0,      // leak
        0.0,      // resting potential
        0,        // excitatory
        s.refractory_period,
        1.0, // excitability
        s.consecutive_fire_limit,
        s.snooze_period,
        true, // mp_charge_accumulation
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let id = idx as u32;
    let mut sparse = ahash::AHashMap::new();
    let mut rows = Vec::new();
    for burst in 0..BURSTS {
        let drive = if s.weak_bursts.contains(&burst) {
            WEAK
        } else {
            STRONG
        };
        let countdown_before = storage.refractory_countdowns()[idx];
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id), drive);
        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut sparse,
            None,
            &mut storage,
            burst,
        )
        .unwrap();
        let fired = result
            .fire_queue
            .get_all_neuron_ids()
            .contains(&NeuronId(id));
        let mut fired_set = AHashSet::new();
        if fired {
            fired_set.insert(id);
        }
        finish_burst_refractory_period(&mut storage, &fired_set);
        let blocked_by = if fired {
            None
        } else if countdown_before > 0 {
            Some("rest".to_string())
        } else if drive >= THRESHOLD {
            Some("limit".to_string())
        } else {
            Some("input".to_string())
        };
        rows.push(Row {
            burst,
            drive,
            fired,
            blocked_by,
            countdown: storage.refractory_countdowns()[idx],
            fire_count: storage.consecutive_fire_counts()[idx],
            membrane_potential: storage.membrane_potentials()[idx]
                + storage.membrane_fractions()[idx],
        });
    }
    Trace {
        name: name.to_string(),
        settings: s,
        rows,
    }
}

fn settings(r: u16, l: u16, sn: u16, weak: &[u64]) -> Settings {
    Settings {
        refractory_period: r,
        consecutive_fire_limit: l,
        snooze_period: sn,
        weak_bursts: weak.to_vec(),
    }
}

fn generate_traces() -> Vec<Trace> {
    vec![
        // The guide's default: three fires, then a rest of refrac + snooze.
        run_trace(
            "default_refractory_1_limit_3_snooze_2",
            settings(1, 3, 2, &[]),
        ),
        // Guide step 1 and 2: no limit.
        run_trace("refractory_1_no_limit", settings(1, 0, 2, &[])),
        run_trace("refractory_2_no_limit", settings(2, 0, 0, &[])),
        // Refractory 0 exercises the pre-fire limit check.
        run_trace("refractory_0_no_limit", settings(0, 0, 0, &[])),
        run_trace("refractory_0_limit_3_snooze_0", settings(0, 3, 0, &[])),
        run_trace("refractory_0_limit_3_snooze_2", settings(0, 3, 2, &[])),
        run_trace("refractory_0_limit_1_snooze_0", settings(0, 1, 0, &[])),
        run_trace("refractory_2_limit_2_snooze_1", settings(2, 2, 1, &[])),
        run_trace("limit_2_snooze_0_refractory_1", settings(1, 2, 0, &[])),
        run_trace("refractory_5_limit_6_snooze_6", settings(5, 6, 6, &[])),
        // Snooze with no limit does nothing.
        run_trace("snooze_ignored_without_limit", settings(1, 0, 6, &[])),
        // Weak bursts: the no-fire path resets the consecutive fire count.
        run_trace("refractory_0_limit_3_weak_gap", settings(0, 3, 2, &[1, 5])),
        run_trace("default_with_weak_gap", settings(1, 3, 2, &[3, 4, 9])),
    ]
}

fn golden(commit: &str, traces: Vec<Trace>) -> Golden {
    Golden {
        source_crate: "feagi-npu-burst-engine".to_string(),
        functions: vec![
            "process_neural_dynamics".to_string(),
            "finish_burst_refractory_period".to_string(),
        ],
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn neuron_pacing_golden_traces_match_checked_in_json() {
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
        "neuron pacing drifted from the golden traces"
    );

    // Sanity: the guide's outcomes.
    let fired = |name: &str| -> Vec<u64> {
        traces
            .iter()
            .find(|t| t.name == name)
            .unwrap()
            .rows
            .iter()
            .filter(|r| r.fired)
            .map(|r| r.burst)
            .collect()
    };
    assert_eq!(
        fired("default_refractory_1_limit_3_snooze_2"),
        vec![0, 2, 4, 8, 10, 12]
    );
    assert_eq!(
        fired("refractory_1_no_limit"),
        vec![0, 2, 4, 6, 8, 10, 12, 14]
    );
    assert_eq!(fired("refractory_2_no_limit"), vec![0, 3, 6, 9, 12, 15]);
}

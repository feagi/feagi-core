// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of how a temporal window becomes one memory pattern, for a teaching visual.
//!
//! Each trace drives a real `FireLedger` (`track_area`, `archive_burst`,
//! `get_dense_window_bitmaps`) and the real `PatternDetector::detect_pattern`. No NPU is
//! involved. Scope stops at pattern identity: neuron lifecycle (creation, ageing, lifespan)
//! belongs to the `memory_formation_golden_traces` test.
//!
//! By default (plain `cargo test`) the test regenerates the traces in memory and asserts they
//! equal `tests/fixtures/memory-window/golden-traces.json`. `feagi_core_commit` is ignored by
//! the comparison.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-npu/plasticity/tests/fixtures/memory-window:<feagi-react-core-pro>/src/feagi-academy/visuals/memory-window/fixtures \
//! cargo test -p feagi-npu-plasticity --test memory_window_golden_traces
//! ```
//!
//! Setup per trace. `temporal_depth` quiet frames are archived first (timesteps 1..=depth),
//! so the area looks like it has already been running quietly; the `track_area` window size
//! equals the depth (it must be at least the depth, `fire_ledger.rs`
//! `get_dense_window_bitmaps`). Then one frame per input is archived at strictly increasing
//! timesteps (`archive_burst` rejects a non-increasing one). After each input the dense window
//! of `temporal_depth` frames ending at that timestep is read and handed to `detect_pattern`.
//!
//! Row semantics. `input` is `A`, `B` or `quiet`. `window` lists the labels of the frames in
//! the window, oldest first. `outcome` is `new` (cache miss: a new pattern), `seen` (cache hit:
//! the same pattern again) or `none` (the window was too quiet to form a pattern). Both come
//! from `get_stats()` deltas around the call, not from the returned value. `pattern` is the
//! identity of the pattern, numbered in order of first appearance within the trace (the hash
//! itself is deliberately not stored); `null` for `none`. `memories` is
//! `cached_pattern_count()` after the call.
//!
//! The newest frame of a window is masked to the neurons that fired live this burst
//! (`PlasticityService`, `service.rs`: `pattern_bitmap_for_frame`). Here every burst archives
//! exactly what fired, so the live ids equal the ledger ids and the mask changes nothing; it
//! is mirrored below for fidelity.

use feagi_npu_burst_engine::fire_structures::{FireQueue, FiringNeuron};
use feagi_npu_burst_engine::FireLedger;
use feagi_npu_neural::types::NeuronId;
use feagi_npu_plasticity::{PatternConfig, PatternDetector};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/memory-window";
const UPSTREAM_AREA: u32 = 3;
const MEMORY_AREA: u32 = 5;
const A_IDS: [u32; 2] = [1, 2];
const B_IDS: [u32; 2] = [3, 4];

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Settings {
    temporal_depth: u32,
    /// `A`, `B` or `quiet`, one per burst.
    inputs: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Row {
    burst: u64,
    input: String,
    window: Vec<String>,
    outcome: String,
    pattern: Option<u32>,
    memories: usize,
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

fn ids_for(label: &str) -> Vec<u32> {
    match label {
        "A" => A_IDS.to_vec(),
        "B" => B_IDS.to_vec(),
        "quiet" => Vec::new(),
        other => panic!("unknown input {other}"),
    }
}

fn label_of(set: &HashSet<u32>) -> String {
    let mut ids: Vec<u32> = set.iter().copied().collect();
    ids.sort_unstable();
    if ids.is_empty() {
        "quiet".to_string()
    } else if ids == A_IDS {
        "A".to_string()
    } else if ids == B_IDS {
        "B".to_string()
    } else {
        panic!("unexpected frame {ids:?}")
    }
}

fn queue_for(ids: &[u32], timestep: u64) -> FireQueue {
    let mut queue = FireQueue::new();
    queue.timestep = timestep;
    for &id in ids {
        queue.add_neuron(FiringNeuron {
            neuron_id: NeuronId(id),
            cortical_idx: UPSTREAM_AREA,
            ..Default::default()
        });
    }
    queue
}

fn run_trace(name: &str, s: Settings) -> Trace {
    let depth = s.temporal_depth as usize;
    let mut ledger = FireLedger::new(16);
    ledger.track_area(UPSTREAM_AREA, depth).unwrap();
    // An area that has already been running quietly: `depth` quiet frames first.
    let mut timestep: u64 = 0;
    for _ in 0..depth {
        timestep += 1;
        ledger
            .archive_burst(timestep, &queue_for(&[], timestep))
            .unwrap();
    }

    let detector = PatternDetector::new(PatternConfig::default());
    let mut identities: HashMap<u64, u32> = HashMap::new();
    let mut rows = Vec::new();

    for (burst, input) in s.inputs.iter().enumerate() {
        timestep += 1;
        let live_ids = ids_for(input);
        ledger
            .archive_burst(timestep, &queue_for(&live_ids, timestep))
            .unwrap();
        let window = ledger
            .get_dense_window_bitmaps(UPSTREAM_AREA, timestep, depth)
            .unwrap();
        assert_eq!(window.len(), depth);

        // Newest frame masked to live firings, as the plasticity service does.
        let newest = window.len() - 1;
        let sets: Vec<HashSet<u32>> = window
            .iter()
            .enumerate()
            .map(|(i, (_t, bitmap))| {
                if i == newest {
                    live_ids.iter().copied().collect()
                } else {
                    bitmap.iter().collect()
                }
            })
            .collect();
        let labels: Vec<String> = sets.iter().map(label_of).collect();

        let before = detector.get_stats();
        let pattern = detector.detect_pattern(
            MEMORY_AREA,
            &[UPSTREAM_AREA],
            timestep,
            sets,
            Some(s.temporal_depth),
        );
        let after = detector.get_stats();

        let hits = after.cache_hits - before.cache_hits;
        let misses = after.cache_misses - before.cache_misses;
        let empties = after.empty_patterns - before.empty_patterns;
        assert_eq!(hits + misses + empties, 1, "exactly one outcome per burst");
        let outcome = if misses == 1 {
            "new"
        } else if hits == 1 {
            "seen"
        } else {
            "none"
        };
        assert_eq!(pattern.is_some(), outcome != "none");

        let identity = pattern.map(|p| {
            let next = identities.len() as u32;
            *identities.entry(p.pattern_hash).or_insert(next)
        });
        rows.push(Row {
            burst: burst as u64,
            input: input.clone(),
            window: labels,
            outcome: outcome.to_string(),
            pattern: identity,
            memories: detector.cached_pattern_count(),
        });
    }

    Trace {
        name: name.to_string(),
        settings: s,
        rows,
    }
}

fn settings(temporal_depth: u32, inputs: &[&str]) -> Settings {
    Settings {
        temporal_depth,
        inputs: inputs.iter().map(|s| s.to_string()).collect(),
    }
}

fn generate_traces() -> Vec<Trace> {
    vec![
        run_trace("depth1_abab", settings(1, &["A", "B", "A", "B"])),
        run_trace("depth2_abab", settings(2, &["A", "B", "A", "B"])),
        run_trace("depth2_baba", settings(2, &["B", "A", "B", "A"])),
        run_trace(
            "depth3_ababab",
            settings(3, &["A", "B", "A", "B", "A", "B"]),
        ),
        run_trace(
            "depth1_quiet_gap",
            settings(1, &["A", "quiet", "A", "quiet", "quiet"]),
        ),
        run_trace(
            "depth2_quiet_gap",
            settings(2, &["A", "quiet", "quiet", "A", "quiet", "quiet", "quiet"]),
        ),
        run_trace(
            "depth3_quiet_gap",
            settings(
                3,
                &[
                    "A", "quiet", "quiet", "quiet", "quiet", "A", "quiet", "quiet", "quiet",
                ],
            ),
        ),
        run_trace(
            "depth2_all_quiet",
            settings(2, &["quiet", "quiet", "quiet"]),
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
            "FireLedger::track_area".to_string(),
            "FireLedger::archive_burst".to_string(),
            "FireLedger::get_dense_window_bitmaps".to_string(),
            "PatternDetector::detect_pattern".to_string(),
        ],
        feagi_core_commit: commit.to_string(),
        traces,
    }
}

#[test]
fn memory_window_golden_traces_match_checked_in_json() {
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
        "memory window drifted from the golden traces"
    );
}

fn outcomes(trace: &Trace) -> Vec<&str> {
    trace.rows.iter().map(|r| r.outcome.as_str()).collect()
}

fn windows(trace: &Trace) -> Vec<String> {
    trace.rows.iter().map(|r| r.window.join(" ")).collect()
}

#[test]
fn traces_show_what_the_guide_claims() {
    let traces = generate_traces();
    let by_name = |n: &str| traces.iter().find(|t| t.name == n).unwrap();

    // Depth 1: each burst stands alone; two memories, each seen again.
    let d1 = by_name("depth1_abab");
    assert_eq!(outcomes(d1), ["new", "new", "seen", "seen"]);
    assert_eq!(d1.rows[3].memories, 2);
    assert_eq!(
        d1.rows.iter().map(|r| r.pattern).collect::<Vec<_>>(),
        [Some(0), Some(1), Some(0), Some(1)]
    );

    // Depth 2: "- A", "A B", "B A" are three memories, then "A B" is seen again.
    let d2 = by_name("depth2_abab");
    assert_eq!(windows(d2), ["quiet A", "A B", "B A", "A B"]);
    assert_eq!(outcomes(d2), ["new", "new", "new", "seen"]);
    assert_eq!(d2.rows[3].pattern, d2.rows[1].pattern);
    assert_ne!(
        d2.rows[1].pattern, d2.rows[2].pattern,
        "A B differs from B A"
    );
    assert_eq!(d2.rows[3].memories, 3);

    // B then A is a different order: its windows are a different set of memories.
    let b2 = by_name("depth2_baba");
    assert_eq!(windows(b2), ["quiet B", "B A", "A B", "B A"]);
    assert_eq!(outcomes(b2), ["new", "new", "new", "seen"]);

    // Depth 3: four distinct windows over six bursts; A B A and B A B differ.
    let d3 = by_name("depth3_ababab");
    assert_eq!(
        windows(d3),
        [
            "quiet quiet A",
            "quiet A B",
            "A B A",
            "B A B",
            "A B A",
            "B A B"
        ]
    );
    assert_eq!(outcomes(d3), ["new", "new", "new", "new", "seen", "seen"]);

    // Quiet gaps: a quiet frame serialises as length 0; a window with any activity still
    // forms; an all-quiet window forms nothing.
    let g1 = by_name("depth1_quiet_gap");
    assert_eq!(outcomes(g1), ["new", "none", "seen", "none", "none"]);
    assert!(g1.rows[1].pattern.is_none());
    let g2 = by_name("depth2_quiet_gap");
    assert_eq!(
        outcomes(g2),
        ["new", "new", "none", "seen", "seen", "none", "none"]
    );
    let g3 = by_name("depth3_quiet_gap");
    assert_eq!(
        outcomes(g3),
        ["new", "new", "new", "none", "none", "seen", "seen", "seen", "none"]
    );
    let all_quiet = by_name("depth2_all_quiet");
    assert_eq!(outcomes(all_quiet), ["none", "none", "none"]);
    assert!(all_quiet.rows.iter().all(|r| r.memories == 0));
}

#[test]
fn drift_is_detected_negative_case() {
    let traces = generate_traces();
    let mut altered = traces.clone();
    altered[1].rows[3].outcome = "new".to_string();
    assert_ne!(altered, traces);
    // A different depth must change the outcomes for the same inputs.
    let other = run_trace("x", settings(1, &["A", "B", "A", "B"]));
    assert_ne!(other.rows, traces[1].rows);
}

#[test]
fn json_round_trips_exactly() {
    let traces = generate_traces();
    let text = serde_json::to_string(&golden("x", traces.clone())).unwrap();
    let back: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(back.traces, traces);
}

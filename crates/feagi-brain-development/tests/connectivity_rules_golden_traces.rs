// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Golden traces of which destination voxels one source voxel connects to, for the Academy
//! "Connectivity rules" and "Write your own connectivity rule" visuals.
//!
//! Every core rule that feagi-core implements is driven through the real function that
//! `connectome_manager.rs` calls for it, for every voxel of a small source grid:
//!
//! - vectors (`block_to_block`, `lateral_*`): vectors from `barebones_genome.json`, through
//!   `apply_vector_offset` (scalar 1.0, as `core_morphologies/vectors.rs` calls it);
//! - patterns (`all_to_all`, `0-0-0_to_all`, `all_to_0-0-0`): patterns from the genome, tokens
//!   parsed by `PatternElement::from_value`, through `match_patterns_batch`;
//! - functions: `syn_projector` (also for `transpose_*`, with the transpose the dispatcher
//!   passes), `calculate_centered_projection`, `syn_rotator_z`, `next_sweep_position`,
//!   `first_to_last_destination`, `last_to_first_destination`, `map_positions_for_bitmask`;
//! - composite: `tile_destinations` with `tile_mode`.
//!
//! The file also holds the cases the pattern and vector editor replays (every token kind on
//! either side, out-of-bounds vectors) and a table of how `PatternElement::from_value` parses
//! token strings, including strings it silently turns into `*`.
//!
//! `randomizer` and `lateral_pairs_x` are in the genome but have no code path in
//! `apply_function_morphology` (it falls through to `Ok(0)`), so they have no trace.
//!
//! By default the test regenerates the traces and asserts they equal
//! `tests/fixtures/connectivity-rules/golden-traces.json`. `feagi_core_commit` is ignored.
//!
//! Regenerate both checked-in copies:
//!
//! ```text
//! FEAGI_CORE_COMMIT=$(git rev-parse HEAD) \
//! FEAGI_WRITE_VISUAL_FIXTURES=<feagi-core>/crates/feagi-brain-development/tests/fixtures/connectivity-rules:<feagi-react-core-pro>/src/feagi-academy/visuals/connectivity-rules/fixtures \
//! cargo test -p feagi-brain-development --test connectivity_rules_golden_traces
//! ```

use feagi_brain_development::connectivity::core_morphologies::{
    calculate_centered_projection, first_to_last_destination, last_to_first_destination,
    map_positions_for_bitmask, next_sweep_position, tile_destinations, tile_mode, BitmaskAxis,
    BitmaskMode,
};
use feagi_brain_development::connectivity::rules::patterns::Direction;
use feagi_brain_development::connectivity::rules::{
    apply_vector_offset, match_patterns_batch, syn_projector, syn_rotator_z, Pattern3D,
    PatternElement,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

const FILE_NAME: &str = "golden-traces.json";
const FIXTURE_DIR: &str = "tests/fixtures/connectivity-rules";
const GENOME: &str = "../feagi-evolutionary/genomes/barebones_genome.json";

type Dims = (usize, usize, usize);
type Voxel = (u32, u32, u32);
type Walker = Box<dyn FnMut(Voxel) -> Vec<Voxel>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SourceTrace {
    source: [u32; 3],
    targets: Vec<[u32; 3]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Case {
    id: String,
    /// Empty when the rule has one dimension setup.
    variant: String,
    /// `vectors`, `patterns` or `function`.
    kind: String,
    /// Genome definition for `vectors` and `patterns`; null for `function`.
    definition: Value,
    source_dims: [usize; 3],
    destination_dims: [usize; 3],
    sources: Vec<SourceTrace>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TokenParse {
    text: String,
    element: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Golden {
    feagi_core_commit: String,
    cases: Vec<Case>,
    tokens: Vec<TokenParse>,
}

/// One core rule and the dimensions it is traced on.
struct Spec {
    id: &'static str,
    variant: &'static str,
    src: Dims,
    dst: Dims,
}

const fn spec(id: &'static str, variant: &'static str, src: Dims, dst: Dims) -> Spec {
    Spec {
        id,
        variant,
        src,
        dst,
    }
}

fn core_specs() -> Vec<Spec> {
    let g4 = (4, 4, 1);
    let tall = (2, 2, 3);
    let asym = (4, 3, 2);
    vec![
        // Vectors
        spec("block_to_block", "", g4, g4),
        spec("lateral_+x", "", g4, g4),
        spec("lateral_-x", "", g4, g4),
        spec("lateral_+y", "", g4, g4),
        spec("lateral_-y", "", g4, g4),
        spec("lateral_+z", "", tall, tall),
        spec("lateral_-z", "", tall, tall),
        // Patterns
        spec("all_to_all", "", g4, g4),
        spec("0-0-0_to_all", "", g4, g4),
        spec("all_to_0-0-0", "", g4, g4),
        // Functions
        spec("projector", "scale_down", g4, (2, 2, 1)),
        spec("projector", "scale_up", (2, 2, 1), g4),
        spec("centered_projector", "to_larger", (3, 3, 1), (5, 5, 1)),
        spec("centered_projector", "to_smaller", (5, 5, 1), (3, 3, 1)),
        spec("centered_projector", "even_to_odd", g4, (3, 3, 1)),
        spec("transpose_xy", "", asym, asym),
        spec("transpose_xz", "", asym, asym),
        spec("transpose_yz", "", asym, asym),
        spec("rotator_z", "", (5, 5, 1), (5, 5, 3)),
        spec("sweeper", "", (3, 2, 2), (3, 2, 2)),
        spec("first_to_last", "", (3, 2, 2), (2, 3, 2)),
        spec("last_to_first", "", (3, 2, 2), (2, 3, 2)),
        spec("bitmask_encoder_x", "", (3, 1, 1), (8, 1, 1)),
        spec("bitmask_encoder_y", "", (1, 3, 1), (1, 8, 1)),
        spec("bitmask_encoder_z", "", (1, 1, 3), (1, 1, 8)),
        spec("bitmask_decoder_x", "", (8, 1, 1), (3, 1, 1)),
        spec("bitmask_decoder_y", "", (1, 8, 1), (1, 3, 1)),
        spec("bitmask_decoder_z", "", (1, 1, 8), (1, 1, 3)),
        // Composite
        spec("tile", "fold", g4, (2, 2, 1)),
        spec("tile", "replicate", (2, 2, 1), g4),
    ]
}

fn element(value: &Value) -> PatternElement {
    match value {
        Value::String(s) => PatternElement::from_value(s),
        other => PatternElement::from_value(&other.to_string()),
    }
}

fn pattern3d(triple: &Value) -> Pattern3D {
    let a = triple.as_array().expect("triple");
    (element(&a[0]), element(&a[1]), element(&a[2]))
}

fn all_sources(dims: Dims) -> Vec<(u32, u32, u32)> {
    let mut out = Vec::new();
    for z in 0..dims.2 as u32 {
        for y in 0..dims.1 as u32 {
            for x in 0..dims.0 as u32 {
                out.push((x, y, z));
            }
        }
    }
    out
}

fn arr(d: Dims) -> [usize; 3] {
    [d.0, d.1, d.2]
}

fn finish(
    id: &str,
    variant: &str,
    kind: &str,
    definition: Value,
    src: Dims,
    dst: Dims,
    mut each: impl FnMut((u32, u32, u32)) -> Vec<(u32, u32, u32)>,
) -> Case {
    let sources = all_sources(src)
        .into_iter()
        .map(|s| {
            let mut targets = each(s);
            targets.sort_unstable();
            targets.dedup();
            SourceTrace {
                source: [s.0, s.1, s.2],
                targets: targets.into_iter().map(|t| [t.0, t.1, t.2]).collect(),
            }
        })
        .collect();
    Case {
        id: id.to_string(),
        variant: variant.to_string(),
        kind: kind.to_string(),
        definition,
        source_dims: arr(src),
        destination_dims: arr(dst),
        sources,
    }
}

fn vector_case(id: &str, variant: &str, vectors: &Value, src: Dims, dst: Dims) -> Case {
    let vecs: Vec<(i32, i32, i32)> = vectors
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let v = v.as_array().unwrap();
            (
                v[0].as_i64().unwrap() as i32,
                v[1].as_i64().unwrap() as i32,
                v[2].as_i64().unwrap() as i32,
            )
        })
        .collect();
    finish(
        id,
        variant,
        "vectors",
        json!({"type": "vectors", "vectors": vectors}),
        src,
        dst,
        |s| {
            vecs.iter()
                .filter_map(|v| apply_vector_offset(s, *v, 1.0, dst))
                .collect()
        },
    )
}

fn pattern_case(id: &str, variant: &str, patterns: &Value, src: Dims, dst: Dims) -> Case {
    let parsed: Vec<(Pattern3D, Pattern3D)> = patterns
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (pattern3d(&p[0]), pattern3d(&p[1])))
        .collect();
    finish(
        id,
        variant,
        "patterns",
        json!({"type": "patterns", "patterns": patterns}),
        src,
        dst,
        |s| match_patterns_batch(s, &parsed, src, dst),
    )
}

fn function_case(sp: &Spec) -> Case {
    let (src, dst) = (sp.src, sp.dst);
    // Axis transposes exactly as `apply_function_morphology` passes them.
    let transpose = match sp.id {
        "transpose_xy" => Some((1, 0, 2)),
        "transpose_yz" => Some((0, 2, 1)),
        "transpose_xz" => Some((2, 1, 0)),
        _ => None,
    };
    let bitmask = |axis: BitmaskAxis, mode: BitmaskMode| {
        move |s| map_positions_for_bitmask(s, src, dst, axis, mode)
    };
    let run: Walker = match sp.id {
        "projector" | "transpose_xy" | "transpose_xz" | "transpose_yz" => Box::new(move |s| {
            syn_projector("", "", 0, src, dst, s, transpose, None).expect("projector")
        }),
        "centered_projector" => Box::new(move |s| {
            calculate_centered_projection(s, src, dst)
                .into_iter()
                .collect()
        }),
        "rotator_z" => Box::new(move |s| syn_rotator_z(s, src, dst).expect("rotator_z")),
        "sweeper" => Box::new(move |s| next_sweep_position(s, dst).into_iter().collect()),
        "first_to_last" => {
            Box::new(move |s| first_to_last_destination(s, src, dst).into_iter().collect())
        }
        "last_to_first" => {
            Box::new(move |s| last_to_first_destination(s, src, dst).into_iter().collect())
        }
        "bitmask_encoder_x" => Box::new(bitmask(BitmaskAxis::X, BitmaskMode::Encoder)),
        "bitmask_encoder_y" => Box::new(bitmask(BitmaskAxis::Y, BitmaskMode::Encoder)),
        "bitmask_encoder_z" => Box::new(bitmask(BitmaskAxis::Z, BitmaskMode::Encoder)),
        "bitmask_decoder_x" => Box::new(bitmask(BitmaskAxis::X, BitmaskMode::Decoder)),
        "bitmask_decoder_y" => Box::new(bitmask(BitmaskAxis::Y, BitmaskMode::Decoder)),
        "bitmask_decoder_z" => Box::new(bitmask(BitmaskAxis::Z, BitmaskMode::Decoder)),
        "tile" => {
            let mode = tile_mode(src, dst);
            Box::new(move |s| tile_destinations(s, src, dst, mode))
        }
        other => panic!("no driver for rule {other}"),
    };
    finish(sp.id, sp.variant, "function", Value::Null, src, dst, run)
}

fn token_case(side: &str, token: &str) -> Case {
    let (src, dst) = ((5, 2, 1), (5, 2, 1));
    let pattern = if side == "dst" {
        json!([[["*", "*", "*"], [token, "?", "?"]]])
    } else {
        json!([[[token, "*", "*"], ["?", "?", "?"]]])
    };
    pattern_case(&format!("token:{side}:{token}"), "", &pattern, src, dst)
}

const DESTINATION_TOKENS: &[&str] = &[
    "*", "?", "!", "2", "1..3", "?+", "?-", "?+=", "?-=", "?+1", "?-1", "?+0", "?-1:?+1",
    "?+1:?+2", "7", "-1", "3..9", "4..1", "?+3", "?-3",
];
const SOURCE_TOKENS: &[&str] = &[
    "*", "2", "1..3", "?", "!", "?+", "?+1", "?-1:?+1", "9", "-1",
];

const PARSE_TEXTS: &[&str] = &[
    "*",
    "?",
    "!",
    "?+",
    "?-",
    "?+=",
    "?-=",
    "0",
    "2",
    "-2",
    "+3",
    "?+1",
    "?-1",
    "?5",
    "?+0",
    "?-1:?+1",
    "?+1:?+3",
    "?-2:?",
    "?:?+1",
    "1..3",
    "3..1",
    "-1..2",
    "2..2",
    "1...3",
    "1..2..3",
    "a..b",
    "1..",
    "..2",
    "?+x",
    "?+-1",
    "?+=1",
    "",
    " 2",
    "2 ",
    "abc",
    "**",
    "N",
    "1.5",
    "?-1:?+1:?",
    "99999999999",
    "?99999999999",
];

fn describe(element: &PatternElement) -> Value {
    match element {
        PatternElement::Wildcard => json!({"kind": "wildcard"}),
        PatternElement::Skip => json!({"kind": "skip"}),
        PatternElement::Exclude => json!({"kind": "exclude"}),
        PatternElement::Exact(v) => json!({"kind": "exact", "value": v}),
        PatternElement::DirectionExclusive(d) => {
            json!({"kind": "direction_exclusive", "positive": *d == Direction::Positive})
        }
        PatternElement::DirectionInclusive(d) => {
            json!({"kind": "direction_inclusive", "positive": *d == Direction::Positive})
        }
        PatternElement::Offset(v) => json!({"kind": "offset", "value": v}),
        PatternElement::Range(lo, hi) => json!({"kind": "range", "lo": lo, "hi": hi}),
        PatternElement::AbsoluteRange(lo, hi) => {
            json!({"kind": "absolute_range", "lo": lo, "hi": hi})
        }
    }
}

fn generate(commit: &str) -> Golden {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(GENOME);
    let genome: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("genome")).unwrap();
    let morphologies = &genome["neuron_morphologies"];

    let mut cases = Vec::new();
    for sp in core_specs() {
        let morphology = &morphologies[sp.id];
        let case = match morphology["type"].as_str().unwrap_or("functions") {
            "vectors" => vector_case(
                sp.id,
                sp.variant,
                &morphology["parameters"]["vectors"],
                sp.src,
                sp.dst,
            ),
            "patterns" => pattern_case(
                sp.id,
                sp.variant,
                &morphology["parameters"]["patterns"],
                sp.src,
                sp.dst,
            ),
            _ => function_case(&sp),
        };
        cases.push(case);
    }

    // Editor cases: vectors, including offsets that leave the destination grid.
    let g4 = (4, 4, 1);
    cases.push(vector_case(
        "editor:vectors",
        "xy",
        &json!([[1, 0, 0], [0, -1, 0], [2, 1, 0], [-3, 0, 0]]),
        g4,
        g4,
    ));
    cases.push(vector_case(
        "editor:vectors",
        "depth",
        &json!([[0, 0, 1], [1, 1, -1]]),
        (3, 3, 3),
        (3, 3, 3),
    ));
    cases.push(vector_case(
        "editor:vectors",
        "smaller_destination",
        &json!([[0, 0, 0]]),
        g4,
        (2, 2, 1),
    ));
    // Editor cases: patterns, every token on either side, several rows, depth.
    for token in DESTINATION_TOKENS.iter().copied() {
        cases.push(token_case("dst", token));
    }
    for token in SOURCE_TOKENS.iter().copied() {
        cases.push(token_case("src", token));
    }
    cases.push(pattern_case(
        "editor:patterns",
        "two_rows",
        &json!([
            [[0, "*", "*"], ["?", "?+1", "?"]],
            [["*", 1, "*"], ["!", "?", "?"]]
        ]),
        g4,
        g4,
    ));
    cases.push(pattern_case(
        "editor:patterns",
        "depth",
        &json!([
            [["*", "*", 1], ["?", "?", "?+1"]],
            [["*", "*", "!"], ["?", "?", "0..1"]]
        ]),
        (2, 2, 3),
        (2, 2, 3),
    ));

    let tokens = PARSE_TEXTS
        .iter()
        .map(|text| TokenParse {
            text: text.to_string(),
            element: describe(&PatternElement::from_value(text)),
        })
        .collect();

    Golden {
        feagi_core_commit: commit.to_string(),
        cases,
        tokens,
    }
}

#[test]
fn connectivity_rule_golden_traces_match_checked_in_json() {
    let commit = std::env::var("FEAGI_CORE_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let fresh = generate(&commit);

    if let Ok(dirs) = std::env::var("FEAGI_WRITE_VISUAL_FIXTURES") {
        let text = serde_json::to_string_pretty(&fresh).unwrap() + "\n";
        for dir in dirs.split(':').filter(|d| !d.is_empty()) {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(Path::new(dir).join(FILE_NAME), &text).unwrap();
        }
        return;
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIR)
        .join(FILE_NAME);
    let checked_in: Golden = serde_json::from_str(
        &std::fs::read_to_string(&path).expect("fixture missing; see module docs"),
    )
    .unwrap();
    assert_eq!(
        checked_in.cases, fresh.cases,
        "connectivity rules drifted from the golden traces"
    );
    assert_eq!(
        checked_in.tokens, fresh.tokens,
        "pattern token parsing drifted from the golden table"
    );
}

fn targets(case: &str, variant: &str, source: [u32; 3]) -> Vec<[u32; 3]> {
    let golden = generate("unknown");
    golden
        .cases
        .iter()
        .find(|c| c.id == case && c.variant == variant)
        .unwrap_or_else(|| panic!("case {case}/{variant}"))
        .sources
        .iter()
        .find(|t| t.source == source)
        .unwrap()
        .targets
        .clone()
}

#[test]
fn core_rules_show_what_the_visuals_claim() {
    let none: Vec<[u32; 3]> = vec![];
    // Vectors move by one voxel and never clamp.
    assert_eq!(targets("block_to_block", "", [2, 1, 0]), vec![[2, 1, 0]]);
    assert_eq!(targets("lateral_+x", "", [3, 1, 0]), none);
    assert_eq!(targets("lateral_-z", "", [0, 0, 2]), vec![[0, 0, 1]]);
    // Patterns.
    assert_eq!(targets("all_to_all", "", [0, 0, 0]).len(), 16);
    assert_eq!(targets("0-0-0_to_all", "", [1, 0, 0]), none);
    assert_eq!(targets("all_to_0-0-0", "", [3, 3, 0]), vec![[0, 0, 0]]);
    // Projector scales per axis, down and up.
    assert_eq!(
        targets("projector", "scale_down", [3, 2, 0]),
        vec![[1, 1, 0]]
    );
    assert_eq!(
        targets("projector", "scale_up", [1, 0, 0]),
        vec![[2, 0, 0], [2, 1, 0], [3, 0, 0], [3, 1, 0]]
    );
    // Transposes read the source with its axes reordered; the destination keeps its own shape.
    assert_eq!(
        targets("transpose_xz", "", [3, 0, 0]),
        vec![[0, 0, 1], [1, 0, 1]]
    );
    // Bitmask: most significant bit first.
    assert_eq!(
        targets("bitmask_encoder_x", "", [1, 0, 0]),
        vec![[2, 0, 0], [3, 0, 0], [6, 0, 0], [7, 0, 0]]
    );
    assert_eq!(
        targets("bitmask_decoder_x", "", [5, 0, 0]),
        vec![[0, 0, 0], [2, 0, 0]]
    );
    // Tile folds a larger source and replicates a smaller one.
    assert_eq!(targets("tile", "fold", [3, 2, 0]), vec![[1, 0, 0]]);
    assert_eq!(targets("tile", "replicate", [1, 0, 0]).len(), 4);
    // Sweeper ends at the last voxel; first_to_last uses only the first source voxel.
    assert_eq!(targets("sweeper", "", [2, 1, 1]), none);
    assert_eq!(targets("first_to_last", "", [0, 0, 0]), vec![[1, 2, 1]]);
    assert_eq!(targets("first_to_last", "", [1, 0, 0]), none);
    assert_eq!(targets("last_to_first", "", [2, 1, 1]), vec![[0, 0, 0]]);
    // Centered projector, even size picks the lower centre.
    assert_eq!(
        targets("centered_projector", "even_to_odd", [1, 1, 0]),
        vec![[1, 1, 0]]
    );
    // rotator_z: one candidate per destination layer; the middle layer is the identity.
    let rotated = targets("rotator_z", "", [3, 1, 0]);
    assert!(rotated.contains(&[3, 1, 1]));
}

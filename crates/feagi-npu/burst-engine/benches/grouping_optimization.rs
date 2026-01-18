// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Synaptic Grouping Optimization Benchmark
//!
//! Purpose:
//! - Compare different strategies for grouping synaptic contributions by cortical area
//! - Identify the best performing approach for production use
//!
//! Strategies Tested:
//! 1. Baseline: Current sequential HashMap.entry().or_default().push() pattern
//! 2. Presized: Pre-allocate HashMap and Vec capacities
//! 3. Parallel: Rayon parallel fold-reduce pattern
//! 4. SortGroup: Parallel sort followed by sequential scan
//!
//! Real-world context from profiling:
//! - 100K synapses → 17-35ms grouping
//! - 1.6M synapses → 258ms grouping (bottleneck!)
//! - Expected: 1.6M iterations should be ~1-2ms on modern CPU
//! - Current: 130× slower than expected

use std::time::Duration;

use ahash::AHashMap;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use feagi_npu_neural::types::{NeuronId, SynapticContribution};
use feagi_structures::genomic::cortical_area::CorticalID;
use rayon::prelude::*;

/// Propagation result type (matches production code)
type PropagationResult = AHashMap<CorticalID, Vec<(NeuronId, SynapticContribution)>>;

/// Generate realistic test data simulating synaptic propagation output
fn generate_contributions(
    synapse_count: usize,
    area_count: usize,
) -> Vec<(NeuronId, CorticalID, SynapticContribution)> {
    // Realistic distribution:
    // - Most synapses target a few popular areas (power law distribution)
    // - Random neuron IDs
    // - Realistic contribution values (-65025.0 to +65025.0)

    let mut contributions = Vec::with_capacity(synapse_count);

    for i in 0..synapse_count {
        // Power law distribution: 80% of synapses go to 20% of areas
        let area_id = if i % 5 < 4 {
            // 80% go to first 20% of areas
            let area_idx = (i / 5) % (area_count / 5);
            create_cortical_id(area_idx as u32)
        } else {
            // 20% distributed across remaining 80% of areas
            let area_idx = (area_count / 5) + (i % (area_count * 4 / 5));
            create_cortical_id(area_idx as u32)
        };

        let target_neuron = NeuronId((i * 7) as u32); // Deterministic but scattered
        let contribution = SynapticContribution(((i % 255) as f32) * 255.0); // Range: 0-65025

        contributions.push((target_neuron, area_id, contribution));
    }

    contributions
}

/// Create a deterministic CorticalID from an index
fn create_cortical_id(idx: u32) -> CorticalID {
    // Create a base64-encoded string that's valid for CorticalID
    // Use simple encoding: "area_<idx>" -> base64
    let area_name = format!("area_{}", idx);
    let encoded = base64_encode(&area_name);
    CorticalID::try_from_base_64(&encoded).unwrap()
}

/// Simple base64 encoding (matches CorticalID requirements)
fn base64_encode(s: &str) -> String {
    // Manual base64 encoding to avoid external dependency
    // This is sufficient for test data generation
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = s.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }

        let b1 = (buf[0] >> 2) & 0x3F;
        let b2 = ((buf[0] << 4) | (buf[1] >> 4)) & 0x3F;
        let b3 = ((buf[1] << 2) | (buf[2] >> 6)) & 0x3F;
        let b4 = buf[2] & 0x3F;

        result.push(CHARSET[b1 as usize] as char);
        result.push(CHARSET[b2 as usize] as char);
        result.push(if chunk.len() > 1 {
            CHARSET[b3 as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARSET[b4 as usize] as char
        } else {
            '='
        });
    }

    result
}

// ═══════════════════════════════════════════════════════════
// STRATEGY 1: BASELINE (Current Implementation)
// ═══════════════════════════════════════════════════════════

fn group_baseline(
    contributions: Vec<(NeuronId, CorticalID, SynapticContribution)>,
) -> PropagationResult {
    let mut result: PropagationResult = AHashMap::new();
    for (target_neuron, cortical_area, contribution) in contributions {
        result
            .entry(cortical_area)
            .or_default()
            .push((target_neuron, contribution));
    }
    result
}

// ═══════════════════════════════════════════════════════════
// STRATEGY 2: PRESIZED (Pre-allocated HashMap and Vecs)
// ═══════════════════════════════════════════════════════════

fn group_presized(
    contributions: Vec<(NeuronId, CorticalID, SynapticContribution)>,
    estimated_area_count: usize,
) -> PropagationResult {
    // Pre-allocate HashMap with expected number of cortical areas
    let mut result: PropagationResult = AHashMap::with_capacity(estimated_area_count);
    let estimated_size = contributions.len() / estimated_area_count;

    for (target_neuron, cortical_area, contribution) in contributions {
        result
            .entry(cortical_area)
            .or_insert_with(|| Vec::with_capacity(estimated_size))
            .push((target_neuron, contribution));
    }
    result
}

// ═══════════════════════════════════════════════════════════
// STRATEGY 3: PARALLEL (Rayon fold-reduce)
// ═══════════════════════════════════════════════════════════

fn group_parallel(
    contributions: Vec<(NeuronId, CorticalID, SynapticContribution)>,
) -> PropagationResult {
    contributions
        .into_par_iter()
        .fold(
            || AHashMap::<CorticalID, Vec<(NeuronId, SynapticContribution)>>::new(),
            |mut acc, (target_neuron, cortical_area, contribution)| {
                acc.entry(cortical_area)
                    .or_default()
                    .push((target_neuron, contribution));
                acc
            },
        )
        .reduce(
            || AHashMap::new(),
            |mut a, b| {
                for (cortical_id, mut contribs) in b {
                    a.entry(cortical_id).or_default().append(&mut contribs);
                }
                a
            },
        )
}

// ═══════════════════════════════════════════════════════════
// STRATEGY 4: SORT + GROUP (Cache-friendly sequential scan)
// ═══════════════════════════════════════════════════════════

fn group_sort_group(
    mut contributions: Vec<(NeuronId, CorticalID, SynapticContribution)>,
) -> PropagationResult {
    // Sort by cortical area (parallel sort)
    // CorticalID doesn't implement Ord, so we use a custom comparator with the internal bytes
    contributions.par_sort_unstable_by(|(_, a, _), (_, b, _)| a.as_bytes().cmp(b.as_bytes()));

    // Sequential scan with perfect cache locality
    let mut result: PropagationResult = AHashMap::new();
    let mut current_area: Option<CorticalID> = None;
    let mut current_vec = Vec::new();

    for (target_neuron, cortical_area, contribution) in contributions {
        if current_area != Some(cortical_area) {
            if let Some(area) = current_area {
                result.insert(area, current_vec);
                current_vec = Vec::new();
            }
            current_area = Some(cortical_area);
        }
        current_vec.push((target_neuron, contribution));
    }

    // Don't forget the last group
    if let Some(area) = current_area {
        result.insert(area, current_vec);
    }

    result
}

// ═══════════════════════════════════════════════════════════
// BENCHMARKS
// ═══════════════════════════════════════════════════════════

fn bench_grouping_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouping_small");
    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    let synapse_count = 100_000; // 100K synapses (fast case from logs)
    let area_count = 50; // Typical cortical area count

    group.throughput(Throughput::Elements(synapse_count as u64));

    let contributions = generate_contributions(synapse_count, area_count);

    group.bench_function("baseline", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_baseline(contribs))
        });
    });

    group.bench_function("presized", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_presized(contribs, area_count))
        });
    });

    group.bench_function("parallel", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_parallel(contribs))
        });
    });

    group.bench_function("sort_group", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_sort_group(contribs))
        });
    });

    group.finish();
}

fn bench_grouping_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouping_medium");
    group.sample_size(30);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let synapse_count = 500_000; // 500K synapses (moderate case)
    let area_count = 50;

    group.throughput(Throughput::Elements(synapse_count as u64));

    let contributions = generate_contributions(synapse_count, area_count);

    group.bench_function("baseline", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_baseline(contribs))
        });
    });

    group.bench_function("presized", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_presized(contribs, area_count))
        });
    });

    group.bench_function("parallel", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_parallel(contribs))
        });
    });

    group.bench_function("sort_group", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_sort_group(contribs))
        });
    });

    group.finish();
}

fn bench_grouping_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouping_large");
    group.sample_size(20);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    let synapse_count = 1_600_000; // 1.6M synapses (worst case from logs: 258ms!)
    let area_count = 50;

    group.throughput(Throughput::Elements(synapse_count as u64));

    let contributions = generate_contributions(synapse_count, area_count);

    group.bench_function("baseline", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_baseline(contribs))
        });
    });

    group.bench_function("presized", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_presized(contribs, area_count))
        });
    });

    group.bench_function("parallel", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_parallel(contribs))
        });
    });

    group.bench_function("sort_group", |b| {
        b.iter(|| {
            let contribs = black_box(contributions.clone());
            black_box(group_sort_group(contribs))
        });
    });

    group.finish();
}

fn bench_grouping_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouping_scaling");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    let area_count = 50;

    // Test scaling behavior across different synapse counts
    for synapse_count in [50_000, 100_000, 250_000, 500_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*synapse_count as u64));

        let contributions = generate_contributions(*synapse_count, area_count);

        group.bench_with_input(
            BenchmarkId::new("baseline", synapse_count),
            synapse_count,
            |b, _| {
                b.iter(|| {
                    let contribs = black_box(contributions.clone());
                    black_box(group_baseline(contribs))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parallel", synapse_count),
            synapse_count,
            |b, _| {
                b.iter(|| {
                    let contribs = black_box(contributions.clone());
                    black_box(group_parallel(contribs))
                });
            },
        );
    }

    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2))
        .sample_size(30)
}

criterion_group! {
    name = grouping_bench;
    config = criterion_config();
    targets = bench_grouping_small, bench_grouping_medium, bench_grouping_large, bench_grouping_scaling
}
criterion_main!(grouping_bench);

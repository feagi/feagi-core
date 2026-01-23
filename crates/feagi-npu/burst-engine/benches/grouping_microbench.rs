// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Minimal Synaptic Grouping Microbenchmark
//!
//! Compares grouping strategies on realistic data sizes

use std::time::Duration;

use ahash::AHashMap;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rayon::prelude::*;

// Simplified types for testing
type NeuronId = u32;
type CorticalId = u32;
type Contribution = f32;
type PropagationResult = AHashMap<CorticalId, Vec<(NeuronId, Contribution)>>;

/// Generate test data
fn generate_test_data(
    synapse_count: usize,
    area_count: usize,
) -> Vec<(NeuronId, CorticalId, Contribution)> {
    (0..synapse_count)
        .map(|i| {
            let area_id = if i % 5 < 4 {
                (i / 5) % (area_count / 5)
            } else {
                (area_count / 5) + (i % (area_count * 4 / 5))
            } as u32;
            let neuron_id = (i * 7) as u32;
            let contribution = ((i % 255) as f32) * 255.0;
            (neuron_id, area_id, contribution)
        })
        .collect()
}

// Strategy 1: Baseline
fn group_baseline(data: Vec<(NeuronId, CorticalId, Contribution)>) -> PropagationResult {
    let mut result: PropagationResult = AHashMap::new();
    for (neuron_id, cortical_id, contribution) in data {
        result
            .entry(cortical_id)
            .or_default()
            .push((neuron_id, contribution));
    }
    result
}

// Strategy 2: Pre-sized
fn group_presized(
    data: Vec<(NeuronId, CorticalId, Contribution)>,
    area_count: usize,
) -> PropagationResult {
    let mut result: PropagationResult = AHashMap::with_capacity(area_count);
    let estimated_size = data.len() / area_count;

    for (neuron_id, cortical_id, contribution) in data {
        result
            .entry(cortical_id)
            .or_insert_with(|| Vec::with_capacity(estimated_size))
            .push((neuron_id, contribution));
    }
    result
}

// Strategy 3: Parallel
fn group_parallel(data: Vec<(NeuronId, CorticalId, Contribution)>) -> PropagationResult {
    data.into_par_iter()
        .fold(
            || AHashMap::<CorticalId, Vec<(NeuronId, Contribution)>>::new(),
            |mut acc, (neuron_id, cortical_id, contribution)| {
                acc.entry(cortical_id)
                    .or_default()
                    .push((neuron_id, contribution));
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

// Strategy 4: Sort then group
fn group_sort(mut data: Vec<(NeuronId, CorticalId, Contribution)>) -> PropagationResult {
    data.par_sort_unstable_by_key(|(_, cortical_id, _)| *cortical_id);

    let mut result: PropagationResult = AHashMap::new();
    let mut current_area: Option<CorticalId> = None;
    let mut current_vec = Vec::new();

    for (neuron_id, cortical_id, contribution) in data {
        if current_area != Some(cortical_id) {
            if let Some(area) = current_area {
                result.insert(area, current_vec);
                current_vec = Vec::new();
            }
            current_area = Some(cortical_id);
        }
        current_vec.push((neuron_id, contribution));
    }

    if let Some(area) = current_area {
        result.insert(area, current_vec);
    }

    result
}

fn bench_grouping(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouping");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    let area_count = 50;

    // Test different sizes matching production scenarios
    for size in [100_000, 500_000, 1_000_000].iter() {
        let data = generate_test_data(*size, area_count);

        group.bench_with_input(BenchmarkId::new("baseline", size), size, |b, _| {
            b.iter(|| {
                let d = black_box(data.clone());
                black_box(group_baseline(d))
            });
        });

        group.bench_with_input(BenchmarkId::new("presized", size), size, |b, _| {
            b.iter(|| {
                let d = black_box(data.clone());
                black_box(group_presized(d, area_count))
            });
        });

        group.bench_with_input(BenchmarkId::new("parallel", size), size, |b, _| {
            b.iter(|| {
                let d = black_box(data.clone());
                black_box(group_parallel(d))
            });
        });

        group.bench_with_input(BenchmarkId::new("sort", size), size, |b, _| {
            b.iter(|| {
                let d = black_box(data.clone());
                black_box(group_sort(d))
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_grouping);
criterion_main!(benches);

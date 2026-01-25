// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Simple test to verify criterion works

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn simple_bench(c: &mut Criterion) {
    c.bench_function("add_numbers", |b| {
        b.iter(|| {
            let x = black_box(42);
            let y = black_box(58);
            x + y
        });
    });
}

criterion_group!(benches, simple_bench);
criterion_main!(benches);

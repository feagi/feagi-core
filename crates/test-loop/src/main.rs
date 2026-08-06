use rayon::prelude::*;
use std::time::Instant;

const X: usize = 500_000_000;
const BENCH_BATCH_MIN: usize = 64 * 1024;
const BENCH_BATCH_MAX: usize = 4 * 1024 * 1024;

fn maybe_set_one(value: &mut u8) {
    if rand::random::<f64>() < 0.01 {
        *value = 1;
    }
}

fn main() {
    let mut values = vec![0_u8; X];

    values.par_iter_mut().for_each(maybe_set_one);

    let start = Instant::now();

    values
        .par_iter_mut()
        .with_min_len(BENCH_BATCH_MIN)
        .with_max_len(BENCH_BATCH_MAX)
        .for_each(|value| {
            if *value == 1 {
                *value = 2;
            }
        });

    let elapsed = start.elapsed();
    println!("Time taken: {} ns ({elapsed:?})", elapsed.as_nanos());
}

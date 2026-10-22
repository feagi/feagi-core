# Benchmarking tests

Integration tests that measure performance or timing characteristics.

## burst_engine_jitter.rs

### burst_engine_jitter_under_stress

Measures **burst engine jitter** (inter-burst interval stability) of the real burst loop, optionally under lock contention to simulate API stress. No neurons are injected per burst.

- Records timestamps each time the burst count increments.
- Computes inter-burst intervals, then mean, standard deviation, P50/P95/P99.
- Asserts coefficient of variation (CV) and P99 interval stay within bounds.

Optional environment variables:

- `FEAGI_BENCH_JITTER_SECS`: run duration in seconds (default: 5)
- `FEAGI_BENCH_JITTER_HZ`: target burst frequency (default: 30.0)
- `FEAGI_BENCH_JITTER_STRESS`: `"1"` to run a contention thread that periodically calls `get_fcl_snapshot()` (default: `"1"`)

### burst_engine_jitter_with_injection

Measures jitter **with sensory injection load** per burst. Runs five levels: 100, 1,000, 10,000, 100,000, and 1,000,000 neurons injected (staged and drained in Phase 1) each burst.

- For each level: build NPU with that many neurons in a test cortical area, run the burst loop, and have an observer thread inject that many (NeuronId, potential) pairs after each burst so the next burst drains them.
- Collects inter-burst intervals and asserts CV and P99 ratio within level-specific bounds (tighter at low load, looser at 100k/1M).

Run:

```bash
cargo test --release --test benchmarking burst_engine_jitter -- --nocapture
cargo test --release --test benchmarking burst_engine_jitter_with_injection -- --nocapture
```

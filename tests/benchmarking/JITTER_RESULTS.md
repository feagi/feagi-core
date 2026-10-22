# Burst Engine Jitter Benchmark Results

Results from `burst_engine_jitter_with_injection` (15 / 30 / 100 / 1000 Hz, 100 to 1M sensory injections per burst).

## Hardware and environment

| Field | Value |
|-------|--------|
| OS | Darwin 24.6.0 |
| Arch | arm64 |
| Test | `cargo test --release --test benchmarking burst_engine_jitter_with_injection -- --nocapture` |

To capture full hardware (CPU model, core count, RAM) on this machine, run before re-running the test:

```bash
sysctl -n machdep.cpu.brand_string
sysctl -n hw.ncpu
sysctl -n hw.memsize
# or: system_profiler SPHardwareDataType
```

and add the output above.

## Results

Target intervals: 15 Hz = 66.67 ms, 30 Hz = 33.33 ms, 100 Hz = 10 ms, 1000 Hz = 1 ms.

| Hz | Inj | Samples | Mean (ms) | Std (ms) | CV (%) | P99 (ms) | P99 / target |
|----|-----|--------|-----------|----------|--------|----------|--------------|
| 15 | 100 | 80 | 66.66 | 0.49 | 0.7 | 67.69 | 1.02x |
| 15 | 1,000 | 80 | 66.66 | 0.56 | 0.8 | 67.91 | 1.02x |
| 15 | 10,000 | 80 | 66.66 | 0.63 | 0.9 | 68.19 | 1.02x |
| 15 | 100,000 | 80 | 66.74 | 2.19 | 3.3 | 73.52 | 1.10x |
| 15 | 1,000,000 | 81 | 73.68 | 38.19 | 51.8 | 188.00 | 2.82x |
| 30 | 100 | 80 | 33.34 | 0.59 | 1.8 | 34.54 | 1.04x |
| 30 | 1,000 | 80 | 33.33 | 0.54 | 1.6 | 34.40 | 1.03x |
| 30 | 10,000 | 80 | 33.35 | 0.78 | 2.3 | 35.23 | 1.06x |
| 30 | 100,000 | 80 | 33.43 | 3.41 | 10.2 | 44.31 | 1.33x |
| 30 | 1,000,000 | 81 | 49.22 | 37.25 | 75.7 | 177.73 | 5.33x |
| 100 | 100 | 82 | 10.00 | 0.74 | 7.4 | 12.59 | 1.26x |
| 100 | 1,000 | 83 | 10.01 | 0.58 | 5.8 | 11.57 | 1.16x |
| 100 | 10,000 | 82 | 10.05 | 1.16 | 11.5 | 13.19 | 1.32x |
| 100 | 100,000 | 82 | 10.05 | 3.06 | 30.5 | 18.74 | 1.87x |
| 100 | 1,000,000 | 82 | 38.58 | 36.75 | 95.3 | 153.22 | 15.32x |
| 1000 | 100 | 82 | 1.25 | 0.03 | 2.2 | 1.28 | 1.28x |
| 1000 | 1,000 | 160 | 1.25 | 0.05 | 4.0 | 1.40 | 1.40x |
| 1000 | 10,000 | 121 | 1.68 | 0.58 | 34.4 | 2.69 | 2.69x |
| 1000 | 100,000 | 94 | 4.37 | 3.21 | 73.5 | 15.24 | 15.24x |
| 1000 | 1,000,000 | 80 | 43.15 | 48.09 | 111.4 | 172.73 | 172.73x |

## Summary

- **15 Hz:** Mean near target up to 100k injections; 1M raises mean and P99 (about 2.8x target).
- **30 Hz:** Same pattern; 1M gives mean ~49 ms and P99 ~5.3x target.
- **100 Hz:** On target up to 10k; 100k and 1M increase mean and jitter (1M mean ~39 ms, P99 ~15x target).
- **1000 Hz:** On target for 100 and 1k injections; 10k+ exceeds 1 ms target (overload at 100k and 1M).

CV = coefficient of variation (Std/Mean x 100). P99 / target = P99 interval divided by target interval.

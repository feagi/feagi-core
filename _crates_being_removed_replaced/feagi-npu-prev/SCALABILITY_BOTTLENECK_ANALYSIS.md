# FEAGI NPU Scalability Bottleneck Analysis

## Purpose

Capture the current root-cause analysis for burst-loop overruns at larger neuron counts and define high-impact proposals to improve headroom for significantly larger workloads.

## Current Runtime Evidence

Observed in high-activity runs:

- Approximate scale: `~270k` neurons, `~924k` synapses.
- High-activity regime: `~40%` neuron activity (`~114k` fired/burst).
- At `15 Hz` (burst budget `66.67ms`), measured:
  - `process_burst`: roughly `278-333ms`
  - lock hold: roughly `303-358ms`
  - synaptic injections: roughly `~578k`/burst

Conclusion: this is a real capacity overrun (work per burst exceeds timestep budget), not a warning-threshold artifact.

## Primary Bottleneck

### Phase 1 injection path dominates burst cost

In `burst-engine/src/npu.rs`, Phase 1 combines several heavy operations in one hot section:

1. staged sensory drain and candidate insertion,
2. reset/injection logic,
3. synaptic propagation (`propagation_engine.propagate(...)`),
4. per-contribution insertion into FCL (`add_candidate` loops).

At high activity, this path scales with active synapse volume and hash-map accumulation pressure.

## Secondary Bottlenecks

1. **Phase 2 dynamics cost** remains large at high candidate counts.
2. **Lock-hold inflation** from extra in-lock work (queue/sample construction, cloning, map churn).
3. **Coarse lock scope** across multiple heavy stages limits concurrency and worsens tail latency.

## Ranked Improvement Proposals

## 1) Fused Phase 1 accumulation (highest impact)

- Replace multi-step propagation-to-vector-to-hash insertion with fused/thread-local accumulation and single merge pass.
- Minimize transient allocations and repeated hashmap entry operations.
- Expected impact: major reduction in Phase 1 time and allocator pressure under high activity.

## 2) Remove avoidable broad scans in injection/reset path

- Move to strictly touched/dirty-neuron reset semantics where possible.
- Avoid full neuron scans in common runtime paths.
- Expected impact: meaningful constant-time reduction as model size grows.

## 3) Eliminate fire-queue deep clones in hot path

- Replace clone-heavy transitions with double-buffer ownership + `swap`.
- Build downstream sample views with minimal copying.
- Expected impact: reduced lock-hold and memory bandwidth consumption.

## 4) Parallelize Phase 2 with safe sharding

- Partition candidate processing by disjoint neuron shards to enable multi-core mutation safely.
- Preserve deterministic semantics while increasing throughput.
- Expected impact: substantial speedup for large candidate sets on multi-core hosts.

## 5) Tighten lock scope and phase boundaries

- Keep only unavoidable shared-state mutation inside locks.
- Move expensive transforms/aggregation outside lock where correctness permits.
- Expected impact: lower lock wait spikes and better throughput under concurrent services.

## 6) Accelerator path (GPU/WGPU/CUDA) for long-term targets

- Offload propagation and/or dynamics kernels for large active sets.
- Keep CPU deterministic path as baseline and fallback.
- Expected impact: largest long-term ceiling increase for large-scale brains.

## Execution Plan

1. Establish reproducible benchmark harness for representative genomes and activity regimes.
2. Implement in this order:
   - hot-path clone elimination,
   - fused Phase 1 accumulation,
   - reset-path scan elimination,
   - Phase 2 sharded parallelism.
3. Re-measure after each change (`p50/p95/p99`):
   - `phase1_ms`, `phase2_ms`, `process_burst_ms`,
   - lock hold/wait,
   - overrun rate versus timestep budget.
4. Only after performance gains: revisit warning/telemetry verbosity.

## Acceptance Criteria

- Sustained `p95 process_burst_ms` below timestep budget for target workloads with margin.
- Stable overrun rate near zero in steady-state for supported scales.
- No correctness regressions in firing, propagation, and plasticity behavior.
- Deterministic behavior preserved across supported platforms.

# feagi-burst-engine

High-performance neural processing unit (NPU) for FEAGI burst cycle execution.

## Overview

The burst engine processes neural activity in discrete time steps:
- Synaptic propagation
- Neural dynamics (membrane potential updates)
- Firing detection and management
- Optional GPU acceleration (WGPU, CUDA)

## Installation

```toml
[dependencies]
feagi-burst-engine = "2.0"

# With GPU support
feagi-burst-engine = { version = "2.0", features = ["gpu"] }

# With CUDA support (NVIDIA only)
feagi-burst-engine = { version = "2.0", features = ["cuda"] }
```

## Usage

```rust
use feagi_burst_engine::RustNPU;

let mut npu = RustNPU::new(100_000, 1_000_000, 20)?;
npu.load_connectome("brain.json")?;
npu.process_burst()?;
```

## Features

- `gpu` - Cross-platform GPU acceleration via WGPU
- `cuda` - NVIDIA CUDA acceleration
- `all-gpu` - Enable all GPU backends

## Sensory injection cadence and `sensory_hold_last_frame`

Sensory input is injected per burst. Each burst the loop
(`burst_loop_runner.rs`) drains the sensory intake, keeps the newest payload per
source (`select_fresh_payloads_for_burst`, payloads older than one burst period
are dropped), decodes it to `(cortical_id, xyzp)` lists and calls
`inject_sensory_xyzp_by_id` for each area. That stages `(neuron, potential)`
pairs in `pending_sensory_injections` (`npu.rs`,
`inject_sensory_with_potentials`); Phase 1 of `process_burst` moves them into the
fire candidate list for the current burst only (`phase1_injection_with_synapses`:
`fcl.clear()` then drain of the staged pairs). Phase 2 evaluates only fire
candidates.

Consequence: on a burst where no payload arrived for an input area, that area
has no fire candidates and its neurons are silent. Nothing carries over from the
previous burst: a neuron that fired had its membrane potential reset by the
dynamics step, and a sub-threshold candidate from the previous burst is cleared
in Phase 1 unless `mp_charge_accumulation` is on (and even then it is not
re-evaluated without a new candidate entry). A controller streaming a scalar
sensor slightly slower than the burst rate (IMU at ~49 Hz into a 40 Hz brain)
therefore produces input activity only every other burst and downstream
detectors flicker.

`sensory_hold_last_frame` (per cortical area, input areas only) bridges that gap.
When `true`, the loop remembers the last non-empty frame injected into the area
and, on a burst with no new frame, re-injects it once: a frame injected at burst
`n` is re-injected at `n+1` if nothing arrived; if `n+2` also has no frame the
area goes silent. A new frame always replaces the held one. When `false` the
behaviour above is unchanged. The decision is the pure `SensoryHold` type in
`sensory_hold.rs`; the enabled set is pushed from the services layer
(`BurstLoopRunner::refresh_sensory_hold_areas`) whenever a genome loads or a
cortical area is created, updated or deleted.

Default: `true` for scalar sensor units (IMU, encoder/servo position,
proximity, infrared, battery, analog GPIO, Cartesian position, count), `false`
for vision, depth, segmentation, audio, digital GPIO and `MiscData` inputs. The
default comes from the sensory unit template
(`SensoryCorticalUnit::get_default_sensory_hold_last_frame` in
`feagi-structures`); auto-created IPU areas store it as a property, and areas
without the property fall back to it by type
(`resolve_sensory_hold_last_frame`). It is read and written through the cortical
area property routes (`POST /v1/cortical_area/cortical_area_properties`,
`PUT /v1/cortical_area/cortical_area`) as a metadata change (no rebuild) and is
saved in the genome as `snhold-b`.

## Performance

- 50-100x faster than Python implementations
- Supports 30Hz+ burst frequency
- Tested with millions of neurons

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.


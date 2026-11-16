# Deterministic memory test rigs

How to build a single-thread rig that drives a real NPU, a real `PlasticityService` and the
production post-burst step, burst by burst. Reference implementations:
`tests/memory_regression_golden.rs` (golden traces) and `tests/memory_capture_recording.rs`
(`Rig::new`, `Rig::step`).

## Build the rig

1. NPU: `DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), neurons, synapses, 8)`
   wrapped in `Arc<TracingMutex<_>>`.
2. Manager: `ConnectomeManager::new_for_testing_with_npu(npu)`, then
   `setup_core_morphologies_for_testing()`.
3. Plasticity: build an `AsyncPlasticityExecutor` (default `PlasticityConfig`, a memory stats
   cache from `create_memory_stats_cache()`, the NPU), take `get_service()`, and link it with
   `manager.set_plasticity_executor(executor)`. Without the link the manager never registers
   memory areas.
4. Create areas with `add_cortical_area`, then create mappings with `update_cortical_mapping`
   plus `regenerate_synapses_for_mapping`. The mapping call registers the memory area with the
   service.
5. Drive each burst on one thread: inject stimulus, `process_burst`,
   `PlasticityService::run_burst_sync(burst)`, drain the queued commands, then
   `PlasticityRuntime::apply_post_burst(burst)` (feagi-services, plasticity feature).
   Never use a background thread: the service iterates areas in a `HashMap`, so use one memory
   area per rig where order matters.

## Cargo setup

- The root crate already has `feagi-services = { ..., features = ["plasticity"] }` as a
  dev-dependency. Keep it there; a rig that imports `PlasticityRuntime` fails to compile
  without it.
- Tests that touch the runtime inside `feagi-services` need the feature on the command line:
  `cargo test -p feagi-services --features plasticity --test test_replay_runtime`.

## Set memory settings through area properties only

Put lifecycle, MP mode, `capture_max_bursts` and the like in the area's `properties`, then call
`manager.sync_memory_associative_lif_params(&mem_id)` (the same sync a genome load and an area
update use). Do not write NPU parameters directly: the next linked mapping edit re-syncs from
the properties and silently overwrites the direct write. A rig that passes before an edit and
fails after one usually did this.

## Read what fired

- `npu.configure_fire_ledger_window(area_idx, n)` first, so the area is on the fire ledger.
- `get_fire_ledger_recent_frames_with_kinds(area_idx, n)` returns frames with `timestep`,
  `fired` and `kind_of(id)`. Compare kinds with the exported `FIRE_KIND_*` constants (encoding,
  episodic memory, chain recall, STDP eligible).
- Count fires of an episode neuron apart from state fires: filter by the episode neuron's id
  (its member list is in the neuron detail) and by kind. Mixing the two hides a state that
  stopped firing behind an episode neuron that still fires.
- Compare sets per burst, never spike order.

## Golden fixtures

`tests/fixtures/memory-regression/golden-traces.json` and `fire-kinds.json` are the contract
for memory behavior. A memory change must leave both byte-identical. If a golden test fails,
the code is wrong until proven otherwise: find the behavior change. Never regenerate a fixture
to make a failure pass. Regenerate only when a reviewed design deliberately changes recorded
behavior, say so in the review, and keep the diff of the fixture small and explained (the
procedure is in the header of `memory_regression_golden.rs`).

## Checks worth keeping in every rig

- A subscribed but silent driver changes nothing (compare with the unsubscribed trace).
- A mapping edit keeps the state that existed before it.
- Run the rig twice and compare traces: any difference is nondeterminism in the rig.

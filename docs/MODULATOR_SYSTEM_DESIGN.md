# Modulator System Design

**Status:** Implemented in feagi-core, Brain Visualizer, and feagi-mcp. GPU backends remain out of scope. Unit coverage covers the signal model, spike-train steps, factor combination, the v3 to v4 migrator, R-STDP rebaseline, and classifier reward. Burst-isolation and no-modulator performance tests from the Testing section are not in the suite yet.
**Owners:** feagi-core (structures, evolutionary, brain-development, npu, services, api), brain-visualizer, feagi-mcp
**Last updated:** 2026-10-06

## Goal

Add a modulator system that regulates brain-level processes such as neuron
excitability, synaptic transmission, and learning. Modulators come in two kinds:

- **Neuromodulators** act on the neurons of the cortical areas that subscribe to them.
- **Synaptic modulators** act on the synapses of the mappings that subscribe to them.

FEAGI ships a fixed set of **modulator types**. Users create **modulator instances**
of those types with their own parameters, and cortical areas or mappings subscribe
to instances.

Existing reward mechanisms (R-STDP and classifier reward training) move onto this
system so there is one way to deliver reward and modulation signals.

## Scope

In scope:

- Modulator types, instances, and the genome `modulators` section.
- A new modulator cortical area type that drives each instance.
- Area-level and mapping-level subscriptions.
- A general **spike train** neuron property (usable by any area).
- Migrating R-STDP and classifier reward training onto modulators.
- API, Brain Visualizer, and feagi-mcp support.

Out of scope:

- GPU backends (WGSL, CUDA). The NPU is undergoing a major refactor; this work
  makes no GPU changes. GPU parity is a follow-up after the refactor.
- Homeostatic plasticity (`rate_modulated_leak`). It stays a built-in area
  property because it regulates each neuron from that neuron's own firing rate,
  which a single broadcast neuron cannot represent. See
  `crates/feagi-npu/neural/docs/rate_modulated_leak.md`.

## Concepts

| Concept | Defined by | Description |
|---|---|---|
| Modulator type | FEAGI code | A built-in category, e.g. "firing threshold neuromodulator". Fixed set, not user-editable. |
| Modulator instance | Genome `modulators` section | A user-named instance of a type with its own magnitude and timing, e.g. "my excit 1". |
| Driver area | Genome cortical areas | The 1x1x1 modulator cortical area owned by one instance. Its single neuron decides when the instance is active. |
| Subscription | Cortical area or mapping | A reference from an area or a mapping to an instance ID. |

Each instance owns exactly one driver area, and each driver area belongs to exactly
one instance.

## Decisions (agreed)

| # | Decision | Rationale |
|---|---|---|
| 1 | Each instance is driven by a 1x1x1 cortical area with one neuron, placed in the root region on its own plate. | The driver is a normal cortical area: it can be wired, monitored, and connected to other areas or other modulators. |
| 2 | Magnitude lives in the instance definition as a signed percentage (e.g. -80% or +30%). Subscriptions carry no percentage. | One clear meaning per instance. Two areas that need different strengths use two instances. |
| 3 | An instance is active in the bursts its driver neuron fires. Effect duration is set by a spike train on the driver neuron. | Reuses neuron dynamics instead of new per-instance state. |
| 4 | Driver areas have refractory period locked to 0 and spike train locked on. | Required for the spike train to fire on consecutive bursts. |
| 5 | Spike train is a general neuron property available to any cortical area. | Mirrors biological spike trains after an initial stimulation. |
| 6 | Graded effect uses `mp_driven_psp` on the driver area: the effect scales with the driver neuron's firing-time membrane potential. | Preserves graded reward for R-STDP. |
| 7 | Synaptic modulators attach to individual mappings, tracked by a per-synapse modulation-group index. | Two mappings between the same pair of areas (e.g. one excitatory, one inhibitory) must be modulated independently. |
| 8 | The genome has a top-level `modulators` section, similar to `morphologies`. Areas and mappings reference instances by ID. | Same reference model as morphologies. |
| 9 | Instance lifecycle (rename, update, delete) follows the morphology model, including usage lookup. | Consistent behavior across referenced genome objects. |
| 10 | Homeostatic plasticity stays a built-in area property. | Per-neuron self-regulation does not fit a broadcast model. |
| 11 | R-STDP and classifier reward training migrate onto modulators. | One reward and modulation path. |
| 12 | Several instances on one target combine by multiplication, then clamp to the parameter's valid range. | Predictable composition; no instance can push a parameter out of range. |
| 13 | During a spike train, forced firings supersede the refractory period. Refractory and snooze apply after the train ends. | Spike train behaves the same on driver and normal areas. |
| 14 | Area-level effects are applied by rewriting per-neuron values on change (no kernel changes). | Zero cost for unsubscribed areas and independent of the NPU kernel refactor. |
| 15 | Deleting an instance with subscribers is rejected unless `force` is set. With `force`, all subscriptions are removed, then the instance and its driver area are deleted. | Safe by default, clean when explicit. |
| 16 | No GPU changes in this work. | NPU refactor in progress. |

## Modulator types (proposed initial set)

Each type names the parameter it acts on. Exact parameter mapping is listed so the
effect of a magnitude is unambiguous.

| Kind | Type | Acts on | Effect of magnitude `m` at full strength |
|---|---|---|---|
| Neuromodulator | Firing threshold | Neuron firing threshold | threshold x (1 + m) |
| Neuromodulator | Leak | Neuron leak coefficient | leak x (1 + m), clamped to [0, 1] |
| Neuromodulator | Firing probability | Neuron excitability (probabilistic firing) | excitability x (1 + m), clamped to [0, 1] |
| Synaptic | Transmission gain | PSP of the mapping's synapses | psp x (1 + m) |
| Synaptic | Reward | R-STDP reward signal of the mapping | contributes `m x strength` to R(t) |
| Synaptic | Learning rate | STDP / R-STDP eta of the mapping | eta x (1 + m) |

The "Reward" type replaces `reward_source_area` and `punishment_source_area`:
pleasure is a Reward instance with positive magnitude, pain one with negative
magnitude.

## Instance definition

Genome `modulators` section (hierarchical form; flat 3.0 keys follow the existing
flat conventions and are defined during implementation):

```json
"modulators": {
  "my_excit_1": {
    "type": "neuro.firing_threshold",
    "magnitude_percent": 30.0,
    "effect_duration_bursts": 5,
    "rest_bursts": 20,
    "graded": true,
    "full_scale_potential": 10.0,
    "driver_cortical_id": "<base64 cortical id>"
  }
}
```

| Field | Type | Description |
|---|---|---|
| `type` | string | One of the built-in modulator types. |
| `magnitude_percent` | f32 | Signed effect at full strength. |
| `effect_duration_bursts` | u16 >= 1 | Length of the driver neuron's spike train. Written to the driver area's `consecutive_fire_limit`. |
| `rest_bursts` | u16 | Rest after the spike train. Written to the driver area's `snooze_period`. |
| `graded` | bool | Written to the driver area's `mp_driven_psp`. When false, strength is always 1.0. |
| `full_scale_potential` | f32 > 0 | Firing-time membrane potential that produces full strength. Required when `graded` is true. |
| `driver_cortical_id` | string | The instance's driver area. |

The instance definition is the single source of truth for `consecutive_fire_limit`,
`snooze_period`, `mp_driven_psp`, refractory period (0), and spike train (on) on the
driver area. Those fields are read-only on driver areas in the API and the Brain
Visualizer. All other driver area properties (threshold, leak, mappings, etc.)
stay editable.

## Signal model

For instance `k` at burst `t`:

```
fired_k(t)    = driver neuron fired at burst t (including spike-train firings)
strength_k(t) = 0                                       if not fired_k(t)
              = 1.0                                     if fired_k(t) and not graded
              = min(mp_k(t) / full_scale_potential, 1)  if fired_k(t) and graded
signal_k(t)   = magnitude_percent / 100 x strength_k(t)
```

`mp_k(t)` is the firing-time membrane potential captured in the fire queue, the
same value `mp_driven_psp` uses today. Within a spike train, every firing carries
the membrane potential of the firing that started the train (see Spike train).

Timing is deterministic: a driver that fires at burst `t` affects subscribers during
burst `t + 1`. A spike train of length N started at burst `t` therefore affects
bursts `t + 1` to `t + N`.

### Driver neuron behavior during rest

Verified against the current burst engine: while a neuron's refractory countdown is
above 0 (which includes snooze), its membrane is frozen and incoming input is
discarded. A driver therefore
ignores input during `rest_bursts`. Slow build-up comes from a high firing
threshold, not from the rest period.

### Combining several instances on one target

Several instances may target the same parameter of the same area or mapping. Their
factors multiply, and the result is clamped to the parameter's valid range:

```
factor(t)    = product over subscribed k of (1 + signal_k(t))
effective(t) = clamp(baseline x factor(t), param_min, param_max)
```

`magnitude_percent` is validated to be >= -100 so that every factor stays >= 0.

Reward instances are the exception: their signals add into `R(t)` (see R-STDP
migration).

## Spike train (general neuron property)

New area-level property `spike_train: bool`, available on any cortical area.

Behavior when enabled:

1. A neuron fires normally (crosses threshold). Its firing-time membrane potential
   is held.
2. While `1 <= consecutive_fire_count < consecutive_fire_limit`, the neuron is
   force-fired every burst regardless of membrane potential, carrying the held
   potential. Forced firings are real firings: they enter the fire queue, the fire
   ledger, activity monitoring, and outgoing synaptic propagation.
3. When the limit is reached, the existing extended refractory applies
   (refractory period + snooze period).

Validation: `spike_train` requires `consecutive_fire_limit >= 1`. A limit of 0
means "unlimited" today and would produce an endless train, so the combination is
rejected.

Refractory period on non-driver areas: forced firings supersede the refractory
period during the train. Refractory period plus snooze applies once the train ends.

Performance (`@cursor:critical-path`):

- The flag is checked only for neurons that fired, at the point where the
  consecutive fire count is already updated.
- Neurons in an active train go into a sparse active-train list that also holds
  the potential.
- After each burst, only that list is walked to schedule forced firings for the
  next burst, using the existing force-fire-with-potential path (used today by
  memory replay).
- Areas without the flag pay nothing. No change to the LIF kernels.

GPU backends are out of scope (see Scope).

## Applying effects

### Neuromodulators (area level)

After each burst, the engine computes the combined factor per (area, parameter)
from all subscribed instances' signals. When an area's factor differs from the
previous burst, the engine rewrites the effective per-neuron value for that area
from a stored per-neuron baseline. This is the same cold-path pattern as
`rate_modulated_leak`.

- No changes to the LIF kernels, so this work is independent of the NPU refactor.
- Areas with no subscriptions pay nothing.
- Cost is proportional to the subscribed area's neuron count, only on bursts where
  its factor changes (a spike train start or end, or a graded strength change).
- The baseline is per neuron, because values already vary within an area
  (threshold increments along x, y, z; homeostatic leak). Baselines are allocated
  only for (area, parameter) pairs with at least one subscription, and released
  when the last subscription is removed.
- Property updates from the genome or the API write the baseline, then reapply the
  current factor.

Leak needs an explicit composition rule with homeostatic leak, because both would
write the per-neuron leak. Proposed: homeostatic leak updates the baseline, and the
leak modulator scales the baseline.

### Synaptic modulators (mapping level)

- Subscriptions live in the mapping rule object (`"modulators": ["<id>", ...]`),
  next to the existing plasticity parameters.
- During synaptogenesis each mapping rule with subscriptions gets a modulation
  group ID, written to a per-synapse `modulation_groups: Vec<u16>` column
  (0 = no group).
- The column is allocated only when at least one mapping has a synaptic
  modulator, so genomes without them pay no memory.
- The column must survive every path that rebuilds, compacts, or snapshots
  synapses (including the embedded `SynapseArray<N>`).
- Per burst, a small table maps group ID to the combined factor or reward signal.

### R-STDP migration

Today: `R(t) = density(reward_source_area) - density(punishment_source_area)`.

After: `R(t) = sum of signal_k(t) over Reward instances subscribed by the mapping`.
Pleasure and pain become Reward instances with positive and negative magnitude.
Graded reward is preserved through `graded = true`. Learning behavior will differ
from today's density-based reward and must be re-baselined in
`test_rstdp_plasticity.rs`.

`reward_source_area` and `punishment_source_area` are replaced by subscriptions.
A genome schema migrator converts existing genomes: each referenced reward or
punishment area gets a Reward instance and a driver area wired from the old source
area.

### Classifier reward migration

Each classifier's hidden pleasure and pain areas become per-classifier Reward
instances. The per-scanning-instance logic that decides which class channels
receive reward (`classifier_instance_reward`) stays in the classifier path. The
modulator supplies the signal, and the classifier path decides where it applies.

## Driver area type

- New `CorticalAreaType::Modulator` with its own cortical ID prefix.
- Dimensions locked to 1x1x1, one neuron per voxel.
- Placed only in the root region, on a dedicated modulator plate in the Brain
  Visualizer.
- Created and deleted together with its instance. Not creatable on its own.

## Lifecycle

Follows the morphology model:

- **Usage lookup:** an endpoint lists all areas and mappings subscribed to an
  instance (same idea as `/morphology/morphology_usage`).
- **Update:** changing an instance updates its driver area's locked fields.
  Changing `type` is rejected while subscribers exist.
- **Rename:** rewrites all subscriber references.
- **Delete:** enforced in feagi-core, not only in clients.
  - Without `force`: rejected while subscribers exist. The error returns the usage
    list.
  - With `force`: every subscription is removed from areas and mapping rules
    (affected areas revert to baseline values, affected synapses drop their
    modulation group), then the instance and its driver area are deleted. The
    operation is atomic: on any failure nothing is changed.
  - Morphology delete does not check usage in core today. This work does not
    change morphology behavior.
- **Region export and amalgamation:** a region that subscribes to modulators carries
  their instances and driver areas along, the same way imported areas carry their
  morphologies.

## Genome schema

- New top-level `modulators` section.
- New area properties: `modulators` (subscription list) and `spike_train`.
- New field in mapping rule objects: `modulators`.
- `CURRENT_SCHEMA_VERSION` bump with a `vN -> vN+1` migrator that converts R-STDP
  reward and punishment areas, per `GENOME_SCHEMA_VERSIONING.md`.

## API and tooling

feagi-core API (`feagi-api`):

- CRUD for modulator instances, usage lookup, list of modulator types.
- Area and mapping subscription fields in the existing update endpoints.
- `spike_train` in cortical area properties.

Brain Visualizer:

- "Modulations" section in cortical area properties, directly above Advanced.
- Add flow: pick a type (listed like IPU/OPU areas), then create a new instance or
  pick an existing one.
- Modulator subscriptions in the mapping editor.
- Modulator plate in the root region. Locked fields shown read-only on driver areas.

feagi-mcp (public API only):

- List types, CRUD instances, subscribe and unsubscribe areas and mappings, usage
  lookup, delete with `force`.

## Testing

- Unit: signal model (graded and non-graded, clamping), spike train state machine
  (trigger, train, rest, validation of limit 0), factor combination, modulation
  group index through rebuild and compaction.
- Burst engine integration: timing (`t` to `t + 1` to `t + N`), forced firings
  propagate through outgoing synapses, area-level effect on firing, mapping-level
  effect isolated to one of two mappings between the same areas.
- R-STDP and classifier reward: re-baselined learning tests on the modulator path.
- Genome: schema migrator for existing R-STDP genomes, round trip hierarchical to
  flat, lifecycle (rename, delete rejected with subscribers, forced delete removes
  all subscriptions atomically).
- Area-level effects: baseline restored exactly when the last subscription is
  removed; property update while modulated keeps the factor applied.
- Performance: burst time with no modulators must match current baseline.

## Open questions

None. All questions from review are resolved in Decisions #12 to #16.

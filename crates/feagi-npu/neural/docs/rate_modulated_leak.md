# Rate-modulated leak (intrinsic firing-rate homeostasis)

## Purpose

For opt-in **custom** (dense LIF) cortical areas, FEAGI can run a **cold** post-burst pass that updates each neuron’s **effective** `leak_coefficient` from a low-passed estimate of its **firing rate**, producing negative feedback: sustained high activity increases leak and reduces excitability; low activity has the opposite effect. This is **not** synaptic homeostatic plasticity; it does not change synapse weights.

## Hot path vs cold path

- The **LIF** integrate step only **reads** the per-neuron leak value already in storage, unchanged.
- The **homeostat** runs **only** in areas that have a `rate_modulated_leak` entry in the genome / cortical properties with `"enabled": true`, and only **after** the current burst is archived to the fire ledger. **No** extra work when no area opts in (registry is empty: early return).

## Genome and flat 3.0

- Hierarchical: optional object `rate_modulated_leak` on the cortical area blueprint.
- Flat 3.0: key `_____10c-<base64_cortical_id>-cx-hmlk-d` → same JSON object (`hmlk` = homeostatic leak; `-d` = object).

Baseline static leak remains `leak_coefficient` / `nx-leak_c-f` as today; the homeostat **writes** the per-neuron leak slot for the next burst, seeding from the value at registration.

## Algorithm (current implementation)

For each **enabled** area and each regular neuron in that area on eligible bursts:

1. **EMA of firing** (in `[0,1]`, per burst): `r_ema <- (1-α) r_ema + α * 1[neuron fired this burst]`, with `α = 1 - exp(-1/τ_r)` and `τ_r` = `rate_ema_tau_bursts`.
2. **Error** `e = r_ema - target_firing_per_burst` (target in `[0,1]`).
3. **Leak** `g <- clamp(g + gain * e, leak_min, leak_max)`; `g` is also clamped to `[0,1]` for the LIF model.
4. **Cadence** `update_every_n_bursts`: the EMA (step 1) is updated on every burst; the leak step (step 3) is applied only when `burst_count % n == 0` (0 treated as 1). Sampling the EMA every burst keeps neurons that fire between update bursts from looking silent.

Registering an area also ensures the main fire ledger tracks it (window >= 1; an existing larger window is never shrunk), since the homeostat reads firing from the ledger. Removing the area does not untrack it.

Neurons are skipped when their storage index is above the memory neuron band or invalid.

## Parameters (JSON)

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | bool | If false, registration is a no-op and the area is removed from the registry. |
| `target_firing_per_burst` | f32 | Set-point in `[0,1]`. |
| `rate_ema_tau_bursts` | f32 | EMA time constant in bursts; finite and > 0. |
| `gain` | f32 | Maps rate error to leak change; finite and >= 0. |
| `leak_min`, `leak_max` | f32 | Clamps; each finite and in `[0,1]`, `leak_min <= leak_max`. |
| `update_every_n_bursts` | u32 | Default 1. |

## Validation

`RateModulatedLeakConfig::validate(&Value) -> Result<Option<Config>, String>` is the single check. A disabled config (`enabled` false or absent) is `Ok(None)`. An enabled config is rejected, naming the field, when: `rate_ema_tau_bursts` is not finite or <= 0; `gain` is not finite or < 0; `target_firing_per_burst`, `leak_min` or `leak_max` is outside `[0,1]` or not finite; `leak_min > leak_max`; or the JSON is not an object / a field has the wrong type. `parse_from_cortical_property` uses it and returns `None` on any `Err`. The cortical-area update endpoint calls it first and returns `InvalidInput` before anything is applied (`null` is accepted and clears the setting).

## Leak edits, delete and import

- **Leak edit:** a bulk leak update for an area (`update_cortical_area_leak`, i.e. the `leak_coefficient` parameter via the API, genome apply or parameter queue) reseeds the homeostat: every neuron's internal leak is set to the new value (clamped to `[0,1]`, rate EMA kept) so the edit is the starting point and is not overwritten on the next burst. The next leak step still clamps to `[leak_min, leak_max]`.
- **Area delete:** `ConnectomeManager::remove_cortical_area` removes the area from the registry and from the modulator engine's homeostatic-leak set, so stale neuron indices never apply to another area. Disabling the setting does the same.
- **NPU reset / full connectome import:** `reset_for_new_genome` (run by `apply_connectome_snapshot`) clears the registry, because it holds neuron storage indices. After a full (non-lite) `import_connectome`, `ConnectomeManager::reregister_rate_modulated_leak_for_all_areas` re-registers every area with an enabled `rate_modulated_leak`, seeding from the area's `leak_coefficient` (the homeostat state, not the snapshot's leak values, restarts). Lite import rebuilds neurons from the genome and registers them on creation.

## API and Brain Visualizer

- Updates use the key `rate_modulated_leak` in `PUT` `/v1/cortical_area/cortical_area` with a JSON object (classifier: **parameter** change, no synapse rebuild).
- BV exposes the fields under **Cortical area details** → **Advanced** (the existing Advanced / Danger section).

## See also

- LIF: `neural/src/models/lif.rs`
- Plasticity: `feagi-npu-burst-engine` STDP path is independent of this homeostat.

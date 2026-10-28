# Rate-modulated leak (intrinsic firing-rate homeostasis)

## Purpose

For opt-in **custom** (dense LIF) cortical areas, FEAGI can run a **cold** post-burst pass that updates each neuron’s **effective** `leak_coefficient` from a low-passed estimate of its **firing rate**, producing negative feedback: sustained high activity increases leak and reduces excitability; low activity has the opposite effect. This is **not** synaptic homeostatic plasticity; it does not change synapse weights.

## Hot path vs cold path

- The **LIF** integrate step only **reads** the per-neuron leak value already in storage, unchanged.
- **Leak formula (all neurons, homeostat or not):** per burst `V_t = V_{t-1} * (1 - g) + I_t`, where `g` is the per-neuron `leak_coefficient` (fraction of V lost per burst). Neurons are only touched when they receive input, so the leak is applied lazily: each neuron stores `t_last`, the burst of its last update, and at burst `t` its charge is `V * (1 - g)^(t - t_last) + I_t` before the threshold test (fire resets `V` to 0). Readers (neuron state endpoint, property reads, connectome export) apply the same decay to the current burst without mutating state. Idle neurons therefore decay exactly as if they had been updated every burst, except that in `area` mode the *current* `g` is used for the whole idle stretch (the homeostat changes `g` only for the next burst, so a long idle stretch is decayed with the latest value). In `per_neuron` mode the idle stretch is made exact: before the homeostat changes a neuron's `g`, the burst engine catches its charge up with the previous `g` (`V <- V * (1 - g_old)^(t - t_last)`, `t_last <- t`), so each stretch decays with the leak that was in force.
- **Rate EMA:** unchanged. The EMA is already sampled for every neuron of a registered area on every burst (a neuron that does not fire contributes 0, whether or not it received input), so idle neurons' rate decays and the leak step sees the true rate.
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

## Target modes

`target_mode` selects where each neuron's set-point comes from.

- **`"area"` (default, unchanged):** one `target_firing_per_burst` for every neuron. Right for populations whose neurons share one natural rate. In a population code (a few neurons active, the rest silent) it drives active neurons down to the area mean and pins silent ones at `leak_min`.
- **`"per_neuron"`:** each neuron is held at its own equilibrium.
  1. **Arming** (`arm_bursts`, default 500, after enable or re-arm): `r_arm <- (1-α_a) r_arm + α_a * fired`, `α_a = 1 - exp(-1/τ_a)`, `τ_a = arm_tau_bursts` (default `arm_bursts / 4`). The leak is not adjusted; every neuron runs at the area's base leak. At the end the target is the bias-corrected estimate `target = r_arm / (1 - (1-α_a)^arm_bursts)`.
  2. **Freeze:** targets stop changing. Each regulated neuron then follows `g <- clamp(g + gain * (r_ema - target_j), leak_min, leak_max)` with the usual cadence, gain and clamps (`r_ema` is the fast EMA of `rate_ema_tau_bursts`, which also runs during arming).
  3. **Silent-armed neurons** (`target_j < min_target_firing_per_burst`, default 0.001), selected by `silent_neurons`:
     - `"one_sided"` (default): regulated with target 0. While `r_ema >= min_target_firing_per_burst`: `g <- clamp(g + gain * r_ema, max(base, leak_min), leak_max)` (error `r_ema - 0 >= 0`, so the leak only rises). While `r_ema < min_target_firing_per_burst` (rate ~0) the same step becomes an explicit relaxation toward base at the same gain: `g <- clamp(g - gain * (g - base), max(base, leak_min), leak_max)`. The leak never drops below the area's base leak (`base` clamped into `[leak_min, leak_max]`). This protects against a disturbance that recruits neurons that were silent during arming.
     - `"unregulated"` (legacy): the neuron is not regulated; its leak stays at the area's base leak.
  4. **Re-arm:** changing `arm_epoch` (or `arm_bursts`, the neuron set, or the mode) in the area parameters restarts arming: leaks return to the base leak and targets are re-measured. Re-sending the config with the same `arm_epoch` keeps the armed targets and only updates parameters (gain, clamps, tau). A reseed from a bulk leak edit changes the base leak but keeps targets.
  Arming is counted in bursts since registration, so a brain that is not driven during arming arms to silence; arm while the brain is in the regime to be held.

`target_firing_per_burst` is ignored in `per_neuron` mode.

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
| `target_mode` | string | `"area"` (default) or `"per_neuron"`. |
| `arm_bursts` | u32 | `per_neuron`: arming length in bursts; default 500, >= 1. |
| `arm_tau_bursts` | f32 | `per_neuron`: arming EMA time constant; finite and > 0; default `arm_bursts / 4`. |
| `min_target_firing_per_burst` | f32 | `per_neuron`: neurons with a lower armed target are not regulated; in `[0,1]`, default 0.001. |
| `arm_epoch` | u32 | `per_neuron`: change it to re-arm; default 0. |
| `silent_neurons` | string | `per_neuron`: `"one_sided"` (default) or `"unregulated"`; handling of neurons armed below `min_target_firing_per_burst`. Changing it with the same `arm_epoch` keeps the armed targets and reclassifies the neurons. |

## Validation

`RateModulatedLeakConfig::validate(&Value) -> Result<Option<Config>, String>` is the single check. A disabled config (`enabled` false or absent) is `Ok(None)`. An enabled config is rejected, naming the field, when: `rate_ema_tau_bursts` is not finite or <= 0; `gain` is not finite or < 0; `target_firing_per_burst`, `leak_min` or `leak_max` is outside `[0,1]` or not finite; `leak_min > leak_max`; `target_mode` is not `"area"` or `"per_neuron"`; `arm_bursts` is 0; `arm_tau_bursts` is not finite or <= 0; `min_target_firing_per_burst` is outside `[0,1]`; `silent_neurons` is not `"one_sided"` or `"unregulated"`; or the JSON is not an object / a field has the wrong type. `parse_from_cortical_property` uses it and returns `None` on any `Err`. The cortical-area update endpoint calls it first and returns `InvalidInput` before anything is applied (`null` is accepted and clears the setting).

## Leak edits, delete and import

- **Leak edit:** a bulk leak update for an area (`update_cortical_area_leak`, i.e. the `leak_coefficient` parameter via the API, genome apply or parameter queue) reseeds the homeostat: every neuron's internal leak is set to the new value (clamped to `[0,1]`, rate EMA kept) so the edit is the starting point and is not overwritten on the next burst. The next leak step still clamps to `[leak_min, leak_max]`.
- **Area delete:** `ConnectomeManager::remove_cortical_area` removes the area from the registry and from the modulator engine's homeostatic-leak set, so stale neuron indices never apply to another area. Disabling the setting does the same.
- **NPU reset / full connectome import:** `reset_for_new_genome` (run by `apply_connectome_snapshot`) clears the registry, because it holds neuron storage indices. After a full (non-lite) `import_connectome`, `ConnectomeManager::reregister_rate_modulated_leak_for_all_areas` re-registers every area with an enabled `rate_modulated_leak`, seeding from the area's `leak_coefficient` (the homeostat state, not the snapshot's leak values, restarts). Lite import rebuilds neurons from the genome and registers them on creation.

## API and Brain Visualizer

- Updates use the key `rate_modulated_leak` in `PUT` `/v1/cortical_area/cortical_area` with a JSON object (classifier: **parameter** change, no synapse rebuild).
- Read the live homeostat with the generic per-neuron state endpoint, `GET /v1/connectome/cortical_area/<cortical_id>/neuron_state?fields=leak_coefficient,rate_ema`. The response is columnar: `{cortical_id, count, x[], y[], z[], fields:{leak_coefficient:[..], rate_ema:[..]}}`. `leak_coefficient` is the value the LIF step uses (the homeostat writes its leak into neuron storage), `rate_ema` is the firing-rate EMA in fraction of bursts and is `null` for neurons of an area that is not registered. `rate_target` is the set-point held now (the area target; in `per_neuron` mode the frozen target, or the running estimate while arming); `homeostat_regulated` is 1.0 when the leak is adjusted two-sided (always in `area` mode; `per_neuron` mode after arming for neurons with an armed target), 0.5 when it is regulated one-sided (silent-armed neuron, `silent_neurons: "one_sided"`; `rate_target` reads 0), and 0.0 when not regulated (arming, or silent-armed with `silent_neurons: "unregulated"`). Backward compatible: a reader that tests `> 0` sees "regulated"; a reader that tests `== 1.0` sees the legacy meaning for every neuron except the one-sided ones. Both are `null` for an unregistered area. Studio's run-data recorder passes field names through unchanged, so `rate_target` and `homeostat_regulated` can be requested by name. It is a read-only snapshot, not a stream. An unknown area returns 404; an unknown field returns 400 listing the valid ones.
- BV exposes the fields under **Cortical area details** → **Advanced** (the existing Advanced / Danger section).

## See also

- LIF: `neural/src/models/lif.rs`
- Plasticity: `feagi-npu-burst-engine` STDP path is independent of this homeostat.

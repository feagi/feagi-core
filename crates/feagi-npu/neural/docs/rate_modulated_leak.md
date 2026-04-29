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
4. **Cadence** `update_every_n_bursts`: the whole area update (including EMA) runs only when `burst_count % n == 0` (0 treated as 1).

Neurons are skipped when their storage index is above the memory neuron band or invalid.

## Parameters (JSON)

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | bool | If false, registration is a no-op and the area is removed from the registry. |
| `target_firing_per_burst` | f32 | Set-point in `[0,1]`. |
| `rate_ema_tau_bursts` | f32 | EMA time constant in bursts; must be > 0. |
| `gain` | f32 | Maps rate error to leak change. |
| `leak_min`, `leak_max` | f32 | Clamps; typically within `[0,1]`. |
| `update_every_n_bursts` | u32 | Default 1. |

## API and Brain Visualizer

- Updates use the key `rate_modulated_leak` in `PUT` `/v1/cortical_area/cortical_area` with a JSON object (classifier: **parameter** change, no synapse rebuild).
- BV exposes the fields under **Cortical area details** → **Advanced** (the existing Advanced / Danger section).

## See also

- LIF: `neural/src/models/lif.rs`
- Plasticity: `feagi-npu-burst-engine` STDP path is independent of this homeostat.

// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Neural Dynamics (Phase 2)
//!
//! SIMD-optimized membrane potential updates, threshold checks, and firing logic.
//!
//! ## Performance Critical Path
//! This is the hottest code path in FEAGI. Every optimization matters.
//!
//! ## @npu-debug-instrumentation (cleanup)
//! Temporary runtime wiring (`set_runtime_trace_cortical_idx`, extra trace targets): remove or
//! narrow after membrane/FCL root-cause is finished.
//!
//! ## Optimization Strategy
//! 1. **SIMD**: Process 8+ neurons at once using AVX2/AVX-512
//! 2. **Rayon**: Parallelize across cores for large neuron counts
//! 3. **Cache-friendly**: Sequential access patterns, no pointer chasing

use crate::fire_structures::{FireQueue, FiringNeuron, FIRE_KIND_STDP_ELIGIBLE};
use crate::sparse_memory_lif::{
    resolve_memory_neuron_output, MemoryAssociativeLifParamsByArea,
    SparseMemoryAssociativeLifStates,
};
use ahash::AHashSet;
use feagi_npu_neural::types::*;
use feagi_npu_runtime::NeuronStorage;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Once, OnceLock};
use tracing::debug;

// Use platform-agnostic core algorithms (Phase 1 - NO DUPLICATION)
use feagi_npu_neural::{decayed_charge, excitability_random, update_neurons_lif_batch};

// SIMD support (architecture-agnostic)
// Note: Using LLVM auto-vectorization for now (architecture-agnostic)
// Future: Can use portable_simd crate for explicit SIMD operations if needed

// NOTE: Encoding helpers removed - data structure now uses MAX encoding directly
// threshold_limit: T::MAX = no limit (SIMD-friendly)
// consecutive_fire_limit: u16::MAX = no limit (SIMD-friendly)

/// Runtime-gated tracing config for neural dynamics.
/// Enable with:
/// - `FEAGI_NPU_TRACE_DYNAMICS=1`
///
/// **Anti-spam:** at least one of the following must be set or no per-neuron traces are emitted:
/// - `FEAGI_NPU_TRACE_NEURON=<u32>` — single neuron id
/// - `FEAGI_NPU_TRACE_CORTICAL_IDX=<u32>` — numeric `cortical_idx` (same as neuron array / BDU assignment)
///
/// Power area (`cortical_idx == 1`) is suppressed unless `FEAGI_NPU_TRACE_CORTICAL_IDX=1`.
struct DynamicsTraceCfg {
    enabled: bool,
    neuron_filter: Option<u32>,
    cortical_idx_filter: Option<u32>,
}

/// Sentinel: no runtime override for cortical_idx tracing (see `set_runtime_trace_cortical_idx`).
const RUNTIME_TRACE_CORTICAL_IDX_UNSET: u32 = u32::MAX;

static RUNTIME_TRACE_CORTICAL_IDX: AtomicU32 = AtomicU32::new(RUNTIME_TRACE_CORTICAL_IDX_UNSET);

/// Set after brain load so dynamics/FCL use the same cortical_idx as `FEAGI_NPU_TRACE_CORTICAL_ID`.
/// Env `FEAGI_NPU_TRACE_CORTICAL_IDX` **wins** when set (explicit).
pub fn set_runtime_trace_cortical_idx(idx: Option<u32>) {
    match idx {
        Some(i) => RUNTIME_TRACE_CORTICAL_IDX.store(i, Ordering::Release),
        None => {
            RUNTIME_TRACE_CORTICAL_IDX.store(RUNTIME_TRACE_CORTICAL_IDX_UNSET, Ordering::Release)
        }
    }
}

fn runtime_trace_cortical_idx() -> Option<u32> {
    let v = RUNTIME_TRACE_CORTICAL_IDX.load(Ordering::Acquire);
    if v == RUNTIME_TRACE_CORTICAL_IDX_UNSET {
        None
    } else {
        Some(v)
    }
}

fn effective_cortical_idx_filter() -> Option<u32> {
    let c = dynamics_trace_cfg();
    c.cortical_idx_filter.or_else(runtime_trace_cortical_idx)
}

fn dynamics_trace_cfg() -> &'static DynamicsTraceCfg {
    static CFG: OnceLock<DynamicsTraceCfg> = OnceLock::new();
    CFG.get_or_init(|| {
        let enabled = std::env::var("FEAGI_NPU_TRACE_DYNAMICS")
            .ok()
            .as_deref()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let neuron_filter = std::env::var("FEAGI_NPU_TRACE_NEURON")
            .ok()
            .and_then(|v| v.parse().ok());

        let cortical_idx_filter = std::env::var("FEAGI_NPU_TRACE_CORTICAL_IDX")
            .ok()
            .and_then(|v| v.parse().ok());

        let cfg = DynamicsTraceCfg {
            enabled,
            neuron_filter,
            cortical_idx_filter,
        };

        let cortical_id_for_late_idx = std::env::var("FEAGI_NPU_TRACE_CORTICAL_ID")
            .ok()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);

        static WARN_NO_FOCUS: Once = Once::new();
        if cfg.enabled
            && cfg.neuron_filter.is_none()
            && cfg.cortical_idx_filter.is_none()
            && !cortical_id_for_late_idx
        {
            WARN_NO_FOCUS.call_once(|| {
                tracing::warn!(
                    target: "feagi-npu-trace",
                    "FEAGI_NPU_TRACE_DYNAMICS is set but neither FEAGI_NPU_TRACE_NEURON nor FEAGI_NPU_TRACE_CORTICAL_IDX is set — per-neuron dynamics traces are disabled (anti-spam). Set one of those to scope logging."
                );
            });
        }

        cfg
    })
}

/// Returns true when per-neuron dynamics tracing should emit for this neuron.
/// Power neurons are skipped unless `FEAGI_NPU_TRACE_CORTICAL_IDX=1`.
#[inline]
fn dynamics_trace_emit(neuron_id: u32, cortical_idx: u32) -> bool {
    let cfg = dynamics_trace_cfg();
    if !cfg.enabled {
        return false;
    }
    let idx = effective_cortical_idx_filter();
    let has_focus = cfg.neuron_filter.is_some() || idx.is_some();
    if !has_focus {
        return false;
    }
    if cortical_idx == 1 && idx != Some(1) {
        return false;
    }
    let neuron_ok = cfg.neuron_filter.map(|n| n == neuron_id).unwrap_or(true);
    let cortical_ok = idx.map(|c| c == cortical_idx).unwrap_or(true);
    neuron_ok && cortical_ok
}

/// When set, [`process_burst`](crate::RustNPU::process_burst) logs one FCL aggregate line for this
/// cortical index (phase 1, pre-dynamics) when dynamics tracing is enabled.
pub fn trace_fcl_cortical_idx_for_logging() -> Option<u32> {
    let cfg = dynamics_trace_cfg();
    if !cfg.enabled {
        return None;
    }
    effective_cortical_idx_filter()
}

/// Result of neural dynamics processing
#[derive(Debug, Clone)]
pub struct DynamicsResult {
    /// Neurons that fired this burst
    pub fire_queue: FireQueue,

    /// Performance metrics
    pub neurons_processed: usize,
    pub neurons_fired: usize,
    pub neurons_in_refractory: usize,
}

/// Option B refractory: tick countdown once per simulation burst for **all** neurons.
///
/// Call at **end** of each burst after [`process_neural_dynamics`] (and after merging any
/// authoritative fires into the fire queue). Neurons that **fired** this burst must appear in
/// `fired_neuron_ids` so their freshly assigned refractory countdown is not decremented in the
/// same burst.
///
/// Semantics:
/// - Every neuron with `refractory_countdown > 0` that **did not** fire this burst has its
///   countdown decremented by one (membrane unchanged here; integration freeze happens in
///   dynamics).
/// - When countdown reaches zero after decrement, extended-refractory consecutive-fire reset runs
///   (same rules as the former inline path in [`process_single_neuron`]).
pub fn finish_burst_refractory_period<T: NeuralValue>(
    neuron_array: &mut impl NeuronStorage<Value = T>,
    fired_neuron_ids: &AHashSet<u32>,
) {
    let n = neuron_array.count();
    for idx in 0..n {
        if !neuron_array.valid_mask()[idx] {
            continue;
        }
        let id = idx as u32;
        if fired_neuron_ids.contains(&id) {
            continue;
        }
        let cd = neuron_array.refractory_countdowns()[idx];
        if cd == 0 {
            continue;
        }
        let new_cd = cd - 1;
        neuron_array.refractory_countdowns_mut()[idx] = new_cd;

        let consecutive_fire_limit_raw = neuron_array.consecutive_fire_limits()[idx];
        let consecutive_fire_limit = if consecutive_fire_limit_raw == 0 {
            u16::MAX
        } else {
            consecutive_fire_limit_raw
        };
        if new_cd == 0
            && consecutive_fire_limit_raw != 0
            && consecutive_fire_limit != u16::MAX
            && neuron_array.consecutive_fire_counts()[idx] >= consecutive_fire_limit
        {
            neuron_array.consecutive_fire_counts_mut()[idx] = 0;
        }
    }
}

/// Membrane charge of neuron `idx` as of `burst`, without mutating storage.
///
/// Neurons are only updated when they receive input, so the stored value is the charge at
/// `last_update_burst`. Readers apply the same lazy leak the burst step uses:
/// `V * (1 - g) ^ (burst - last_update_burst)`.
pub(crate) fn caught_up_membrane_charge<T: NeuralValue>(
    neuron_array: &impl NeuronStorage<Value = T>,
    idx: usize,
    burst: u64,
) -> f32 {
    decayed_charge(
        neuron_array.membrane_potentials()[idx].to_f32() + neuron_array.membrane_fractions()[idx],
        neuron_array.leak_coefficients()[idx],
        burst.saturating_sub(neuron_array.last_update_bursts()[idx]),
    )
}

/// Apply the lazy leak of the idle stretch with `old_leak` and stamp `burst` as the last update.
///
/// Call before changing a neuron's leak coefficient outside the LIF step, so the idle stretch
/// decays with the leak that was in force, not the new one. No-op when already current.
pub(crate) fn catch_up_before_leak_change<T: NeuralValue>(
    neuron_array: &mut impl NeuronStorage<Value = T>,
    idx: usize,
    old_leak: f32,
    burst: u64,
) {
    if idx >= neuron_array.count() {
        return;
    }
    let elapsed = burst.saturating_sub(neuron_array.last_update_bursts()[idx]);
    if elapsed == 0 {
        return;
    }
    let charge =
        neuron_array.membrane_potentials()[idx].to_f32() + neuron_array.membrane_fractions()[idx];
    if charge != 0.0 {
        let decayed = decayed_charge(charge, old_leak, elapsed);
        let (p, f) = T::zero().integrate_charge(decayed, 0.0);
        neuron_array.membrane_potentials_mut()[idx] = p;
        neuron_array.membrane_fractions_mut()[idx] = f;
    }
    neuron_array.last_update_bursts_mut()[idx] = burst;
}

/// Process neural dynamics for all candidate neurons
///
/// This is the CRITICAL HOT PATH - every microsecond matters!
///
/// ## Algorithm (standard per-burst LIF, leak applied lazily):
/// Per burst `V_t = V_{t-1} * (1 - g) + I_t`. Only FCL candidates are touched; each neuron
/// stores the burst of its last update, so the leak of the idle stretch is applied in one step.
/// 1. Catch up leak: `V = V * (1 - g) ^ (burst - last_update_burst)` (non-refractory FCL neurons)
/// 2. Add candidate potentials from FCL (skipped when refractory — Option B freeze)
/// 3. Check firing thresholds (with refractory period); firing resets `V` to 0
/// 4. Apply probabilistic excitability
/// 5. Store `last_update_burst = burst` (fired or not); no post-input leak is applied
/// 6. Create Fire Queue from firing neurons
///
/// Readers between inputs apply the same catch-up without mutating state (see `RustNPU`).
///
/// **Refractory:** After this returns, callers **must** invoke [`finish_burst_refractory_period`]
/// once per burst so countdown advances for **all** neurons, including those not listed in the FCL.
#[allow(clippy::too_many_arguments)] // Hot path: explicit params avoid alloc and match call sites
pub fn process_neural_dynamics<T: NeuralValue>(
    fcl: &FireCandidateList,
    memory_candidate_cortical_idx: Option<&ahash::AHashMap<u32, u32>>,
    memory_candidate_fire_kind: Option<&ahash::AHashMap<u32, u8>>,
    memory_associative_psp: Option<&ahash::AHashMap<u32, f32>>,
    sparse_memory_associative_lif: &mut SparseMemoryAssociativeLifStates,
    memory_associative_lif_params: Option<&MemoryAssociativeLifParamsByArea>,
    neuron_array: &mut impl NeuronStorage<Value = T>,
    burst_count: u64,
) -> Result<DynamicsResult> {
    let profile_enabled = tracing::enabled!(tracing::Level::DEBUG);
    let dynamics_start = profile_enabled.then(std::time::Instant::now);
    let candidate_count = fcl.len();

    if candidate_count == 0 {
        let mut fire_queue = FireQueue::new();
        fire_queue.set_timestep(burst_count);
        return Ok(DynamicsResult {
            fire_queue,
            neurons_processed: 0,
            neurons_fired: 0,
            neurons_in_refractory: 0,
        });
    }

    // NOTE: We CANNOT use Rayon here because process_single_neuron mutates neuron_array
    // This would require unsafe code or a different approach (batch processing)
    // For now, use single-threaded processing (still very fast!)
    // Memory neuron ID range (reserved, not backed by neuron_array indices).
    // NOTE: Keep in sync with `feagi-npu/plasticity/src/neuron_id_manager.rs`.
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;

    let (fired_neurons, refractory_count): (Vec<_>, usize) = fcl.with_cached(|candidates| {
            // For large candidate counts, use batch processing for better cache locality
            // Threshold: 10k candidates (lowered from 50k based on profiling data)
            // Profiling showed sequential path taking 2.67μs per candidate, making SIMD batching
            // worthwhile even at lower counts. The gather/scatter overhead is offset by SIMD speedup.
            const SIMD_BATCH_THRESHOLD: usize = 10_000;

            if candidates.len() >= SIMD_BATCH_THRESHOLD {
                // SIMD batch processing path
                process_candidates_with_simd_batching(
                    candidates,
                    memory_candidate_cortical_idx,
                    memory_candidate_fire_kind,
                    memory_associative_psp,
                    sparse_memory_associative_lif,
                    memory_associative_lif_params,
                    neuron_array,
                    burst_count,
                )
            } else {
                // Sequential processing for small candidate counts (< 10k)
                // Use batch processing in chunks for better cache locality even in sequential path
                let sequential_start = profile_enabled.then(std::time::Instant::now);
                let mut results = Vec::with_capacity(candidates.len());
                let mut refractory = 0;
                let mut memory_fired = 0usize;
                let mut memory_missing_meta = 0usize;
                for &(neuron_id, candidate_potential) in candidates {
                    // Memory neurons are force-fired and do not use the regular neuron storage array.
                    if neuron_id.0 >= MEMORY_NEURON_ID_START {
                        let cortical_idx = match memory_candidate_cortical_idx
                            .and_then(|m| m.get(&neuron_id.0).copied())
                        {
                            Some(idx) => idx,
                            None => {
                                memory_missing_meta += 1;
                                continue; // No metadata for this memory neuron candidate
                            }
                        };

                        let staged = memory_candidate_fire_kind
                            .and_then(|m| m.get(&neuron_id.0).copied());
                        if let Some(n) = resolve_memory_neuron_output(
                            neuron_id.0,
                            cortical_idx,
                            candidate_potential,
                            staged,
                            burst_count,
                            memory_associative_psp,
                            sparse_memory_associative_lif,
                            memory_associative_lif_params,
                        ) {
                            results.push(n);
                            memory_fired += 1;
                        }
                        continue;
                    }

                    // Keep the synaptic current in f32 until it is added to the membrane.
                    if let Some(neuron) = process_single_neuron(
                        neuron_id,
                        candidate_potential,
                        neuron_array,
                        burst_count,
                    ) {
                        results.push(neuron);
                    }

                    // Count refractory neurons (neuron_id == array index)
                    let idx = neuron_id.0 as usize;
                    if idx < neuron_array.count()
                        && neuron_array.refractory_countdowns_mut()[idx] > 0
                    {
                        refractory += 1;
                    }
                }

                if memory_fired > 0 || memory_missing_meta > 0 {
                    tracing::debug!(
                        "[PHASE2] Memory candidates: fired={} missing_meta={}",
                        memory_fired,
                        memory_missing_meta
                    );
                }

                if let Some(sequential_start) = sequential_start {
                    let sequential_duration = sequential_start.elapsed();
                    // Log profiling for sequential path if slow
                    if sequential_duration.as_millis() > 5 || candidate_count > 5_000 {
                        tracing::debug!(
                            "[PHASE2-PROFILE] Sequential processing: total={:.2}ms | candidates={} | fired={} | refractory={}",
                            sequential_duration.as_secs_f64() * 1000.0,
                            candidates.len(),
                            results.len(),
                            refractory
                        );
                    }
                }

                (results, refractory)
            }
    });

    // Build Fire Queue
    let mut fire_queue = FireQueue::new();
    fire_queue.set_timestep(burst_count); // CRITICAL: Set timestep for FQ Sampler deduplication
    for neuron in fired_neurons.iter() {
        fire_queue.add_neuron(neuron.clone());
    }

    if let Some(dynamics_start) = dynamics_start {
        let dynamics_duration = dynamics_start.elapsed();
        // Log dynamics processing timing only for extreme cases (>30 seconds)
        // Batch processing with millions of candidates can legitimately take 5-10 seconds
        if dynamics_duration.as_millis() > 30000 {
            tracing::warn!(
                "[PHASE2-DYNAMICS] Very slow dynamics processing: {:.2}ms for {} candidates, {} fired",
                dynamics_duration.as_secs_f64() * 1000.0,
                candidate_count,
                fired_neurons.len()
            );
        } else if candidate_count > 1_000_000 {
            // Info log for large batch processing
            tracing::info!(
                "[PHASE2-DYNAMICS] Batch processing: {:.2}ms for {} candidates, {} fired",
                dynamics_duration.as_secs_f64() * 1000.0,
                candidate_count,
                fired_neurons.len()
            );
        }
    }

    Ok(DynamicsResult {
        fire_queue,
        neurons_processed: candidate_count,
        neurons_fired: fired_neurons.len(),
        neurons_in_refractory: refractory_count,
    })
}

/// Process candidates with SIMD batch processing (gather/scatter pattern)
///
/// For large candidate counts, this function:
/// 1. Separates candidates into SIMD-eligible (not refractory, simple constraints) and sequential
/// 2. Gathers SIMD-eligible candidates into contiguous arrays
/// 3. Uses `update_neurons_lif_batch` for basic LIF operations (SIMD-optimized)
/// 4. Handles complex state (threshold_limit, consecutive_fire_limit, excitability) sequentially
/// 5. Scatters results back to sparse locations
///
/// This maintains 100% correctness while achieving 3-6x speedup for large candidate counts.
#[allow(clippy::too_many_arguments)] // Mirrors `process_neural_dynamics` for SIMD batch path
fn process_candidates_with_simd_batching<T: NeuralValue>(
    candidates: &[(NeuronId, f32)],
    memory_candidate_cortical_idx: Option<&ahash::AHashMap<u32, u32>>,
    memory_candidate_fire_kind: Option<&ahash::AHashMap<u32, u8>>,
    memory_associative_psp: Option<&ahash::AHashMap<u32, f32>>,
    sparse_memory_associative_lif: &mut SparseMemoryAssociativeLifStates,
    memory_associative_lif_params: Option<&MemoryAssociativeLifParamsByArea>,
    neuron_array: &mut impl NeuronStorage<Value = T>,
    burst_count: u64,
) -> (Vec<FiringNeuron>, usize) {
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;
    const SIMD_BATCH_SIZE: usize = 10_000; // Process in chunks for cache locality and SIMD efficiency

    let profile_enabled = tracing::enabled!(tracing::Level::DEBUG);
    let total_start = profile_enabled.then(std::time::Instant::now);
    let mut results = Vec::with_capacity(candidates.len());
    let mut refractory = 0;
    let mut memory_fired = 0usize;
    let mut memory_missing_meta = 0usize;

    // Separate candidates into categories
    let separate_start = profile_enabled.then(std::time::Instant::now);
    let mut memory_candidates = Vec::new();
    let mut simd_eligible = Vec::new(); // Not in refractory, can use SIMD for basic ops
    let mut sequential_only = Vec::new(); // In refractory or have complex constraints

    for &(neuron_id, candidate_potential) in candidates {
        // Memory neurons are force-fired
        if neuron_id.0 >= MEMORY_NEURON_ID_START {
            memory_candidates.push((neuron_id, candidate_potential));
            continue;
        }

        let idx = neuron_id.0 as usize;
        if idx >= neuron_array.count() || !neuron_array.valid_mask()[idx] {
            continue; // Invalid neuron
        }

        // Check if in refractory (must process sequentially to decrement countdown)
        if neuron_array.refractory_countdowns()[idx] > 0 {
            sequential_only.push((neuron_id, candidate_potential));
            refractory += 1;
            continue;
        }

        // For Approach 6 + Approach 2: Process ALL candidates with SIMD masks
        // We'll use SIMD masks to handle constraints, so we can process more candidates with SIMD
        // Only truly sequential cases (refractory) go to sequential_only
        simd_eligible.push((neuron_id, candidate_potential));
    }
    let separate_duration = separate_start.map(|start| start.elapsed());

    // Capture lengths before vectors are moved in loops
    let memory_candidates_len = memory_candidates.len();
    let sequential_only_len = sequential_only.len();
    let simd_eligible_len = simd_eligible.len();

    // Handle memory neurons (force-fired)
    let memory_start = profile_enabled.then(std::time::Instant::now);
    for (neuron_id, candidate_potential) in memory_candidates {
        let cortical_idx =
            match memory_candidate_cortical_idx.and_then(|m| m.get(&neuron_id.0).copied()) {
                Some(idx) => idx,
                None => {
                    memory_missing_meta += 1;
                    continue;
                }
            };

        let staged = memory_candidate_fire_kind.and_then(|m| m.get(&neuron_id.0).copied());
        if let Some(n) = resolve_memory_neuron_output(
            neuron_id.0,
            cortical_idx,
            candidate_potential,
            staged,
            burst_count,
            memory_associative_psp,
            sparse_memory_associative_lif,
            memory_associative_lif_params,
        ) {
            results.push(n);
            memory_fired += 1;
        }
    }
    if memory_fired > 0 || memory_missing_meta > 0 {
        tracing::debug!(
            "[PHASE2] Memory candidates: fired={} missing_meta={}",
            memory_fired,
            memory_missing_meta
        );
    }
    let memory_duration = memory_start.map(|start| start.elapsed());

    // Process SIMD-eligible candidates in batches with SIMD mask-based constraint handling
    let mut total_gather_time = std::time::Duration::ZERO;
    let mut total_lif_time = std::time::Duration::ZERO;
    let mut total_constraint_time = std::time::Duration::ZERO;
    let mut total_scatter_time = std::time::Duration::ZERO;
    let mut batch_count = 0;

    for batch in simd_eligible.chunks(SIMD_BATCH_SIZE) {
        batch_count += 1;
        let batch_size = batch.len();

        // Gather: Collect data into contiguous arrays (including constraint data)
        let gather_start = profile_enabled.then(std::time::Instant::now);
        let mut batch_mp = Vec::with_capacity(batch_size);
        let mut batch_mp_fractions = Vec::with_capacity(batch_size);
        let mut batch_thresholds = Vec::with_capacity(batch_size);
        let mut batch_threshold_fractions = Vec::with_capacity(batch_size);
        let mut batch_threshold_limits = Vec::with_capacity(batch_size);
        let mut batch_leaks = Vec::with_capacity(batch_size);
        let mut batch_elapsed: Vec<u64> = Vec::with_capacity(batch_size);
        let mut batch_candidates = Vec::with_capacity(batch_size);
        let mut batch_consecutive_fire_counts = Vec::with_capacity(batch_size);
        let mut batch_consecutive_fire_limits = Vec::with_capacity(batch_size);
        let mut batch_excitabilities = Vec::with_capacity(batch_size);
        let mut batch_indices = Vec::with_capacity(batch_size); // Store original indices for scatter

        for &(neuron_id, candidate_potential) in batch {
            let idx = neuron_id.0 as usize;
            batch_mp.push(neuron_array.membrane_potentials()[idx]);
            batch_mp_fractions.push(neuron_array.membrane_fractions()[idx]);
            batch_thresholds.push(neuron_array.thresholds()[idx]);
            batch_threshold_fractions.push(neuron_array.threshold_fractions()[idx]);
            batch_threshold_limits.push(neuron_array.threshold_limits()[idx]);
            batch_leaks.push(neuron_array.leak_coefficients()[idx]);
            batch_elapsed.push(burst_count.saturating_sub(neuron_array.last_update_bursts()[idx]));
            let mut cand = candidate_potential;
            if neuron_array.mp_charge_accumulation()[idx] && cand < 0.0 {
                cand = 0.0;
            }
            batch_candidates.push(cand);
            batch_consecutive_fire_counts.push(neuron_array.consecutive_fire_counts()[idx]);
            batch_consecutive_fire_limits.push(neuron_array.consecutive_fire_limits()[idx]);
            batch_excitabilities.push(neuron_array.excitabilities()[idx]);
            batch_indices.push((neuron_id, idx));
        }
        if let Some(gather_start) = gather_start {
            total_gather_time += gather_start.elapsed();
        }

        // Process: Use SIMD batch function for basic LIF operations
        let lif_start = profile_enabled.then(std::time::Instant::now);
        // IMPORTANT: `update_neurons_lif_batch` resets membrane potential to 0 for fired neurons
        // (matching LIF behavior). For downstream features (e.g., mp_driven_psp) and visualization,
        // we must preserve the *pre-reset* membrane potential at the moment of firing.
        // Lazy leak: the stored value is as of `last_update_burst`, so the pre-update charge
        // used for firing-time reporting and constraint checks is the caught-up charge.
        let batch_charge_decayed: Vec<f32> = (0..batch_size)
            .map(|i| {
                decayed_charge(
                    batch_mp[i].to_f32() + batch_mp_fractions[i],
                    batch_leaks[i],
                    batch_elapsed[i],
                )
            })
            .collect();
        let mut fired_mask = vec![false; batch_size];
        update_neurons_lif_batch(
            &mut batch_mp,
            &mut batch_mp_fractions,
            &batch_thresholds,
            &batch_threshold_fractions,
            &batch_leaks,
            &batch_candidates,
            &batch_elapsed,
            &mut fired_mask,
        );
        if let Some(lif_start) = lif_start {
            total_lif_time += lif_start.elapsed();
        }

        // Apply constraint masks: threshold_limit, consecutive_fire_limit, excitability
        // Approach 2: Batch constraint checks for better SIMD auto-vectorization
        // LLVM can auto-vectorize these loops if we structure them correctly
        let constraint_start = profile_enabled.then(std::time::Instant::now);
        for i in 0..batch_size {
            if !fired_mask[i] {
                continue; // Already determined not to fire by basic LIF
            }

            let &(neuron_id, _idx) = &batch_indices[i];

            // Check threshold_limit against MP at threshold-crossing time (before reset).
            // `update_neurons_lif_batch` resets MP to 0 for firing candidates, so we must
            // evaluate constraints using pre-reset MP to preserve scalar-path semantics.
            let threshold_limit = batch_threshold_limits[i];
            let charge_before_reset = batch_charge_decayed[i] + batch_candidates[i];
            // threshold_limit == T::MAX means no upper bound.
            let limit_open = threshold_limit.to_f32() == T::max_value().to_f32();
            if !limit_open && charge_before_reset > threshold_limit.to_f32() {
                fired_mask[i] = false; // Blocked by threshold_limit
                                       // Match scalar path: blocked firings keep the caught-up charge (no post-input leak).
                let (level, fraction) = T::split_charge(charge_before_reset);
                batch_mp[i] = level;
                batch_mp_fractions[i] = fraction;
                continue;
            }

            // Check consecutive_fire_limit (SIMD-friendly: u16::MAX = no limit, direct comparison)
            // Semantics: 0 disables the limiter (unlimited).
            // Internally, most paths use u16::MAX as the unlimited encoding; support both.
            let consecutive_fire_limit_raw = batch_consecutive_fire_limits[i];
            let consecutive_fire_limit = if consecutive_fire_limit_raw == 0 {
                u16::MAX
            } else {
                consecutive_fire_limit_raw
            };
            let consecutive_fire_count = batch_consecutive_fire_counts[i];
            // SIMD-friendly: uniform comparison (no branching for "no limit" case)
            // If consecutive_fire_limit == u16::MAX, count will always be < MAX
            if consecutive_fire_count >= consecutive_fire_limit {
                fired_mask[i] = false; // Blocked by consecutive_fire_limit
                                       // Match scalar path: blocked firings keep the caught-up charge (no post-input leak).
                let (level, fraction) = T::split_charge(charge_before_reset);
                batch_mp[i] = level;
                batch_mp_fractions[i] = fraction;
                continue;
            }

            // Check excitability (probabilistic) - must be sequential due to random generation
            let excitability = batch_excitabilities[i];
            if excitability < 0.999 {
                if excitability <= 0.0 {
                    fired_mask[i] = false; // Never fire
                                           // Match scalar path: blocked firings keep the caught-up charge (no post-input leak).
                    let (level, fraction) = T::split_charge(charge_before_reset);
                    batch_mp[i] = level;
                    batch_mp_fractions[i] = fraction;
                    continue;
                }
                // Probabilistic: use deterministic random (needs neuron_id, so sequential)
                let random_val = excitability_random(neuron_id.0, burst_count);
                if random_val >= excitability {
                    fired_mask[i] = false; // Blocked by excitability
                                           // Match scalar path: blocked firings keep the caught-up charge (no post-input leak).
                    let (level, fraction) = T::split_charge(charge_before_reset);
                    batch_mp[i] = level;
                    batch_mp_fractions[i] = fraction;
                    continue;
                }
            }
        }
        if let Some(constraint_start) = constraint_start {
            total_constraint_time += constraint_start.elapsed();
        }

        // Scatter: Write results back and handle firing
        let scatter_start = profile_enabled.then(std::time::Instant::now);
        for (i, (neuron_id, idx)) in batch_indices.iter().enumerate() {
            // Update membrane potential
            neuron_array.membrane_potentials_mut()[*idx] = batch_mp[i];
            neuron_array.membrane_fractions_mut()[*idx] = batch_mp_fractions[i];
            neuron_array.last_update_bursts_mut()[*idx] = burst_count;

            if fired_mask[i] {
                // Neuron fired - handle firing logic
                let cortical_idx = neuron_array.cortical_areas()[*idx];
                let refractory_period = neuron_array.refractory_periods()[*idx];

                // Set refractory countdown
                neuron_array.refractory_countdowns_mut()[*idx] = refractory_period;

                // Increment consecutive fire count
                let old_count = neuron_array.consecutive_fire_counts()[*idx];
                neuron_array.consecutive_fire_counts_mut()[*idx] = old_count.saturating_add(1);

                // Get coordinates
                let coord_idx = *idx * 3;
                let (x, y, z) = (
                    neuron_array.coordinates()[coord_idx],
                    neuron_array.coordinates()[coord_idx + 1],
                    neuron_array.coordinates()[coord_idx + 2],
                );

                // Capture firing-time membrane potential BEFORE reset.
                // LIF semantics: MP_fire = MP_old * (1-g)^elapsed + candidate.
                let mp_at_fire = batch_charge_decayed[i] + batch_candidates[i];

                results.push(FiringNeuron {
                    neuron_id: *neuron_id,
                    membrane_potential: mp_at_fire,
                    cortical_idx,
                    x,
                    y,
                    z,
                    fire_kind: FIRE_KIND_STDP_ELIGIBLE,
                });
            } else {
                // Neuron didn't fire - reset consecutive fire count if needed
                let consecutive_fire_limit = batch_consecutive_fire_limits[i];
                if consecutive_fire_limit != 0 && consecutive_fire_limit != u16::MAX {
                    neuron_array.consecutive_fire_counts_mut()[*idx] = 0;
                }
            }
            // If not fired, the caught-up charge was stored (no post-input leak).
        }
        if let Some(scatter_start) = scatter_start {
            total_scatter_time += scatter_start.elapsed();
        }
    }

    // Process sequential-only candidates (refractory, complex constraints)
    let sequential_start = profile_enabled.then(std::time::Instant::now);
    for (neuron_id, candidate_potential) in sequential_only {
        if let Some(neuron) =
            process_single_neuron(neuron_id, candidate_potential, neuron_array, burst_count)
        {
            results.push(neuron);
        }
    }
    let sequential_duration = sequential_start.map(|start| start.elapsed());
    let total_duration = total_start.map(|start| start.elapsed());

    // Log detailed profiling if processing is slow or periodically.
    // Debug level by default to avoid log spam in normal operation.
    if let Some(total_duration) = total_duration {
        if total_duration.as_millis() > 10 || candidates.len() > 30_000 {
            tracing::debug!(
                "[PHASE2-PROFILE] SIMD batching breakdown: total={:.2}ms | separate={:.2}ms | memory={:.2}ms | batches={} | gather={:.2}ms | lif={:.2}ms | constraint={:.2}ms | scatter={:.2}ms | sequential={:.2}ms | candidates={} | simd_eligible={} | sequential_only={} | memory={}",
                total_duration.as_secs_f64() * 1000.0,
                separate_duration.unwrap_or_default().as_secs_f64() * 1000.0,
                memory_duration.unwrap_or_default().as_secs_f64() * 1000.0,
                batch_count,
                total_gather_time.as_secs_f64() * 1000.0,
                total_lif_time.as_secs_f64() * 1000.0,
                total_constraint_time.as_secs_f64() * 1000.0,
                total_scatter_time.as_secs_f64() * 1000.0,
                sequential_duration.unwrap_or_default().as_secs_f64() * 1000.0,
                candidates.len(),
                simd_eligible_len,
                sequential_only_len,
                memory_candidates_len
            );
        }
    }

    (results, refractory)
}

/// Process a single neuron's dynamics
///
/// Returns Some(FiringNeuron) if the neuron fires, None otherwise
#[inline(always)]
fn process_single_neuron<T: NeuralValue>(
    neuron_id: NeuronId,
    candidate_potential: f32,
    neuron_array: &mut impl NeuronStorage<Value = T>,
    burst_count: u64,
) -> Option<FiringNeuron> {
    // neuron_id == array index (direct access, no HashMap needed!)
    let idx = neuron_id.0 as usize;

    // Validate index
    if idx >= neuron_array.count() || !neuron_array.valid_mask()[idx] {
        return None; // Neuron doesn't exist
    }

    let cortical_idx = neuron_array.cortical_areas()[idx];
    let allow_trace = dynamics_trace_emit(neuron_id.0, cortical_idx);
    let mp_acc = neuron_array.mp_charge_accumulation()[idx];
    let candidate_potential = if mp_acc && candidate_potential < 0.0 {
        0.0
    } else {
        candidate_potential
    };

    // CRITICAL DEBUG: Log entry for neuron 16438 (disabled to reduce spam)
    // if neuron_id.0 == 16438 {
    //     println!("[RUST-16438] Burst {}: START - countdown={}, count={}/{}, potential={:.6}, threshold={:.6}, candidate={:.6}",
    //              burst_count,
    //              neuron_array.refractory_countdowns_mut()[idx],
    //              neuron_array.consecutive_fire_counts()[idx],
    //              neuron_array.consecutive_fire_limits()[idx],
    //              neuron_array.membrane_potentials()[idx],
    //              neuron_array.thresholds()[idx],
    //              candidate_potential);
    // }

    // 1. Option B refractory: block integration and firing; membrane unchanged this burst.
    // Countdown decrements once per simulation burst in [`finish_burst_refractory_period`], not
    // here (so sparse FCL still advances global refractory time).
    let refractory_countdown = neuron_array.refractory_countdowns()[idx];
    if refractory_countdown > 0 {
        if allow_trace {
            let mp = neuron_array.membrane_potentials()[idx].to_f32();
            let thr = neuron_array.thresholds()[idx].to_f32();
            let leak = neuron_array.leak_coefficients()[idx];
            debug!(
                target: "feagi-npu-trace",
                "[DYN] burst={} neuron={} area={} mp_acc={} REFRACTORY countdown={} candidate={:.6} mp={:.6} thr={:.6} leak={:.6}",
                burst_count,
                neuron_id.0,
                cortical_idx,
                mp_acc,
                refractory_countdown,
                candidate_potential.to_f32(),
                mp,
                thr,
                leak
            );
        }
        return None;
    }

    // 2. Catch up the leak of the idle stretch, then add the candidate potential (add BEFORE
    // checking threshold). V_t = V_last * (1-g)^(t - t_last) + I_t.
    // The fraction keeps a sub-byte PSP and an inhibitory current that a byte cannot hold.
    let old_potential = neuron_array.membrane_potentials()[idx];
    let old_fraction = neuron_array.membrane_fractions()[idx];
    let leak_coefficient = neuron_array.leak_coefficients()[idx];
    let elapsed = burst_count.saturating_sub(neuron_array.last_update_bursts()[idx]);
    let decayed = decayed_charge(
        old_potential.to_f32() + old_fraction,
        leak_coefficient,
        elapsed,
    );
    let (current_potential, current_fraction) =
        T::zero().integrate_charge(decayed, candidate_potential);
    neuron_array.membrane_potentials_mut()[idx] = current_potential;
    neuron_array.membrane_fractions_mut()[idx] = current_fraction;
    neuron_array.last_update_bursts_mut()[idx] = burst_count;
    let charge = current_potential.to_f32() + current_fraction;

    // 3. Check threshold (matches Python: "Check firing conditions BEFORE decay")
    let threshold = neuron_array.thresholds()[idx];
    let threshold_fraction = neuron_array.threshold_fractions()[idx];
    let threshold_limit = neuron_array.threshold_limits()[idx];
    let threshold_charge = threshold.to_f32() + threshold_fraction;

    // Firing window: threshold <= MP <= threshold_limit.
    // threshold_limit == T::MAX means no upper bound.
    let above_min = charge >= threshold_charge;
    let limit_open = threshold_limit.to_f32() == T::max_value().to_f32();
    let below_max = limit_open || charge <= threshold_limit.to_f32();

    if above_min && below_max {
        if allow_trace {
            debug!(
                target: "feagi-npu-trace",
                "[DYN] burst={} neuron={} area={} mp_acc={} CROSS mp_old={:.6} cand={:.6} mp_new={:.6} thr={:.6} thr_limit={:.6} leak={:.6} excit={:.6}",
                burst_count,
                neuron_id.0,
                cortical_idx,
                mp_acc,
                old_potential.to_f32() + old_fraction,
                candidate_potential,
                charge,
                threshold_charge,
                threshold_limit.to_f32(),
                neuron_array.leak_coefficients()[idx],
                neuron_array.excitabilities()[idx]
            );
        }
        // 5. Check consecutive fire limit (matches Python SIMD implementation)
        // SIMD-friendly encoding: consecutive_fire_limit == u16::MAX means unlimited
        // Semantics: 0 disables the limiter (unlimited). Support both 0 and u16::MAX as unlimited.
        let consecutive_fire_limit_raw = neuron_array.consecutive_fire_limits()[idx];
        let consecutive_fire_limit = if consecutive_fire_limit_raw == 0 {
            u16::MAX
        } else {
            consecutive_fire_limit_raw
        };
        let consecutive_fire_count = neuron_array.consecutive_fire_counts()[idx];

        // SIMD-friendly: uniform comparison (no branching for "no limit" case)
        // If consecutive_fire_limit == u16::MAX, count will always be < MAX
        let consecutive_fire_constraint = consecutive_fire_count < consecutive_fire_limit;

        if !consecutive_fire_constraint {
            // Neuron exceeded consecutive fire limit - prevent firing
            // CRITICAL: Reset count to 0 (matches Python SIMD: all non-firing neurons get count reset)
            // This allows the neuron to fire again after being blocked for one burst
            neuron_array.consecutive_fire_counts_mut()[idx] = 0;

            // CRITICAL FIX: Don't apply leak when blocked by consecutive fire limit
            // The neuron will enter refractory period, and leak will be applied during refractory
            // Applying leak here causes the neuron to lose potential and need extra time to re-fire
            // This was causing the "gap of 3 instead of 2" bug

            return None;
        }

        // 6. Apply probabilistic excitability
        let excitability = neuron_array.excitabilities()[idx];

        // Fast path: excitability >= 0.999 means always fire (matches Python)
        let should_fire = if excitability >= 0.999 {
            true
        } else if excitability <= 0.0 {
            false
        } else {
            // Probabilistic firing based on excitability (matches Python RNG logic)
            // CRITICAL: Use excitability_random() which combines neuron_id AND burst_count
            // This ensures different random values each burst (e.g., 20% excitability = 20% chance per burst)
            let random_val = excitability_random(neuron_id.0, burst_count);
            random_val < excitability
        };

        if should_fire {
            if allow_trace {
                debug!(
                    target: "feagi-npu-trace",
                    "[DYN] burst={} neuron={} area={} mp_acc={} FIRED mp_new={:.6} thr={:.6} refrac_period={} cfc={}/{} snooze={}",
                    burst_count,
                    neuron_id.0,
                    cortical_idx,
                    mp_acc,
                    charge,
                    threshold_charge,
                    neuron_array.refractory_periods()[idx],
                    neuron_array.consecutive_fire_counts()[idx],
                    neuron_array.consecutive_fire_limits()[idx],
                    neuron_array.snooze_periods()[idx]
                );
            }
            // Reset membrane potential and the fractional leftover.
            neuron_array.membrane_potentials_mut()[idx] = T::zero();
            neuron_array.membrane_fractions_mut()[idx] = 0.0;

            // Increment consecutive fire count (saturating to prevent overflow)
            let old_count = neuron_array.consecutive_fire_counts()[idx];
            neuron_array.consecutive_fire_counts_mut()[idx] = old_count.saturating_add(1);
            let new_count = neuron_array.consecutive_fire_counts()[idx];

            // Apply refractory period (additive if hit consecutive fire limit)
            // SEMANTICS: refractory_period=N means "skip N bursts between fires"
            // e.g., refrac=1 → fire, skip 1, fire → pattern: 1_1_1_
            //       refrac=2 → fire, skip 2, fire → pattern: 1__1__
            let refractory_period = neuron_array.refractory_periods()[idx];
            let consecutive_fire_limit_raw = neuron_array.consecutive_fire_limits()[idx];
            let consecutive_fire_limit = if consecutive_fire_limit_raw == 0 {
                u16::MAX
            } else {
                consecutive_fire_limit_raw
            };

            if consecutive_fire_limit_raw != 0
                && consecutive_fire_limit != u16::MAX
                && new_count >= consecutive_fire_limit
            {
                // Hit burst limit → ADDITIVE extended refractory
                // countdown = refrac + snooze (total bursts to skip)
                // e.g., refrac=1, snooze=2, cfc_limit=3 → 1_1_1___1_1_1___
                let snooze_period = neuron_array.snooze_periods()[idx];
                let countdown = refractory_period.saturating_add(snooze_period);
                neuron_array.refractory_countdowns_mut()[idx] = countdown;
                // if neuron_id.0 == 16438 {
                //     println!("[RUST-16438] → FIRED! count {} → {} (HIT LIMIT), extended refrac={}+{}={}",
                //              new_count-1, new_count, refractory_period, snooze_period, countdown);
                // }
                // Note: consecutive_fire_count will be reset when countdown expires
            } else {
                // Normal fire → normal refractory only
                // countdown = refrac (bursts to skip)
                // e.g., refrac=1 → countdown=1 → fire, 1 blocked, fire
                neuron_array.refractory_countdowns_mut()[idx] = refractory_period;
                // if neuron_id.0 == 16438 {
                //     println!("[RUST-16438] → FIRED! count {} → {}, refrac={}",
                //              new_count-1, new_count, refractory_period);
                // }
            }

            // Get neuron coordinates
            let coord_idx = idx * 3;
            let (x, y, z) = (
                neuron_array.coordinates()[coord_idx],
                neuron_array.coordinates()[coord_idx + 1],
                neuron_array.coordinates()[coord_idx + 2],
            );

            return Some(FiringNeuron {
                neuron_id,
                membrane_potential: charge,
                cortical_idx: neuron_array.cortical_areas()[idx], // Use cortical_idx directly - no conversion needed
                x,
                y,
                z,
                fire_kind: FIRE_KIND_STDP_ELIGIBLE,
            });
        }
    }

    // Neuron did not fire - reset consecutive fire count. The caught-up charge stays stored;
    // the next leak is applied lazily at the next input (or by readers).

    // Reset consecutive fire count (matches Python SIMD implementation)
    let consecutive_fire_limit = neuron_array.consecutive_fire_limits()[idx];
    if consecutive_fire_limit != 0 && consecutive_fire_limit != u16::MAX {
        neuron_array.consecutive_fire_counts_mut()[idx] = 0;
    }

    if allow_trace {
        debug!(
            target: "feagi-npu-trace",
            "[DYN] burst={} neuron={} area={} mp_acc={} NOFIRE mp_old={:.6} cand={:.6} mp_decayed={:.6} mp_stored={:.6} thr={:.6} leak={:.6}",
            burst_count,
            neuron_id.0,
            cortical_idx,
            mp_acc,
            old_potential.to_f32() + old_fraction,
            candidate_potential,
            decayed,
            neuron_array.membrane_potentials()[idx].to_f32()
                + neuron_array.membrane_fractions()[idx],
            threshold_charge,
            leak_coefficient
        );
    }

    None
}

// REMOVED: process_neural_dynamics_simd - dead code with fallback
// SIMD optimization should be done in platform-agnostic core (feagi-neural) if needed

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fire_structures::{FIRE_KIND_EPISODIC_MEMORY, FIRE_KIND_STDP_ELIGIBLE};
    use crate::sparse_memory_lif::MemoryAssociativeLifParams;
    use ahash::AHashSet;
    use feagi_npu_runtime::StdNeuronArray; // OK: dev-dependency for tests

    fn empty_assoc_sparse() -> crate::sparse_memory_lif::SparseMemoryAssociativeLifStates {
        ahash::AHashMap::new()
    }

    #[test]
    fn test_neuron_fires_when_above_threshold() {
        let mut neurons = StdNeuronArray::new(10);
        let mut empty_sparse = empty_assoc_sparse();

        // Add a neuron with threshold 1.0
        let id = neurons
            .add_neuron(
                1.0,      // threshold
                f32::MAX, // threshold_limit (MAX = no limit, SIMD-friendly encoding)
                0.0,      // leak_coefficient
                0.0,      // resting_potential
                0,        // neuron_type
                5,        // refractory_period
                1.0,      // excitability
                u16::MAX, // consecutive_fire_limit (MAX = unlimited, SIMD-friendly encoding)
                0,        // snooze_period
                true,     // mp_charge_accumulation
                1,        // cortical_area
                0,
                0,
                0,
            )
            .unwrap();

        // Create FCL with enough potential to fire
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), 1.5);

        // Process dynamics
        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();

        assert_eq!(result.neurons_fired, 1);
        assert_eq!(result.fire_queue.total_neurons(), 1);
        assert_eq!(neurons.membrane_potentials[0].to_f32(), 0.0); // Reset after firing
        assert_eq!(neurons.refractory_countdowns[0], 5); // Refractory set
    }

    #[test]
    fn test_consecutive_fire_limit_zero_does_not_block_firing() {
        let mut neurons = StdNeuronArray::new(10);
        let mut empty_sparse = empty_assoc_sparse();

        // consecutive_fire_limit=0 should DISABLE the limiter (unlimited).
        let id = neurons
            .add_neuron(
                1.0,      // threshold
                f32::MAX, // threshold_limit (MAX = no limit)
                0.0,      // leak_coefficient
                0.0,      // resting_potential
                0,        // neuron_type
                0,        // refractory_period (so we can fire back-to-back)
                1.0,      // excitability
                0,        // consecutive_fire_limit (0 = disabled/unlimited)
                0,        // snooze_period
                true,     // mp_charge_accumulation
                1,        // cortical_area
                0,
                0,
                0,
            )
            .unwrap();

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), 2.0);

        // Burst 0: should fire
        let result0 = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();
        assert_eq!(result0.neurons_fired, 1);

        // Burst 1: should ALSO fire (still unlimited)
        let result1 = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            1,
        )
        .unwrap();
        assert_eq!(result1.neurons_fired, 1);
    }

    #[test]
    fn test_neuron_does_not_fire_below_threshold() {
        let mut neurons = StdNeuronArray::new(10);
        let mut empty_sparse = empty_assoc_sparse();

        let id = neurons
            .add_neuron(
                1.0,
                f32::MAX,
                0.1,
                0.0,
                0,
                5,
                1.0,
                u16::MAX,
                0,
                true,
                1,
                0,
                0,
                0,
            )
            .unwrap();

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), 0.5); // Below threshold

        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();

        assert_eq!(result.neurons_fired, 0);
        assert_eq!(result.fire_queue.total_neurons(), 0);
        assert!(neurons.membrane_potentials[0].to_f32() > 0.0); // Potential accumulated
    }

    #[test]
    fn test_mp_charge_accumulation_skips_negative_fcl_candidates() {
        let mut neurons = StdNeuronArray::new(10);
        let mut empty_sparse = empty_assoc_sparse();
        let id = neurons
            .add_neuron(
                100.0,    // threshold — no fire
                f32::MAX, // threshold_limit
                0.0,      // leak_coefficient
                0.0,
                0,
                5,
                1.0,
                u16::MAX,
                0,
                true, // mp_charge_accumulation
                1,
                0,
                0,
                0,
            )
            .unwrap();

        neurons.membrane_potentials[id] = 3.0f32;

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), -10.0);

        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();
        assert_eq!(result.neurons_fired, 0);
        assert_eq!(neurons.membrane_potentials[id], 3.0);
    }

    #[test]
    fn test_mp_charge_accumulation_false_still_applies_negative_fcl_candidates() {
        let mut neurons = StdNeuronArray::new(10);
        let mut empty_sparse = empty_assoc_sparse();
        let id = neurons
            .add_neuron(
                100.0,
                f32::MAX,
                0.0,
                0.0,
                0,
                5,
                1.0,
                u16::MAX,
                0,
                false, // mp_charge_accumulation
                1,
                0,
                0,
                0,
            )
            .unwrap();

        neurons.membrane_potentials[id] = 3.0f32;

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), -10.0);

        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();
        assert_eq!(result.neurons_fired, 0);
        assert_eq!(neurons.membrane_potentials[id], -7.0);
    }

    #[test]
    fn test_refractory_period_blocks_firing() {
        let mut neurons = StdNeuronArray::new(10);
        let mut empty_sparse = empty_assoc_sparse();

        let id = neurons
            .add_neuron(
                1.0,
                f32::MAX,
                0.0,
                0.0,
                0,
                5,
                1.0,
                u16::MAX,
                0,
                true,
                1,
                0,
                0,
                0,
            )
            .unwrap();

        // Set refractory countdown
        neurons.refractory_countdowns[0] = 3;

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), 2.0); // Well above threshold

        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();

        assert_eq!(result.neurons_fired, 0);
        assert_eq!(neurons.refractory_countdowns[0], 3); // Unchanged until end-of-burst tick
        let fired: AHashSet<u32> = result
            .fire_queue
            .get_all_neuron_ids()
            .iter()
            .map(|n| n.0)
            .collect();
        finish_burst_refractory_period(&mut neurons, &fired);
        assert_eq!(neurons.refractory_countdowns[0], 2); // One global burst elapsed
    }

    #[test]
    fn test_refractory_countdown_ticks_without_fcl_entry() {
        let mut neurons = StdNeuronArray::new(10);
        let mut empty_sparse = empty_assoc_sparse();

        let _id = neurons
            .add_neuron(
                1.0,
                f32::MAX,
                0.0,
                0.0,
                0,
                5,
                1.0,
                u16::MAX,
                0,
                true,
                1,
                0,
                0,
                0,
            )
            .unwrap();

        neurons.refractory_countdowns[0] = 4;

        let fcl = FireCandidateList::new();
        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();
        assert_eq!(result.neurons_processed, 0);
        assert_eq!(neurons.refractory_countdowns[0], 4);

        finish_burst_refractory_period(&mut neurons, &AHashSet::new());
        assert_eq!(neurons.refractory_countdowns[0], 3);
    }

    #[test]
    fn test_leak_decay() {
        let mut neurons = StdNeuronArray::new(10);
        let mut empty_sparse = empty_assoc_sparse();

        let id = neurons
            .add_neuron(
                10.0, // High threshold (won't fire)
                0.0,  // threshold_limit
                0.5,  // leak_coefficient (50% leak toward resting)
                0.0,  // resting_potential
                0,    // neuron_type
                0,    // refractory_period
                1.0,  // excitability
                0,    // consecutive_fire_limit
                0,    // snooze_period
                true, // mp_charge_accumulation
                1,    // cortical_area
                0, 0, 0,
            )
            .unwrap();

        // Set initial potential
        neurons.membrane_potentials[0] = 1.0;

        // Add small candidate potential
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), 0.1);

        process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            1,
        )
        .unwrap();

        // Per-burst LIF, leak first then input: V_1 = 1.0 * (1 - 0.5) + 0.1 = 0.6
        // (V_0 = 1.0 was last updated at burst 0, so one burst of leak is caught up.)
        assert!((neurons.membrane_potentials[0].to_f32() - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_multiple_neurons_firing() {
        let mut neurons = StdNeuronArray::new(100);
        let mut empty_sparse = empty_assoc_sparse();

        // Add 10 neurons
        let mut ids = Vec::new();
        for _i in 0..10 {
            let id = neurons
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.1,
                    0.0,
                    0,
                    5,
                    1.0,
                    u16::MAX,
                    0,
                    true,
                    1,
                    0,
                    0,
                    0,
                )
                .unwrap();
            ids.push(id);
        }

        // Create FCL with all above threshold
        let mut fcl = FireCandidateList::new();
        for id in &ids {
            fcl.add_candidate(NeuronId(*id as u32), 1.5);
        }

        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();

        assert_eq!(result.neurons_processed, 10);
        assert_eq!(result.neurons_fired, 10);
    }

    #[test]
    fn test_simd_threshold_limit_blocks_firing_and_preserves_mp() {
        let neuron_count = 10_000; // Triggers SIMD path (threshold is 10_000)
        let mut neurons = StdNeuronArray::new(neuron_count + 16);
        let mut empty_sparse = empty_assoc_sparse();

        let mut fcl = FireCandidateList::new();
        for _ in 0..neuron_count {
            let id = neurons
                .add_neuron(
                    1.0,  // threshold
                    1.5,  // threshold_limit (cap)
                    0.0,  // leak_coefficient (keep MP unchanged on no-fire)
                    0.0,  // resting_potential
                    0,    // neuron_type
                    0,    // refractory_period
                    1.0,  // excitability
                    0,    // consecutive_fire_limit
                    0,    // snooze_period
                    true, // mp_charge_accumulation
                    1,    // cortical_area
                    0, 0, 0,
                )
                .unwrap();
            fcl.add_candidate(NeuronId(id as u32), 2.0); // MP crosses threshold but exceeds limit
        }

        let result = process_neural_dynamics(
            &fcl,
            None,
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            0,
        )
        .unwrap();

        assert_eq!(result.neurons_fired, 0);
        assert_eq!(result.fire_queue.total_neurons(), 0);

        // Blocked-by-limit neurons must keep MP at 2.0 (with leak=0.0).
        for idx in 0..neuron_count {
            assert!((neurons.membrane_potentials[idx].to_f32() - 2.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_memory_neuron_forced_fire_without_storage_backing() {
        let mut neurons = StdNeuronArray::<f32>::new(1);
        let mut empty_sparse = empty_assoc_sparse();

        let memory_id = NeuronId(50_000_000);
        let cortical_idx = 42u32;
        let potential = 1.5f32;

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(memory_id, potential);

        let mut mem_map = ahash::AHashMap::new();
        mem_map.insert(memory_id.0, cortical_idx);

        let result = process_neural_dynamics(
            &fcl,
            Some(&mem_map),
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            7,
        )
        .unwrap();

        assert_eq!(result.neurons_fired, 1);
        assert_eq!(result.fire_queue.total_neurons(), 1);

        let area_neurons = result.fire_queue.get_area_neurons(cortical_idx).unwrap();
        assert_eq!(area_neurons.len(), 1);
        assert_eq!(area_neurons[0].neuron_id, memory_id);
        assert_eq!(area_neurons[0].cortical_idx, cortical_idx);
        assert!((area_neurons[0].membrane_potential - potential).abs() < 1e-6);
    }

    #[test]
    fn test_memory_neuron_forced_fire_allows_cortical_idx_zero() {
        let mut neurons = StdNeuronArray::<f32>::new(1);
        let mut empty_sparse = empty_assoc_sparse();

        let memory_id = NeuronId(50_000_000);
        let cortical_idx = 0u32; // valid in some deployments (0-based cortical indices)

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(memory_id, 1.5);

        let mut mem_map = ahash::AHashMap::new();
        mem_map.insert(memory_id.0, cortical_idx);

        let result = process_neural_dynamics(
            &fcl,
            Some(&mem_map),
            None,
            None,
            &mut empty_sparse,
            None,
            &mut neurons,
            1,
        )
        .unwrap();
        assert_eq!(result.fire_queue.total_neurons(), 1);
        assert!(result.fire_queue.get_area_neurons(cortical_idx).is_some());
    }

    #[test]
    fn test_memory_neuron_associative_sparse_lif_fires_when_above_threshold() {
        let mut neurons = StdNeuronArray::<f32>::new(1);
        let mut empty_sparse = empty_assoc_sparse();

        let memory_id = NeuronId(50_000_000);
        let cortical_idx = 7u32;
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(memory_id, 2.0);

        let mut mem_map = ahash::AHashMap::new();
        mem_map.insert(memory_id.0, cortical_idx);

        let mut assoc_psp = ahash::AHashMap::new();
        assoc_psp.insert(memory_id.0, 2.0);

        let mut lif_params = ahash::AHashMap::new();
        lif_params.insert(
            cortical_idx,
            MemoryAssociativeLifParams {
                threshold: 1.0,
                ..Default::default()
            },
        );

        let result = process_neural_dynamics(
            &fcl,
            Some(&mem_map),
            None,
            Some(&assoc_psp),
            &mut empty_sparse,
            Some(&lif_params),
            &mut neurons,
            0,
        )
        .unwrap();

        assert_eq!(result.neurons_fired, 1);
        let area = result.fire_queue.get_area_neurons(cortical_idx).unwrap();
        assert_eq!(area[0].fire_kind, FIRE_KIND_STDP_ELIGIBLE);
        assert!(!empty_sparse.is_empty());
    }

    #[test]
    fn test_memory_neuron_associative_lif_threshold_three_needs_accumulated_drive() {
        let mut neurons = StdNeuronArray::<f32>::new(1);
        let mut sparse = empty_assoc_sparse();
        let memory_id = NeuronId(50_000_000);
        let cortical_idx = 7u32;
        let mut mem_map = ahash::AHashMap::new();
        mem_map.insert(memory_id.0, cortical_idx);
        let mut lif_params = ahash::AHashMap::new();
        lif_params.insert(
            cortical_idx,
            MemoryAssociativeLifParams {
                threshold: 3.0,
                ..Default::default()
            },
        );

        let mut fired_per_burst = Vec::new();
        for burst in 0..3u64 {
            let mut fcl = FireCandidateList::new();
            fcl.add_candidate(memory_id, 1.0);
            let mut assoc_psp = ahash::AHashMap::new();
            assoc_psp.insert(memory_id.0, 1.0);
            let result = process_neural_dynamics(
                &fcl,
                Some(&mem_map),
                None,
                Some(&assoc_psp),
                &mut sparse,
                Some(&lif_params),
                &mut neurons,
                burst,
            )
            .unwrap();
            fired_per_burst.push(result.neurons_fired);
        }
        assert_eq!(fired_per_burst, vec![0, 0, 1]);
    }

    #[test]
    fn test_memory_neuron_associative_lif_no_fire_below_threshold() {
        let mut neurons = StdNeuronArray::<f32>::new(1);
        let mut empty_sparse = empty_assoc_sparse();

        let memory_id = NeuronId(50_000_000);
        let cortical_idx = 7u32;
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(memory_id, 0.3);

        let mut mem_map = ahash::AHashMap::new();
        mem_map.insert(memory_id.0, cortical_idx);

        let mut assoc_psp = ahash::AHashMap::new();
        assoc_psp.insert(memory_id.0, 0.3);

        let mut lif_params = ahash::AHashMap::new();
        lif_params.insert(
            cortical_idx,
            MemoryAssociativeLifParams {
                threshold: 1.0,
                ..Default::default()
            },
        );

        let result = process_neural_dynamics(
            &fcl,
            Some(&mem_map),
            None,
            Some(&assoc_psp),
            &mut empty_sparse,
            Some(&lif_params),
            &mut neurons,
            0,
        )
        .unwrap();

        assert_eq!(result.neurons_fired, 0);
    }

    #[test]
    fn test_memory_neuron_episodic_preempts_associative_sparse_lif() {
        let mut neurons = StdNeuronArray::<f32>::new(1);
        let mut empty_sparse = empty_assoc_sparse();

        let memory_id = NeuronId(50_000_000);
        let cortical_idx = 7u32;
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(memory_id, 2.0);

        let mut mem_map = ahash::AHashMap::new();
        mem_map.insert(memory_id.0, cortical_idx);

        let mut fire_kind = ahash::AHashMap::new();
        fire_kind.insert(memory_id.0, FIRE_KIND_EPISODIC_MEMORY);

        let mut assoc_psp = ahash::AHashMap::new();
        assoc_psp.insert(memory_id.0, 2.0);

        let mut lif_params = ahash::AHashMap::new();
        lif_params.insert(
            cortical_idx,
            MemoryAssociativeLifParams {
                threshold: 100.0,
                ..Default::default()
            },
        );

        let result = process_neural_dynamics(
            &fcl,
            Some(&mem_map),
            Some(&fire_kind),
            Some(&assoc_psp),
            &mut empty_sparse,
            Some(&lif_params),
            &mut neurons,
            0,
        )
        .unwrap();

        assert_eq!(result.neurons_fired, 1);
        let area = result.fire_queue.get_area_neurons(cortical_idx).unwrap();
        assert_eq!(area[0].fire_kind, FIRE_KIND_EPISODIC_MEMORY);
    }

    // ---- Lazy per-burst LIF: V_t = V_{t-1} * (1 - g) + I_t, applied when input arrives ----

    /// One non-refractory, never-firing-by-default neuron array for the leak tests.
    fn lif_test_neuron(threshold: f32, leak: f32) -> StdNeuronArray<f32> {
        let mut neurons = StdNeuronArray::new(4);
        neurons
            .add_neuron(
                threshold,
                f32::MAX, // no upper limit
                leak,
                0.0,
                0,
                0, // no refractory
                1.0,
                u16::MAX,
                0,
                true, // mp_charge_accumulation
                1,
                0,
                0,
                0,
            )
            .unwrap();
        neurons
    }

    /// Run one burst with `input` on neuron 0. Returns whether it fired.
    fn lif_burst(neurons: &mut StdNeuronArray<f32>, burst: u64, input: f32) -> bool {
        let mut sparse = empty_assoc_sparse();
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(0), input);
        process_neural_dynamics(&fcl, None, None, None, &mut sparse, None, neurons, burst)
            .unwrap()
            .neurons_fired
            == 1
    }

    fn stored_v(neurons: &StdNeuronArray<f32>) -> f64 {
        (neurons.membrane_potentials[0] + neurons.membrane_fractions[0]) as f64
    }

    #[test]
    fn lazy_leak_constant_input_matches_per_burst_recurrence() {
        // Storage is f32, so the per-burst recurrence is evaluated in f32 as well (f64 closed
        // form checked to f32 rounding, 5e-6).
        let (g, input) = (0.15_f32, 0.4_f32);
        let mut neurons = lif_test_neuron(1000.0, g);
        let mut v = 0.0_f32;
        let mut max_err = 0.0_f64;
        let mut max_closed_err = 0.0_f64;
        for n in 1..=40u64 {
            assert!(!lif_burst(&mut neurons, n, input));
            v = v * (1.0 - g) + input;
            let closed = input as f64 * (1.0 - (1.0 - g as f64).powi(n as i32)) / g as f64;
            let err = (stored_v(&neurons) - v as f64).abs();
            let closed_err = (stored_v(&neurons) - closed).abs();
            max_err = max_err.max(err);
            max_closed_err = max_closed_err.max(closed_err);
            assert!(
                err < 1e-6,
                "burst {n}: stored {} recurrence {v}",
                stored_v(&neurons)
            );
            assert!(
                closed_err < 5e-6,
                "burst {n}: stored {} closed {closed}",
                stored_v(&neurons)
            );
        }
        eprintln!(
            "constant input: V_40={:.6} max_err_vs_recurrence={:.2e} max_err_vs_closed_form={:.2e}",
            stored_v(&neurons),
            max_err,
            max_closed_err
        );
    }

    #[test]
    fn lazy_leak_input_every_fifth_burst_matches_brute_force() {
        let (g, input) = (0.15_f32, 0.4_f32);
        let mut neurons = lif_test_neuron(1000.0, g);
        let mut v = 0.0_f32;
        let mut max_err = 0.0_f64;
        for t in 1..=100u64 {
            // Brute force: leak every burst, add input only on every 5th.
            let i_t = if t % 5 == 0 { input } else { 0.0 };
            v = v * (1.0 - g) + i_t;
            if t % 5 == 0 {
                lif_burst(&mut neurons, t, input);
                let err = (stored_v(&neurons) - v as f64).abs();
                max_err = max_err.max(err);
                assert!(
                    err < 1e-6,
                    "burst {t}: stored {} brute force {v}",
                    stored_v(&neurons)
                );
            }
        }
        eprintln!(
            "every 5th: V_100={:.6} max_err={:.2e}",
            stored_v(&neurons),
            max_err
        );
    }

    #[test]
    fn lazy_leak_strong_leak_never_fires_weak_leak_fires_on_sparse_input() {
        // Input 0.4 every 3rd burst, threshold 1.0.
        // g=0.5: V saturates at 0.4 / (1 - 0.5^3) = 0.457 < 1.0, never fires.
        let mut strong = lif_test_neuron(1.0, 0.5);
        let mut peak = 0.0_f64;
        for t in (3..=300u64).step_by(3) {
            assert!(!lif_burst(&mut strong, t, 0.4), "g=0.5 fired at burst {t}");
            peak = peak.max(stored_v(&strong));
        }
        assert!((peak - 0.4 / (1.0 - 0.125)).abs() < 1e-4, "peak {peak}");

        // g=0.02: V -> 0.4 / (1 - 0.98^3) = 6.8 > 1.0, fires.
        let mut weak = lif_test_neuron(1.0, 0.02);
        let first_fire = (3..=300u64)
            .step_by(3)
            .find(|&t| lif_burst(&mut weak, t, 0.4));
        eprintln!(
            "g=0.5 peak V={peak:.4} (never fired); g=0.02 first fire at burst {first_fire:?}"
        );
        assert!(first_fire.is_some(), "g=0.02 must fire");
    }

    #[test]
    fn lazy_leak_simd_batch_path_matches_scalar_path() {
        // 10_000+ candidates take the batch path; neuron 0 is compared with a scalar array.
        let (g, input) = (0.15_f32, 0.4_f32);
        let n = 10_050usize;
        let mut batch = StdNeuronArray::<f32>::new(n);
        for _ in 0..n {
            batch
                .add_neuron(
                    1000.0,
                    f32::MAX,
                    g,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    true,
                    1,
                    0,
                    0,
                    0,
                )
                .unwrap();
        }
        let mut scalar = lif_test_neuron(1000.0, g);
        for t in (5..=50u64).step_by(5) {
            let mut sparse = empty_assoc_sparse();
            let mut fcl = FireCandidateList::new();
            for id in 0..n {
                fcl.add_candidate(NeuronId(id as u32), input);
            }
            process_neural_dynamics(&fcl, None, None, None, &mut sparse, None, &mut batch, t)
                .unwrap();
            lif_burst(&mut scalar, t, input);
            let b = (batch.membrane_potentials[7] + batch.membrane_fractions[7]) as f64;
            assert!(
                (b - stored_v(&scalar)).abs() < 1e-6,
                "burst {t}: batch {b} scalar {}",
                stored_v(&scalar)
            );
            assert_eq!(batch.last_update_bursts[7], t);
        }
    }

    #[test]
    fn caught_up_reader_decays_without_mutating() {
        let g = 0.1_f32;
        let mut neurons = lif_test_neuron(1000.0, g);
        lif_burst(&mut neurons, 10, 5.0);
        let (mp, fr, last) = (
            neurons.membrane_potentials[0],
            neurons.membrane_fractions[0],
            neurons.last_update_bursts[0],
        );
        let at_input = caught_up_membrane_charge(&neurons, 0, 10);
        let later = caught_up_membrane_charge(&neurons, 0, 13);
        assert!((at_input - 5.0).abs() < 1e-6);
        assert!(
            (later - 5.0 * 0.9_f32.powi(3)).abs() < 1e-6,
            "later {later}"
        );
        // State untouched, repeated reads agree.
        assert_eq!(neurons.membrane_potentials[0], mp);
        assert_eq!(neurons.membrane_fractions[0], fr);
        assert_eq!(neurons.last_update_bursts[0], last);
        assert_eq!(later, caught_up_membrane_charge(&neurons, 0, 13));
        eprintln!("reader: V at input burst={at_input:.6}, 3 bursts later={later:.6}");
    }

    #[test]
    fn catch_up_before_leak_change_decays_with_old_leak() {
        let mut neurons = lif_test_neuron(1000.0, 0.1);
        lif_burst(&mut neurons, 10, 5.0);
        // Homeostat changes the leak at burst 13; the idle stretch must decay with 0.1.
        catch_up_before_leak_change(&mut neurons, 0, 0.1, 13);
        neurons.leak_coefficients[0] = 0.5;
        assert_eq!(neurons.last_update_bursts[0], 13);
        let v13 = caught_up_membrane_charge(&neurons, 0, 13);
        assert!((v13 - 5.0 * 0.9_f32.powi(3)).abs() < 1e-5, "v13 {v13}");
        // Later bursts decay with the new leak from the catch-up point.
        let v14 = caught_up_membrane_charge(&neurons, 0, 14);
        assert!(
            (v14 - 5.0 * 0.9_f32.powi(3) * 0.5).abs() < 1e-5,
            "v14 {v14}"
        );
        // Already current: no-op.
        catch_up_before_leak_change(&mut neurons, 0, 0.9, 13);
        assert!((caught_up_membrane_charge(&neurons, 0, 13) - v13).abs() < 1e-7);
    }
}

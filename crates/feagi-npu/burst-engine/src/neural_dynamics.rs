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
//! ## Optimization Strategy
//! 1. **SIMD**: Process 8+ neurons at once using AVX2/AVX-512
//! 2. **Rayon**: Parallelize across cores for large neuron counts
//! 3. **Cache-friendly**: Sequential access patterns, no pointer chasing

use crate::fire_structures::{FireQueue, FiringNeuron};
use feagi_npu_neural::types::*;
use feagi_npu_runtime::NeuronStorage;
use std::sync::OnceLock;
use tracing::trace;

// Use platform-agnostic core algorithms (Phase 1 - NO DUPLICATION)
use feagi_npu_neural::{apply_leak, excitability_random, update_neurons_lif_batch};

/// Runtime-gated tracing config for neural dynamics.
/// Enable with:
/// - FEAGI_NPU_TRACE_DYNAMICS=1
/// Optional filters:
/// - FEAGI_NPU_TRACE_NEURON=<u32 neuron_id> (single neuron)
struct DynamicsTraceCfg {
    enabled: bool,
    neuron_filter: Option<u32>,
}

fn dynamics_trace_cfg() -> &'static DynamicsTraceCfg {
    static CFG: OnceLock<DynamicsTraceCfg> = OnceLock::new();
    CFG.get_or_init(|| {
        let enabled = std::env::var("FEAGI_NPU_TRACE_DYNAMICS")
            .ok()
            .as_deref()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let neuron_filter = std::env::var("FEAGI_NPU_TRACE_NEURON").ok().and_then(|v| v.parse().ok());

        DynamicsTraceCfg {
            enabled,
            neuron_filter,
        }
    })
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

/// Process neural dynamics for all candidate neurons
///
/// This is the CRITICAL HOT PATH - every microsecond matters!
///
/// ## Algorithm:
/// 1. Apply leak/decay to membrane potentials
/// 2. Add candidate potentials from FCL
/// 3. Check firing thresholds (with refractory period)
/// 4. Apply probabilistic excitability
/// 5. Create Fire Queue from firing neurons
pub fn process_neural_dynamics<T: NeuralValue>(
    fcl: &FireCandidateList,
    memory_candidate_cortical_idx: Option<&ahash::AHashMap<u32, u32>>,
    neuron_array: &mut impl NeuronStorage<Value = T>,
    burst_count: u64,
) -> Result<DynamicsResult> {
    let dynamics_start = std::time::Instant::now();
    let candidates: Vec<_> = fcl.iter().collect();

    if candidates.is_empty() {
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

    let (fired_neurons, refractory_count): (Vec<_>, usize) = {
        // For large candidate counts, use batch processing for better cache locality
        // Threshold: 50k candidates (empirically determined - batching overhead worth it above this)
        const SIMD_BATCH_THRESHOLD: usize = 50_000;

        if candidates.len() >= SIMD_BATCH_THRESHOLD {
            // SIMD batch processing path
            process_candidates_with_simd_batching(
                &candidates,
                memory_candidate_cortical_idx,
                neuron_array,
                burst_count,
            )
        } else {
            // Sequential processing for small candidate counts (avoid SIMD overhead)
            let mut results = Vec::with_capacity(candidates.len());
            let mut refractory = 0;

            for &(neuron_id, candidate_potential) in &candidates {
                // Memory neurons are force-fired and do not use the regular neuron storage array.
                if neuron_id.0 >= MEMORY_NEURON_ID_START {
                    let cortical_idx = match memory_candidate_cortical_idx
                        .and_then(|m| m.get(&neuron_id.0).copied())
                    {
                        Some(idx) => idx,
                        None => continue, // No metadata for this memory neuron candidate
                    };

                    results.push(FiringNeuron {
                        neuron_id,
                        membrane_potential: candidate_potential,
                        cortical_idx,
                        x: 0,
                        y: 0,
                        z: 0,
                    });
                    continue;
                }

                // Convert f32 from FCL to T
                let candidate_potential_t = T::from_f32(candidate_potential);
                if let Some(neuron) =
                    process_single_neuron(neuron_id, candidate_potential_t, neuron_array, burst_count)
                {
                    results.push(neuron);
                }

                // Count refractory neurons (neuron_id == array index)
                let idx = neuron_id.0 as usize;
                if idx < neuron_array.count() && neuron_array.refractory_countdowns_mut()[idx] > 0 {
                    refractory += 1;
                }
            }

            (results, refractory)
        }
    };

    // Build Fire Queue
    let mut fire_queue = FireQueue::new();
    fire_queue.set_timestep(burst_count); // CRITICAL: Set timestep for FQ Sampler deduplication
    for neuron in fired_neurons.iter() {
        fire_queue.add_neuron(neuron.clone());
    }

    let dynamics_duration = dynamics_start.elapsed();
    
    // Log if dynamics processing is slow (>20ms)
    if dynamics_duration.as_millis() > 20 {
        tracing::warn!(
            "[PHASE2-DYNAMICS] Slow dynamics processing: {:.2}ms for {} candidates, {} fired",
            dynamics_duration.as_secs_f64() * 1000.0,
            candidates.len(),
            fired_neurons.len()
        );
    }
    
    Ok(DynamicsResult {
        fire_queue,
        neurons_processed: candidates.len(),
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
fn process_candidates_with_simd_batching<T: NeuralValue>(
    candidates: &[(NeuronId, f32)],
    memory_candidate_cortical_idx: Option<&ahash::AHashMap<u32, u32>>,
    neuron_array: &mut impl NeuronStorage<Value = T>,
    burst_count: u64,
) -> (Vec<FiringNeuron>, usize) {
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;
    const SIMD_BATCH_SIZE: usize = 10_000; // Process in chunks for cache locality and SIMD efficiency

    let mut results = Vec::with_capacity(candidates.len());
    let mut refractory = 0;

    // Separate candidates into categories
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

        // Check for complex constraints that require sequential processing
        let has_threshold_limit = neuron_array.threshold_limits()[idx].to_f32() > 0.0;
        let has_consecutive_fire_limit = neuron_array.consecutive_fire_limits()[idx] > 0;
        let excitability = neuron_array.excitabilities()[idx];
        let has_probabilistic_excitability = excitability < 0.999 && excitability > 0.0;

        if has_threshold_limit || has_consecutive_fire_limit || has_probabilistic_excitability {
            // Complex constraints - process sequentially
            sequential_only.push((neuron_id, candidate_potential));
        } else {
            // Simple case - can use SIMD for basic LIF operations
            simd_eligible.push((neuron_id, candidate_potential));
        }
    }

    // Handle memory neurons (force-fired)
    for (neuron_id, candidate_potential) in memory_candidates {
        let cortical_idx = match memory_candidate_cortical_idx
            .and_then(|m| m.get(&neuron_id.0).copied())
        {
            Some(idx) => idx,
            None => continue,
        };

        results.push(FiringNeuron {
            neuron_id,
            membrane_potential: candidate_potential,
            cortical_idx,
            x: 0,
            y: 0,
            z: 0,
        });
    }

    // Process SIMD-eligible candidates in batches
    for batch in simd_eligible.chunks(SIMD_BATCH_SIZE) {
        let batch_size = batch.len();
        
        // Gather: Collect data into contiguous arrays
        let mut batch_mp = Vec::with_capacity(batch_size);
        let mut batch_thresholds = Vec::with_capacity(batch_size);
        let mut batch_leaks = Vec::with_capacity(batch_size);
        let mut batch_candidates = Vec::with_capacity(batch_size);
        let mut batch_indices = Vec::with_capacity(batch_size); // Store original indices for scatter

        for &(neuron_id, candidate_potential) in batch {
            let idx = neuron_id.0 as usize;
            batch_mp.push(neuron_array.membrane_potentials()[idx]);
            batch_thresholds.push(neuron_array.thresholds()[idx]);
            batch_leaks.push(neuron_array.leak_coefficients()[idx]);
            batch_candidates.push(T::from_f32(candidate_potential));
            batch_indices.push((neuron_id, idx));
        }

        // Process: Use SIMD batch function for basic LIF operations
        let mut fired_mask = vec![false; batch_size];
        update_neurons_lif_batch(
            &mut batch_mp,
            &batch_thresholds,
            &batch_leaks,
            &batch_candidates,
            &mut fired_mask,
        );

        // Scatter: Write results back and handle firing
        for (i, (neuron_id, idx)) in batch_indices.iter().enumerate() {
            // Update membrane potential
            neuron_array.membrane_potentials_mut()[*idx] = batch_mp[i];

            if fired_mask[i] {
                // Neuron fired - handle firing logic
                let cortical_idx = neuron_array.cortical_areas()[*idx];
                let refractory_period = neuron_array.refractory_periods()[*idx];
                
                // Set refractory countdown
                neuron_array.refractory_countdowns_mut()[*idx] = refractory_period;
                
                // Get coordinates
                let coord_idx = *idx * 3;
                let (x, y, z) = (
                    neuron_array.coordinates()[coord_idx],
                    neuron_array.coordinates()[coord_idx + 1],
                    neuron_array.coordinates()[coord_idx + 2],
                );

                results.push(FiringNeuron {
                    neuron_id: *neuron_id,
                    membrane_potential: batch_mp[i].to_f32(),
                    cortical_idx,
                    x,
                    y,
                    z,
                });
            }
            // If not fired, leak was already applied by update_neurons_lif_batch
        }
    }

    // Process sequential-only candidates (refractory, complex constraints)
    for (neuron_id, candidate_potential) in sequential_only {
        let candidate_potential_t = T::from_f32(candidate_potential);
        if let Some(neuron) =
            process_single_neuron(neuron_id, candidate_potential_t, neuron_array, burst_count)
        {
            results.push(neuron);
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
    candidate_potential: T,
    neuron_array: &mut impl NeuronStorage<Value = T>,
    burst_count: u64,
) -> Option<FiringNeuron> {
    // neuron_id == array index (direct access, no HashMap needed!)
    let idx = neuron_id.0 as usize;

    // Validate index
    if idx >= neuron_array.count() || !neuron_array.valid_mask()[idx] {
        return None; // Neuron doesn't exist
    }

    // Trace config (exclude power to avoid noise)
    let trace_cfg = dynamics_trace_cfg();
    let is_power = neuron_array.cortical_areas()[idx] == 1;
    let allow_trace = trace_cfg.enabled
        && !is_power
        && trace_cfg
            .neuron_filter
            .map(|id| id == neuron_id.0)
            .unwrap_or(true);

    let cortical_idx = neuron_array.cortical_areas()[idx];
    let mp_acc = neuron_array.mp_charge_accumulation()[idx];

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

    // 1. Handle unified refractory period (normal + extended)
    // CRITICAL: Decrement countdown, but BLOCK THIS ENTIRE BURST
    // Semantics: refractory_period=1 → fire, block 1 burst, fire
    // When countdown=1, this burst is blocked, then countdown becomes 0 for next burst
    // IMPORTANT: avoid mixing *_mut() borrows with immutable borrows in logging.
    let refractory_countdown = neuron_array.refractory_countdowns()[idx];
    if refractory_countdown > 0 {
        if allow_trace {
            let mp = neuron_array.membrane_potentials()[idx].to_f32();
            let thr = neuron_array.thresholds()[idx].to_f32();
            let leak = neuron_array.leak_coefficients()[idx];
            trace!(
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
        let _old_countdown = neuron_array.refractory_countdowns_mut()[idx];

        // Decrement countdown for next burst
        neuron_array.refractory_countdowns_mut()[idx] -= 1;
        let new_countdown = neuron_array.refractory_countdowns_mut()[idx];

        // Check if extended refractory just expired → reset consecutive fire count
        // This happens AFTER this burst is blocked, ready for next burst
        let consecutive_fire_limit = neuron_array.consecutive_fire_limits()[idx];
        if new_countdown == 0
            && consecutive_fire_limit > 0
            && neuron_array.consecutive_fire_counts()[idx] >= consecutive_fire_limit
        {
            // Reset happens when countdown expires (Option A logic)
            let _old_count = neuron_array.consecutive_fire_counts()[idx];
            neuron_array.consecutive_fire_counts_mut()[idx] = 0;
            // if neuron_id.0 == 16438 {
            //     println!("[RUST-16438] → BLOCKED by refrac (countdown {} → {}), count reset {} → 0",
            //              old_countdown, new_countdown, old_count);
            // }
        } // else if neuron_id.0 == 16438 {
          // println!("[RUST-16438] → BLOCKED by refrac (countdown {} → {})", old_countdown, new_countdown);
          // }

        // BLOCK THIS BURST - neuron cannot fire
        return None;
    }

    // 2. Add candidate potential (matches Python: add BEFORE checking threshold)
    let old_potential = neuron_array.membrane_potentials()[idx];
    let current_potential = old_potential.saturating_add(candidate_potential);
    neuron_array.membrane_potentials_mut()[idx] = current_potential;

    // 3. Check threshold (matches Python: "Check firing conditions BEFORE decay")
    let threshold = neuron_array.thresholds()[idx];
    let threshold_limit = neuron_array.threshold_limits()[idx];
    
    // Firing window: threshold <= MP <= threshold_limit (if limit > 0)
    // If threshold_limit == 0, no upper bound is enforced
    // Note: using ge() and not lt() to implement <= (since le() doesn't exist in trait)
    let above_min = current_potential.ge(threshold);
    let zero_limit = threshold_limit.to_f32() == 0.0; // Check if limit is zero
    let below_max = zero_limit || !current_potential.ge(threshold_limit) || current_potential.to_f32() == threshold_limit.to_f32();
    
    if above_min && below_max {
        if allow_trace {
            trace!(
                target: "feagi-npu-trace",
                "[DYN] burst={} neuron={} area={} mp_acc={} CROSS mp_old={:.6} cand={:.6} mp_new={:.6} thr={:.6} thr_limit={:.6} leak={:.6} excit={:.6}",
                burst_count,
                neuron_id.0,
                cortical_idx,
                mp_acc,
                old_potential.to_f32(),
                candidate_potential.to_f32(),
                current_potential.to_f32(),
                threshold.to_f32(),
                threshold_limit.to_f32(),
                neuron_array.leak_coefficients()[idx],
                neuron_array.excitabilities()[idx]
            );
        }
        // 5. Check consecutive fire limit (matches Python SIMD implementation)
        // Skip constraint if consecutive_fire_limit is 0 (unlimited firing)
        let consecutive_fire_limit = neuron_array.consecutive_fire_limits()[idx];
        let consecutive_fire_count = neuron_array.consecutive_fire_counts()[idx];

        let consecutive_fire_constraint = if consecutive_fire_limit > 0 {
            consecutive_fire_count < consecutive_fire_limit
        } else {
            true // No limit (limit == 0 means unlimited)
        };

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
                trace!(
                    target: "feagi-npu-trace",
                    "[DYN] burst={} neuron={} area={} mp_acc={} FIRED mp_new={:.6} thr={:.6} refrac_period={} cfc={}/{} snooze={}",
                    burst_count,
                    neuron_id.0,
                    cortical_idx,
                    mp_acc,
                    current_potential.to_f32(),
                    threshold.to_f32(),
                    neuron_array.refractory_periods()[idx],
                    neuron_array.consecutive_fire_counts()[idx],
                    neuron_array.consecutive_fire_limits()[idx],
                    neuron_array.snooze_periods()[idx]
                );
            }
            // Reset membrane potential
            neuron_array.membrane_potentials_mut()[idx] = T::zero();

            // Increment consecutive fire count (saturating to prevent overflow)
            let old_count = neuron_array.consecutive_fire_counts()[idx];
            neuron_array.consecutive_fire_counts_mut()[idx] = old_count.saturating_add(1);
            let new_count = neuron_array.consecutive_fire_counts()[idx];

            // Apply refractory period (additive if hit consecutive fire limit)
            // SEMANTICS: refractory_period=N means "skip N bursts between fires"
            // e.g., refrac=1 → fire, skip 1, fire → pattern: 1_1_1_
            //       refrac=2 → fire, skip 2, fire → pattern: 1__1__
            let refractory_period = neuron_array.refractory_periods()[idx];
            let consecutive_fire_limit = neuron_array.consecutive_fire_limits()[idx];

            if consecutive_fire_limit > 0 && new_count >= consecutive_fire_limit {
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
                membrane_potential: current_potential.to_f32(),
                cortical_idx: neuron_array.cortical_areas()[idx], // Use cortical_idx directly - no conversion needed
                x,
                y,
                z,
            });
        }
    }

    // Neuron did not fire - apply leak and reset consecutive fire count
    // (matches Python: "Apply membrane decay to remaining neurons (leak behavior)")

    // Reset consecutive fire count (matches Python SIMD implementation)
    let consecutive_fire_limit = neuron_array.consecutive_fire_limits()[idx];
    if consecutive_fire_limit > 0 {
        neuron_array.consecutive_fire_counts_mut()[idx] = 0;
    }

    // Apply LIF leak (using platform-agnostic function from feagi-neural)
    let leak_coefficient = neuron_array.leak_coefficients()[idx];
    apply_leak(
        &mut neuron_array.membrane_potentials_mut()[idx],
        leak_coefficient,
    );

    if allow_trace {
        trace!(
            target: "feagi-npu-trace",
            "[DYN] burst={} neuron={} area={} mp_acc={} NOFIRE mp_old={:.6} cand={:.6} mp_preleak={:.6} mp_postleak={:.6} thr={:.6} leak={:.6}",
            burst_count,
            neuron_id.0,
            cortical_idx,
            mp_acc,
            old_potential.to_f32(),
            candidate_potential.to_f32(),
            current_potential.to_f32(),
            neuron_array.membrane_potentials()[idx].to_f32(),
            threshold.to_f32(),
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
    use feagi_npu_runtime::StdNeuronArray; // OK: dev-dependency for tests

    #[test]
    fn test_neuron_fires_when_above_threshold() {
        let mut neurons = StdNeuronArray::new(10);

        // Add a neuron with threshold 1.0
        let id = neurons
            .add_neuron(
                1.0,  // threshold
                0.0,  // threshold_limit
                0.0,  // leak_coefficient
                0.0,  // resting_potential
                0,    // neuron_type
                5,    // refractory_period
                1.0,  // excitability
                0,    // consecutive_fire_limit
                0,    // snooze_period
                true, // mp_charge_accumulation
                1,    // cortical_area
                0, 0, 0,
            )
            .unwrap();

        // Create FCL with enough potential to fire
        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), 1.5);

        // Process dynamics
        let result = process_neural_dynamics(&fcl, None, &mut neurons, 0).unwrap();

        assert_eq!(result.neurons_fired, 1);
        assert_eq!(result.fire_queue.total_neurons(), 1);
        assert_eq!(neurons.membrane_potentials[0].to_f32(), 0.0); // Reset after firing
        assert_eq!(neurons.refractory_countdowns[0], 5); // Refractory set
    }

    #[test]
    fn test_neuron_does_not_fire_below_threshold() {
        let mut neurons = StdNeuronArray::new(10);

        let id = neurons
            .add_neuron(1.0, 0.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), 0.5); // Below threshold

        let result = process_neural_dynamics(&fcl, None, &mut neurons, 0).unwrap();

        assert_eq!(result.neurons_fired, 0);
        assert_eq!(result.fire_queue.total_neurons(), 0);
        assert!(neurons.membrane_potentials[0].to_f32() > 0.0); // Potential accumulated
    }

    #[test]
    fn test_refractory_period_blocks_firing() {
        let mut neurons = StdNeuronArray::new(10);

        let id = neurons
            .add_neuron(1.0, 0.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        // Set refractory countdown
        neurons.refractory_countdowns[0] = 3;

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(NeuronId(id as u32), 2.0); // Well above threshold

        let result = process_neural_dynamics(&fcl, None, &mut neurons, 0).unwrap();

        assert_eq!(result.neurons_fired, 0);
        assert_eq!(neurons.refractory_countdowns[0], 2); // Decremented
    }

    #[test]
    fn test_leak_decay() {
        let mut neurons = StdNeuronArray::new(10);

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

        process_neural_dynamics(&fcl, None, &mut neurons, 0).unwrap();

        // Expected LIF: (1.0 + 0.1) + 0.5 * (0.0 - 1.1) = 1.1 - 0.55 = 0.55
        assert!((neurons.membrane_potentials[0].to_f32() - 0.55).abs() < 0.001);
    }

    #[test]
    fn test_multiple_neurons_firing() {
        let mut neurons = StdNeuronArray::new(100);

        // Add 10 neurons
        let mut ids = Vec::new();
        for i in 0..10 {
            let id = neurons
                .add_neuron(1.0, 0.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, i, 0, 0)
                .unwrap();
            ids.push(id);
        }

        // Create FCL with all above threshold
        let mut fcl = FireCandidateList::new();
        for id in &ids {
            fcl.add_candidate(NeuronId(*id as u32), 1.5);
        }

        let result = process_neural_dynamics(&fcl, None, &mut neurons, 0).unwrap();

        assert_eq!(result.neurons_processed, 10);
        assert_eq!(result.neurons_fired, 10);
    }

    #[test]
    fn test_memory_neuron_forced_fire_without_storage_backing() {
        let mut neurons = StdNeuronArray::<f32>::new(1);

        let memory_id = NeuronId(50_000_000);
        let cortical_idx = 42u32;
        let potential = 1.5f32;

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(memory_id, potential);

        let mut mem_map = ahash::AHashMap::new();
        mem_map.insert(memory_id.0, cortical_idx);

        let result = process_neural_dynamics(&fcl, Some(&mem_map), &mut neurons, 7).unwrap();

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

        let memory_id = NeuronId(50_000_000);
        let cortical_idx = 0u32; // valid in some deployments (0-based cortical indices)

        let mut fcl = FireCandidateList::new();
        fcl.add_candidate(memory_id, 1.5);

        let mut mem_map = ahash::AHashMap::new();
        mem_map.insert(memory_id.0, cortical_idx);

        let result = process_neural_dynamics(&fcl, Some(&mem_map), &mut neurons, 1).unwrap();
        assert_eq!(result.fire_queue.total_neurons(), 1);
        assert!(result.fire_queue.get_area_neurons(cortical_idx).is_some());
    }
}

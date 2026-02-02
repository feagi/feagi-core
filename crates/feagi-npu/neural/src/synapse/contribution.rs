// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Synaptic contribution calculation
//!
//! Pure functions for computing synaptic current contributions.

/// Synapse type (excitatory or inhibitory)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynapseType {
    Excitatory = 0,
    Inhibitory = 1,
}

/// Calculate synaptic contribution (platform-agnostic)
///
/// This is the core formula for synaptic transmission:
/// `contribution = weight × psp × sign`
///
/// Where:
/// - `weight`: Synaptic strength (0-255, direct cast to float, NO normalization)
/// - `psp`: Post-synaptic potential (0-255, direct cast to float, NO normalization)
/// - `sign`: +1.0 for excitatory, -1.0 for inhibitory
///
/// CRITICAL: This matches FEAGI's Python behavior - direct cast with NO division by 255.
/// Values range from 0.0 to 65,025.0 (255 × 255), NOT 0.0 to 1.0.
///
/// # Arguments
/// * `weight` - Synaptic weight (0-255)
/// * `psp` - Postsynaptic potential (0-255)
/// * `synapse_type` - Excitatory or inhibitory
///
/// # Returns
/// Synaptic contribution (positive for excitatory, negative for inhibitory)
///
/// # Example
/// ```
/// use feagi_npu_neural::synapse::{compute_synaptic_contribution, SynapseType};
///
/// let contribution = compute_synaptic_contribution(255, 255, SynapseType::Excitatory);
/// assert_eq!(contribution, 65025.0); // Maximum excitatory (255 × 255)
///
/// let contribution = compute_synaptic_contribution(255, 255, SynapseType::Inhibitory);
/// assert_eq!(contribution, -65025.0); // Maximum inhibitory
/// ```
#[inline]
pub fn compute_synaptic_contribution(weight: u8, psp: u8, synapse_type: SynapseType) -> f32 {
    // CRITICAL: Direct cast, NO normalization (matches Python .astype(np.float32))
    let w = weight as f32;
    let c = psp as f32;
    let sign = match synapse_type {
        SynapseType::Excitatory => 1.0,
        SynapseType::Inhibitory => -1.0,
    };
    w * c * sign
}

/// Batch compute synaptic contributions (SIMD-friendly)
///
/// Processes multiple synapses at once. Data layout is optimized for SIMD.
///
/// # Arguments
/// * `weights` - Slice of synaptic weights (0-255)
/// * `psps` - Slice of PSP values (0-255)
/// * `types` - Slice of synapse types (0=excitatory, 1=inhibitory)
/// * `contributions` - Output slice (mutable)
///
/// # Safety
/// All slices must have the same length.
///
/// # Example
/// ```
/// use feagi_npu_neural::synapse::compute_synaptic_contributions_batch;
///
/// let weights = [255, 128, 200];
/// let psps = [255, 255, 200];
/// let types = [0, 1, 0]; // excitatory, inhibitory, excitatory
/// let mut contributions = [0.0; 3];
///
/// compute_synaptic_contributions_batch(&weights, &psps, &types, &mut contributions);
///
/// assert!(contributions[0] > 0.0); // Excitatory
/// assert!(contributions[1] < 0.0); // Inhibitory
/// assert!(contributions[2] > 0.0); // Excitatory
/// ```
#[inline]
pub fn compute_synaptic_contributions_batch(
    weights: &[u8],
    psps: &[u8],
    types: &[u8],
    contributions: &mut [f32],
) {
    let count = weights.len();
    debug_assert_eq!(psps.len(), count);
    debug_assert_eq!(types.len(), count);
    debug_assert_eq!(contributions.len(), count);

    for i in 0..count {
        let synapse_type = if types[i] == 0 {
            SynapseType::Excitatory
        } else {
            SynapseType::Inhibitory
        };
        contributions[i] = compute_synaptic_contribution(weights[i], psps[i], synapse_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excitatory_contribution() {
        let contribution = compute_synaptic_contribution(255, 255, SynapseType::Excitatory);
        assert_eq!(contribution, 65025.0); // 255 × 255 (NO normalization)
    }

    #[test]
    fn test_inhibitory_contribution() {
        let contribution = compute_synaptic_contribution(255, 255, SynapseType::Inhibitory);
        assert_eq!(contribution, -65025.0); // -(255 × 255)
    }

    #[test]
    fn test_partial_weight() {
        let contribution = compute_synaptic_contribution(128, 255, SynapseType::Excitatory);
        assert_eq!(contribution, 128.0 * 255.0); // Direct multiplication
    }

    #[test]
    fn test_partial_psp() {
        let contribution = compute_synaptic_contribution(255, 128, SynapseType::Excitatory);
        assert_eq!(contribution, 255.0 * 128.0); // Direct multiplication
    }

    #[test]
    fn test_batch_computation() {
        let weights = [255, 128, 200];
        let psps = [255, 255, 200];
        let types = [0, 1, 0];
        let mut contributions = [0.0; 3];

        compute_synaptic_contributions_batch(&weights, &psps, &types, &mut contributions);

        assert!(contributions[0] > 0.0); // Excitatory
        assert!(contributions[1] < 0.0); // Inhibitory
        assert!(contributions[2] > 0.0); // Excitatory
    }
}

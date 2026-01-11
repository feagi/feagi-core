// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! # Synaptic Propagation Engine
//!
//! This module implements the core bottleneck identified in Python profiling:
//! computing synaptic contributions from fired neurons to their targets.
//!
//! ## Python Bottleneck Analysis
//! ```text
//! Phase 1 (Injection):  163.84 ms ( 88.7%)
//!   └─ Synaptic Propagation: 161.07 ms (100% of Phase 1)
//!      └─ Numpy Processing:  164.67 ms ( 91.7%)
//! ```
//!
//! ## Rust Optimization Strategy
//! 1. **Gather Phase**: Build synapse list (minimal Python loop overhead)
//! 2. **SIMD Phase**: Vectorized math (weight × conductance × sign)
//! 3. **Grouping Phase**: Sort/split by cortical area (np.argsort overhead removed)
//!
//! ## Performance Target
//! - Python: ~165ms for 12K neurons
//! - Rust Target: <3ms (50-100x speedup)

use ahash::AHashMap;
use feagi_npu_neural::types::*;
use feagi_npu_runtime::SynapseStorage;
use feagi_structures::genomic::cortical_area::CorticalID;
use rayon::prelude::*;
use std::sync::OnceLock;

// Use platform-agnostic synaptic algorithms (now in feagi-neural)
use feagi_npu_neural::synapse::{compute_synaptic_contribution, SynapseType as FeagiSynapseType};
use tracing::trace;

/// Runtime-gated tracing config for synaptic propagation.
/// Enable with:
/// - FEAGI_NPU_TRACE_SYNAPSE=1
///   Optional filters:
/// - FEAGI_NPU_TRACE_SRC=<u32 neuron_id>
/// - FEAGI_NPU_TRACE_DST=<u32 neuron_id>
struct SynapseTraceCfg {
    enabled: bool,
    src_filter: Option<u32>,
    dst_filter: Option<u32>,
}

fn synapse_trace_cfg() -> &'static SynapseTraceCfg {
    static CFG: OnceLock<SynapseTraceCfg> = OnceLock::new();
    CFG.get_or_init(|| {
        let enabled = std::env::var("FEAGI_NPU_TRACE_SYNAPSE")
            .ok()
            .as_deref()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let src_filter = std::env::var("FEAGI_NPU_TRACE_SRC").ok().and_then(|v| v.parse().ok());
        let dst_filter = std::env::var("FEAGI_NPU_TRACE_DST").ok().and_then(|v| v.parse().ok());

        SynapseTraceCfg {
            enabled,
            src_filter,
            dst_filter,
        }
    })
}

fn power_cortical_id() -> &'static CorticalID {
    static POWER: OnceLock<CorticalID> = OnceLock::new();
    POWER.get_or_init(|| {
        // "_power" is special-cased and stored as base64 in the genome parser.
        // See feagi-evolutionary parser docs; this value is stable.
        CorticalID::try_from_base_64("X19fcG93ZXI=")
            .expect("Power cortical ID base64 must be valid")
    })
}

/// Synapse lookup index: maps source neuron → list of synapse indices
pub type SynapseIndex = AHashMap<NeuronId, Vec<usize>>;

/// Propagation result: cortical area → list of (target_neuron, contribution)
pub type PropagationResult = AHashMap<CorticalID, Vec<(NeuronId, SynapticContribution)>>;

/// High-performance synaptic propagation engine
pub struct SynapticPropagationEngine {
    /// Pre-built index: source neuron → synapse indices
    pub synapse_index: SynapseIndex,
    /// Neuron → Cortical Area mapping
    pub neuron_to_area: AHashMap<NeuronId, CorticalID>,
    /// Cortical Area → mp_driven_psp flag mapping
    pub area_mp_driven_psp: AHashMap<CorticalID, bool>,
    /// Cortical Area → psp_uniform_distribution flag mapping
    /// When false: PSP is divided among all outgoing synapses
    /// When true: Full PSP value is applied to each synapse
    pub area_psp_uniform_distribution: AHashMap<CorticalID, bool>,
    /// Performance stats
    total_propagations: u64,
    total_synapses_processed: u64,
    /// Last propagation profile (timing + counts) for debugging performance spikes.
    last_profile: Option<PropagationProfile>,
}

/// Fine-grained profile of the last synaptic propagation call.
///
/// @cursor:critical-path - kept allocation-free aside from existing per-call collections.
#[derive(Clone, Debug)]
pub struct PropagationProfile {
    pub fired_neurons: usize,
    pub synapse_indices: usize,
    pub unique_sources: usize,
    pub contributions: usize,
    pub gather_ms: f64,
    pub metadata_ms: f64,
    pub compute_ms: f64,
    pub group_ms: f64,
    pub total_ms: f64,
    pub rayon_threads: usize,
}

impl SynapticPropagationEngine {
    /// Create a new propagation engine
    pub fn new() -> Self {
        Self {
            synapse_index: AHashMap::new(),
            neuron_to_area: AHashMap::new(),
            area_mp_driven_psp: AHashMap::new(),
            area_psp_uniform_distribution: AHashMap::new(),
            total_propagations: 0,
            total_synapses_processed: 0,
            last_profile: None,
        }
    }

    /// Returns the most recent propagation profile, if any.
    ///
    /// This is intended for performance diagnostics and is populated on each `propagate()` call.
    pub fn last_profile(&self) -> Option<&PropagationProfile> {
        self.last_profile.as_ref()
    }

    /// Build the synapse index from a synapse array (Structure-of-Arrays)
    /// This should be called once during initialization or when connectome changes
    ///
    /// ZERO-COPY: Works directly with StdSynapseArray without allocating intermediate structures
    pub fn build_synapse_index<S: SynapseStorage>(&mut self, synapse_storage: &S) {
        self.synapse_index.clear();

        for i in 0..synapse_storage.count() {
            if synapse_storage.valid_mask()[i] {
                let source = NeuronId(synapse_storage.source_neurons()[i]);
                self.synapse_index.entry(source).or_default().push(i);
            }
        }
    }

    /// Set the neuron-to-cortical-area mapping
    pub fn set_neuron_mapping(&mut self, mapping: AHashMap<NeuronId, CorticalID>) {
        self.neuron_to_area = mapping;
    }

    /// Set the mp_driven_psp flags for cortical areas
    /// When enabled for an area, PSP will be dynamically set from source neuron's membrane potential
    pub fn set_mp_driven_psp_flags(&mut self, flags: AHashMap<CorticalID, bool>) {
        self.area_mp_driven_psp = flags;
    }

    /// Update mp_driven_psp flag for a single cortical area (in-place).
    ///
    /// This avoids rebuilding/replacing the entire flags map when toggling one area.
    pub fn set_mp_driven_psp_flag(&mut self, cortical_id: CorticalID, enabled: bool) {
        self.area_mp_driven_psp.insert(cortical_id, enabled);
    }

    /// Set the psp_uniform_distribution flags for cortical areas
    /// When false (default): PSP value is divided among all outgoing synapses from the source neuron
    /// When true: Full PSP value is applied to each outgoing synapse
    pub fn set_psp_uniform_distribution_flags(&mut self, flags: AHashMap<CorticalID, bool>) {
        self.area_psp_uniform_distribution = flags;
    }

    /// Update psp_uniform_distribution flag for a single cortical area (in-place).
    ///
    /// This avoids rebuilding/replacing the entire flags map when toggling one area.
    pub fn set_psp_uniform_distribution_flag(&mut self, cortical_id: CorticalID, enabled: bool) {
        self.area_psp_uniform_distribution.insert(cortical_id, enabled);
    }

    /// Compute synaptic propagation for a set of fired neurons
    ///
    /// This is the MAIN PERFORMANCE-CRITICAL function that replaces the Python bottleneck.
    ///
    /// # Parameters
    /// - `fired_neurons`: List of neurons that fired this burst
    /// - `synapse_storage`: Synapse array (weights, PSPs, types)
    /// - `neuron_membrane_potentials`: Source neuron → membrane potential (0-255)
    ///   Used when `mp_driven_psp` is enabled for the source cortical area
    ///
    /// # Performance Notes
    /// - Uses Rayon for parallel processing
    /// - SIMD-friendly vectorized calculations
    /// - ZERO-COPY: Works directly with StdSynapseArray (no allocation overhead)
    /// - Cache-friendly data access patterns
    pub fn propagate(
        &mut self,
        fired_neurons: &[NeuronId],
        synapse_storage: &impl SynapseStorage,
        neuron_membrane_potentials: &AHashMap<NeuronId, u8>,
    ) -> Result<PropagationResult> {
        let total_start = std::time::Instant::now();
        self.total_propagations += 1;

        if fired_neurons.is_empty() {
            self.last_profile = Some(PropagationProfile {
                fired_neurons: 0,
                synapse_indices: 0,
                unique_sources: 0,
                contributions: 0,
                gather_ms: 0.0,
                metadata_ms: 0.0,
                compute_ms: 0.0,
                group_ms: 0.0,
                total_ms: 0.0,
                rayon_threads: rayon::current_num_threads(),
            });
            return Ok(AHashMap::new());
        }

        // PHASE 1: GATHER - Collect all synapse indices for fired neurons (parallel)
        let gather_start = std::time::Instant::now();
        let synapse_indices: Vec<usize> = fired_neurons
            .par_iter()
            .filter_map(|&neuron_id| self.synapse_index.get(&neuron_id))
            .flatten()
            .copied()
            .collect();
        let gather_ms = gather_start.elapsed().as_secs_f64() * 1000.0;

        if synapse_indices.is_empty() {
            self.last_profile = Some(PropagationProfile {
                fired_neurons: fired_neurons.len(),
                synapse_indices: 0,
                unique_sources: 0,
                contributions: 0,
                gather_ms,
                metadata_ms: 0.0,
                compute_ms: 0.0,
                group_ms: 0.0,
                total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
                rayon_threads: rayon::current_num_threads(),
            });
            return Ok(AHashMap::new());
        }

        let total_synapses = synapse_indices.len();
        self.total_synapses_processed += total_synapses as u64;

        // PRE-COMPUTE: Source neuron metadata (area, properties, synapse counts)
        // This eliminates 4 HashMap lookups per synapse in the hot loop
        struct SourceNeuronMetadata {
            area: CorticalID,
            mp_driven: bool,
            uniform: bool,
            synapse_count: usize,
        }
        
        let metadata_start = std::time::Instant::now();
        let source_metadata: AHashMap<NeuronId, SourceNeuronMetadata> = synapse_indices
            .par_iter()
            .map(|&syn_idx| NeuronId(synapse_storage.source_neurons()[syn_idx]))
            .fold(
                AHashMap::<NeuronId, (Option<CorticalID>, usize)>::new,
                |mut acc, source_id| {
                    let entry = acc.entry(source_id).or_insert_with(|| {
                        let area = self.neuron_to_area.get(&source_id).copied();
                        (area, 0)
                    });
                    entry.1 += 1; // Count synapses
                    acc
                },
            )
            .reduce(
                AHashMap::new,
                |mut a, b| {
                    for (id, (area, count)) in b {
                        let entry = a.entry(id).or_insert_with(|| (area, 0));
                        entry.1 += count;
                        if entry.0.is_none() {
                            entry.0 = area;
                        }
                    }
                    a
                },
            )
            .into_iter()
            .filter_map(|(source_id, (area_opt, synapse_count))| {
                let area = area_opt?;
                let mp_driven = self.area_mp_driven_psp.get(&area).copied().unwrap_or(false);
                let uniform = self.area_psp_uniform_distribution.get(&area).copied().unwrap_or(false);
                Some((
                    source_id,
                    SourceNeuronMetadata {
                        area,
                        mp_driven,
                        uniform,
                        synapse_count,
                    },
                ))
            })
            .collect();
        let metadata_ms = metadata_start.elapsed().as_secs_f64() * 1000.0;

        // PHASE 2: COMPUTE - Calculate contributions in parallel (TRUE SIMD!)
        // This is where Python spent 165ms doing inefficient numpy ops
        // ZERO-COPY: Access StdSynapseArray fields directly (Structure-of-Arrays)
        let compute_start = std::time::Instant::now();
        let contributions: Vec<(NeuronId, CorticalID, SynapticContribution)> = synapse_indices
            .par_iter()
            .filter_map(|&syn_idx| {
                // Skip invalid synapses (already filtered by build_synapse_index, but double-check)
                if !synapse_storage.valid_mask()[syn_idx] {
                    return None;
                }

                // Get target neuron from SoA
                let target_neuron = NeuronId(synapse_storage.target_neurons()[syn_idx]);

                // Get target cortical area (single lookup, can't optimize further - each synapse has unique target)
                let cortical_area = *self.neuron_to_area.get(&target_neuron)?;

                // Get source neuron
                let source_neuron = NeuronId(synapse_storage.source_neurons()[syn_idx]);

                // Get pre-computed source neuron metadata (eliminates 4 HashMap lookups per synapse!)
                let source_meta = source_metadata.get(&source_neuron)?;

                // Logging: exclude power sources to avoid noise (cortical_idx=1 maps to _power).
                let trace_cfg = synapse_trace_cfg();
                let allow_trace = trace_cfg.enabled
                    && source_meta.area != *power_cortical_id()
                    && trace_cfg
                        .src_filter
                        .map(|id| id == source_neuron.0)
                        .unwrap_or(true)
                    && trace_cfg
                        .dst_filter
                        .map(|id| id == target_neuron.0)
                        .unwrap_or(true);

                // Calculate base PSP: Use source neuron's MP if mp_driven_psp is enabled, else use static synapse PSP
                let base_psp = if source_meta.mp_driven {
                    // mp_driven_psp enabled: use source neuron's current membrane potential
                    *neuron_membrane_potentials.get(&source_neuron).unwrap_or_else(|| {
                        panic!(
                            "Invariant violation: missing membrane potential for source neuron {} (mp_driven_psp=true). Refusing fallback to 0.",
                            source_neuron.0
                        )
                    })
                } else {
                    // mp_driven_psp disabled: use static PSP from synapse
                    synapse_storage.postsynaptic_potentials()[syn_idx]
                };

                // Calculate base contribution using platform-agnostic function from feagi-synapse
                let weight = synapse_storage.weights()[syn_idx];
                let synapse_type = match synapse_storage.types()[syn_idx] {
                    0 => FeagiSynapseType::Excitatory,
                    _ => FeagiSynapseType::Inhibitory,
                };

                let base_contribution = compute_synaptic_contribution(weight, base_psp, synapse_type);

                // Apply PSP uniformity: divide CONTRIBUTION (not PSP) if uniformity is false
                // This preserves precision by doing float division instead of u8 integer division
                let final_contribution = if source_meta.uniform {
                    // PSP uniformity = true: Each synapse contributes full amount
                    base_contribution
                } else {
                    // PSP uniformity = false: Total contribution is divided among all outgoing synapses
                    if source_meta.synapse_count > 1 {
                        // Divide contribution by number of outgoing synapses (float division preserves precision!)
                        // Example: 1.0 / 10 = 0.1 (not 0 like u8 division would give)
                        base_contribution / source_meta.synapse_count as f32
                    } else {
                        base_contribution
                    }
                };

                if allow_trace {
                    trace!(
                        target: "feagi-npu-trace",
                        "[SYNAPSE] syn_idx={} src={} dst={} src_area={:?} dst_area={:?} type={:?} weight={} psp_used={} mp_driven={} uniform={} outgoing={} base_contrib={:.3} final_contrib={:.3}",
                        syn_idx,
                        source_neuron.0,
                        target_neuron.0,
                        source_meta.area,
                        cortical_area,
                        synapse_type,
                        weight,
                        base_psp,
                        source_meta.mp_driven,
                        source_meta.uniform,
                        source_meta.synapse_count,
                        base_contribution,
                        final_contribution
                    );
                }

                Some((target_neuron, cortical_area, SynapticContribution(final_contribution)))
            })
            .collect();
        let compute_ms = compute_start.elapsed().as_secs_f64() * 1000.0;

        // PHASE 3: GROUP - Group by cortical area (sequential, but very fast)
        // This replaces Python's slow dictionary building
        let group_start = std::time::Instant::now();
        let mut result: PropagationResult = AHashMap::new();
        for (target_neuron, cortical_area, contribution) in contributions {
            result
                .entry(cortical_area)
                .or_default()
                .push((target_neuron, contribution));
        }
        let group_ms = group_start.elapsed().as_secs_f64() * 1000.0;

        self.last_profile = Some(PropagationProfile {
            fired_neurons: fired_neurons.len(),
            synapse_indices: total_synapses,
            unique_sources: source_metadata.len(),
            contributions: result.values().map(|v| v.len()).sum(),
            gather_ms,
            metadata_ms,
            compute_ms,
            group_ms,
            total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
            rayon_threads: rayon::current_num_threads(),
        });

        Ok(result)
    }

    /// Get performance statistics
    pub fn stats(&self) -> (u64, u64) {
        (self.total_propagations, self.total_synapses_processed)
    }

    /// Reset performance statistics
    pub fn reset_stats(&mut self) {
        self.total_propagations = 0;
        self.total_synapses_processed = 0;
    }
}

impl Default for SynapticPropagationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use feagi_npu_runtime::StdSynapseArray;

    fn create_test_synapses() -> StdSynapseArray {
        let mut synapse_storage = StdSynapseArray {
            count: 3,
            source_neurons: vec![1, 1, 2],    // Raw u32 values
            target_neurons: vec![10, 11, 10], // Raw u32 values
            weights: vec![255, 128, 200],     // Raw u8 values
            postsynaptic_potentials: vec![255, 255, 200], // Raw u8 values (renamed from conductances)
            types: vec![0, 1, 0],                         // 0=excitatory, 1=inhibitory
            valid_mask: vec![true, true, true],
            source_index: ahash::AHashMap::new(),
        };

        // Build source index
        for i in 0..synapse_storage.count() {
            let source = synapse_storage.source_neurons()[i];
            synapse_storage
                .source_index
                .entry(source)
                .or_default()
                .push(i);
        }

        synapse_storage
    }

    #[test]
    fn test_synaptic_propagation() {
        let synapses = create_test_synapses();
        let mut engine = SynapticPropagationEngine::new();

        // Build index
        engine.build_synapse_index(&synapses);

        // Set neuron mapping
        use feagi_structures::genomic::cortical_area::CoreCorticalType;
        let mut mapping = AHashMap::new();
        mapping.insert(NeuronId(1), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(2), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(10), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(11), CoreCorticalType::Power.to_cortical_id());
        engine.set_neuron_mapping(mapping);

        // Propagate from neuron 1
        let fired = vec![NeuronId(1)];
        let neuron_mps = AHashMap::new(); // Empty MPs for this test
        let result = engine.propagate(&fired, &synapses, &neuron_mps).unwrap();

        // Should have 2 contributions in area 1
        assert_eq!(result.len(), 1);
        let area1_id = CoreCorticalType::Power.to_cortical_id();
        let area1_contributions = result.get(&area1_id).unwrap();
        assert_eq!(area1_contributions.len(), 2);

        // Check that both targets are present
        let targets: Vec<_> = area1_contributions.iter().map(|(n, _)| *n).collect();
        assert!(targets.contains(&NeuronId(10)));
        assert!(targets.contains(&NeuronId(11)));
    }

    #[test]
    fn test_parallel_propagation() {
        let synapses = create_test_synapses();
        let mut engine = SynapticPropagationEngine::new();
        engine.build_synapse_index(&synapses);

        use feagi_structures::genomic::cortical_area::CoreCorticalType;
        let mut mapping = AHashMap::new();
        mapping.insert(NeuronId(1), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(2), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(10), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(11), CoreCorticalType::Power.to_cortical_id());
        engine.set_neuron_mapping(mapping);

        // Propagate from multiple neurons in parallel
        let fired = vec![NeuronId(1), NeuronId(2)];
        let neuron_mps = AHashMap::new(); // Empty MPs for this test
        let result = engine.propagate(&fired, &synapses, &neuron_mps).unwrap();

        let area1_id = CoreCorticalType::Power.to_cortical_id();
        let area1_contributions = result.get(&area1_id).unwrap();
        assert_eq!(area1_contributions.len(), 3); // 2 from neuron 1, 1 from neuron 2
    }
}

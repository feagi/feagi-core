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
//! 2. **SIMD Phase**: Vectorized math (weight × PSP × sign)
//! 3. **Grouping Phase**: Sort/split by cortical_area area (np.argsort overhead removed)
//!
//! ## Performance Target
//! - Python: ~165ms for 12K neurons
//! - Rust Target: <3ms (50-100x speedup)

use ahash::{AHashMap, AHashSet};
use feagi_npu_neural::types::*;
use feagi_npu_runtime::SynapseStorage;
use feagi_genome_definitions::::CorticalID;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Once, OnceLock};

// Use platform-agnostic synaptic algorithms (now in feagi-neural)
use feagi_npu_neural::synapse::{
    compute_synaptic_contribution, SynapseType as FeagiSynapseType, SYNAPSE_EDGE_ASSOCIATIVE_MEMORY,
};
use tracing::debug;

/// Global id range for plasticity memory neurons (must match NPU / plasticity).
const MEMORY_NEURON_ID_START: u32 = 50_000_000;

/// Runtime-gated tracing config for synaptic propagation.
/// Enable with:
/// - `FEAGI_NPU_TRACE_SYNAPSE=1`
///
/// **Anti-spam:** at least one of:
/// - `FEAGI_NPU_TRACE_SRC=<u32>` — source neuron id
/// - `FEAGI_NPU_TRACE_DST=<u32>` — target neuron id
/// - `FEAGI_NPU_TRACE_CORTICAL_ID=<base64>` — target neuron's cortical_area area id (e.g. genome key)
///
/// Per-synapse lines require `FEAGI_NPU_TRACE_SYNAPSE_VERBOSE=1`. Otherwise one summary line per
/// propagation call counts matching edges (avoids log flooding).
///
/// When `FEAGI_NPU_TRACE_CORTICAL_ID` is set, synapses **from** `_power` are included if they
/// target that area (so power → area feedforward is visible).
///
/// ## @npu-debug-instrumentation (cleanup)
/// Verbose gating and summary line: remove or simplify after root-cause.
struct SynapseTraceCfg {
    enabled: bool,
    src_filter: Option<u32>,
    dst_filter: Option<u32>,
    /// Filter by postsynaptic cortical_area id (matches `neuron_to_area` for target neuron).
    dst_cortical_id_filter: Option<CorticalID>,
    /// When false, emit a single `[SYNAPSE]` summary count instead of per-edge lines.
    synapse_verbose: bool,
}

fn synapse_trace_cfg() -> &'static SynapseTraceCfg {
    static CFG: OnceLock<SynapseTraceCfg> = OnceLock::new();
    CFG.get_or_init(|| {
        let enabled = std::env::var("FEAGI_NPU_TRACE_SYNAPSE")
            .ok()
            .as_deref()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let src_filter = std::env::var("FEAGI_NPU_TRACE_SRC")
            .ok()
            .and_then(|v| v.parse().ok());
        let dst_filter = std::env::var("FEAGI_NPU_TRACE_DST")
            .ok()
            .and_then(|v| v.parse().ok());

        let dst_cortical_id_filter = std::env::var("FEAGI_NPU_TRACE_CORTICAL_ID")
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .and_then(|s| match CorticalID::try_from_base_64(&s) {
                Ok(id) => Some(id),
                Err(e) => {
                    tracing::warn!(
                        target: "feagi-npu-trace",
                        "FEAGI_NPU_TRACE_CORTICAL_ID is invalid ({}); cortical_area id filter disabled",
                        e
                    );
                    None
                }
            });

        let synapse_verbose = std::env::var("FEAGI_NPU_TRACE_SYNAPSE_VERBOSE")
            .ok()
            .as_deref()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let cfg = SynapseTraceCfg {
            enabled,
            src_filter,
            dst_filter,
            dst_cortical_id_filter,
            synapse_verbose,
        };

        static WARN_NO_FOCUS: Once = Once::new();
        if cfg.enabled
            && cfg.src_filter.is_none()
            && cfg.dst_filter.is_none()
            && cfg.dst_cortical_id_filter.is_none()
        {
            WARN_NO_FOCUS.call_once(|| {
                tracing::warn!(
                    target: "feagi-npu-trace",
                    "FEAGI_NPU_TRACE_SYNAPSE is set but neither FEAGI_NPU_TRACE_SRC, FEAGI_NPU_TRACE_DST, nor FEAGI_NPU_TRACE_CORTICAL_ID — synapse traces disabled (anti-spam)."
                );
            });
        }

        cfg
    })
}

fn power_cortical_id() -> &'static CorticalID {
    static POWER: OnceLock<CorticalID> = OnceLock::new();
    POWER.get_or_init(|| {
        // "_power" is special-cased and stored as base64 in the genome parser.
        // See feagi-evolutionary parser docs; this value is stable.
        CorticalID::try_from_base_64("X19fcG93ZXI=")
            .expect("Power cortical_area ID base64 must be valid")
    })
}

/// Synapse lookup index: maps source neuron → list of synapse indices
pub type SynapseIndex = AHashMap<NeuronId, Vec<usize>>;

/// Propagation result: cortical_area area → list of (target_neuron, contribution)
pub type PropagationResult = AHashMap<CorticalID, Vec<(NeuronId, SynapticContribution)>>;

/// PSP contributions and associative-memory PSP sums split by synaptic delay (whole bursts).
#[derive(Debug, Default, Clone)]
pub struct DelayedPropagationResult {
    /// `delay_bursts` → same shape as [`PropagationResult`].
    pub fcl_by_delay: AHashMap<u32, PropagationResult>,
    /// `delay_bursts` → memory-neuron id → associative PSP sum.
    pub memory_associative_by_delay: AHashMap<u32, AHashMap<u32, f32>>,
}

fn fold_contributions_by_delay(
    rows: Vec<(u32, NeuronId, CorticalID, SynapticContribution)>,
) -> AHashMap<u32, PropagationResult> {
    let mut by_delay: AHashMap<u32, PropagationResult> = AHashMap::new();
    for (delay, target, cortical, c) in rows {
        by_delay
            .entry(delay)
            .or_default()
            .entry(cortical)
            .or_default()
            .push((target, c));
    }
    by_delay
}

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
    /// Cortical Area -> configured baseline PSP for that source area.
    /// Used to restore per-synapse PSP on cortical_area runtime reset.
    pub area_postsynaptic_current: AHashMap<CorticalID, f32>,
    /// Cortical Area -> degeneration coefficient (PSP decrement per source fire)
    /// Values <= 0 are treated as disabled and removed from the map.
    pub area_degeneration: AHashMap<CorticalID, f32>,
    /// Conditional gate mappings: (src_area, dst_area) -> gate_area CorticalID.
    /// Synapses belonging to a gated mapping produce zero contribution when the
    /// gate area has no firing activity in the current burst (transistor semantics).
    gate_mappings: AHashMap<(CorticalID, CorticalID), CorticalID>,
    /// Set of (src_area, dst_area) pairs whose gate is currently closed (no activity
    /// in the gate area this burst). Updated before each propagation call by the NPU.
    closed_gates: AHashSet<(CorticalID, CorticalID)>,
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
            area_postsynaptic_current: AHashMap::new(),
            area_degeneration: AHashMap::new(),
            gate_mappings: AHashMap::new(),
            closed_gates: AHashSet::new(),
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

    /// Set the neuron-to-cortical_area-area mapping
    pub fn set_neuron_mapping(&mut self, mapping: AHashMap<NeuronId, CorticalID>) {
        self.neuron_to_area = mapping;
    }

    /// Set the mp_driven_psp flags for cortical_area areas
    /// When enabled for an area, PSP will be dynamically set from source neuron's membrane potential
    pub fn set_mp_driven_psp_flags(&mut self, flags: AHashMap<CorticalID, bool>) {
        self.area_mp_driven_psp = flags;
    }

    /// Update mp_driven_psp flag for a single cortical_area area (in-place).
    ///
    /// This avoids rebuilding/replacing the entire flags map when toggling one area.
    pub fn set_mp_driven_psp_flag(&mut self, cortical_id: CorticalID, enabled: bool) {
        self.area_mp_driven_psp.insert(cortical_id, enabled);
    }

    /// Set the psp_uniform_distribution flags for cortical_area areas
    /// When false (default): PSP value is divided among all outgoing synapses from the source neuron
    /// When true: Full PSP value is applied to each outgoing synapse
    pub fn set_psp_uniform_distribution_flags(&mut self, flags: AHashMap<CorticalID, bool>) {
        self.area_psp_uniform_distribution = flags;
    }

    /// Update psp_uniform_distribution flag for a single cortical_area area (in-place).
    ///
    /// This avoids rebuilding/replacing the entire flags map when toggling one area.
    pub fn set_psp_uniform_distribution_flag(&mut self, cortical_id: CorticalID, enabled: bool) {
        self.area_psp_uniform_distribution
            .insert(cortical_id, enabled);
    }

    /// Set configured baseline PSP values for cortical_area areas.
    ///
    /// Values <= 0 are retained as-is to preserve explicit user intent.
    pub fn set_postsynaptic_current_flags(&mut self, flags: AHashMap<CorticalID, f32>) {
        self.area_postsynaptic_current = flags;
    }

    /// Set configured baseline PSP for one cortical_area area.
    pub fn set_postsynaptic_current_flag(&mut self, cortical_id: CorticalID, postsynaptic: f32) {
        self.area_postsynaptic_current
            .insert(cortical_id, postsynaptic);
    }

    /// Set degeneration coefficients for cortical_area areas.
    ///
    /// Coefficients <= 0 disable degeneration for that area and are omitted.
    pub fn set_degeneration_flags(&mut self, mut flags: AHashMap<CorticalID, f32>) {
        flags.retain(|_, v| *v > 0.0);
        self.area_degeneration = flags;
    }

    /// Set degeneration coefficient for a single cortical_area area (in-place).
    ///
    /// Coefficients <= 0 disable degeneration for that area.
    pub fn set_degeneration_flag(&mut self, cortical_id: CorticalID, degeneration: f32) {
        if degeneration > 0.0 {
            self.area_degeneration.insert(cortical_id, degeneration);
        } else {
            self.area_degeneration.remove(&cortical_id);
        }
    }

    /// Register a conditional gate on a mapping: synapses from `src_area` to `dst_area`
    /// will produce zero contribution unless `gate_area` has firing activity in the
    /// current burst.
    pub fn register_gate_mapping(
        &mut self,
        src_area: CorticalID,
        dst_area: CorticalID,
        gate_area: CorticalID,
    ) {
        self.gate_mappings.insert((src_area, dst_area), gate_area);
    }

    /// Remove the conditional gate from a mapping.
    pub fn unregister_gate_mapping(&mut self, src_area: CorticalID, dst_area: CorticalID) {
        self.gate_mappings.remove(&(src_area, dst_area));
    }

    /// Returns true if any gate mappings are registered.
    pub fn has_gate_mappings(&self) -> bool {
        !self.gate_mappings.is_empty()
    }

    /// Returns a reference to the gate mappings for external gate-state computation.
    pub fn gate_mappings(&self) -> &AHashMap<(CorticalID, CorticalID), CorticalID> {
        &self.gate_mappings
    }

    /// Update the set of closed gates for the current burst. Called by the NPU before
    /// each propagation pass with the set of (src, dst) pairs whose gate area had no
    /// firing activity.
    pub fn set_closed_gates(&mut self, closed: AHashSet<(CorticalID, CorticalID)>) {
        self.closed_gates = closed;
    }

    /// Compute synaptic propagation for a set of fired neurons
    ///
    /// This is the MAIN PERFORMANCE-CRITICAL function that replaces the Python bottleneck.
    ///
    /// # Parameters
    /// - `fired_neurons`: List of neurons that fired this burst
    /// - `synapse_storage`: Synapse array (weights, PSPs, types)
    /// - `neuron_membrane_potentials`: Source neuron → firing-time membrane potential (`f32`)
    ///   Used when `mp_driven_psp` is enabled for the source cortical_area area
    ///
    /// # Performance Notes
    /// - Uses Rayon for parallel processing
    /// - SIMD-friendly vectorized calculations
    /// - ZERO-COPY: Works directly with StdSynapseArray (no allocation overhead)
    /// - Cache-friendly data access patterns
    ///
    /// `memory_associative_psp_out`: when `Some`, sums PSP from [`SYNAPSE_EDGE_ASSOCIATIVE_MEMORY`]
    /// synapses targeting memory-neuron ids (sparse associative LIF input; see `sparse_memory_lif`).
    ///
    /// Returns contributions split by per-synapse `delay_bursts` (see [`DelayedPropagationResult`]).
    pub fn propagate_delayed(
        &mut self,
        fired_neurons: &[NeuronId],
        synapse_storage: &impl SynapseStorage,
        neuron_membrane_potentials: &AHashMap<NeuronId, f32>,
    ) -> Result<DelayedPropagationResult> {
        let profile_enabled = tracing::enabled!(tracing::Level::DEBUG);
        let trace_cfg = synapse_trace_cfg();
        let synapse_verbose = trace_cfg.synapse_verbose;
        let has_focus = trace_cfg.src_filter.is_some()
            || trace_cfg.dst_filter.is_some()
            || trace_cfg.dst_cortical_id_filter.is_some();
        let total_start = profile_enabled.then(std::time::Instant::now);
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
            return Ok(DelayedPropagationResult::default());
        }

        // PHASE 1: GATHER - Collect all synapse indices for fired neurons (parallel)
        let gather_start = profile_enabled.then(std::time::Instant::now);
        let synapse_indices: Vec<usize> = fired_neurons
            .par_iter()
            .filter_map(|&neuron_id| self.synapse_index.get(&neuron_id))
            .flatten()
            .copied()
            .collect();
        let gather_ms = gather_start
            .map(|start| start.elapsed().as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

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
                total_ms: total_start
                    .map(|start| start.elapsed().as_secs_f64() * 1000.0)
                    .unwrap_or(0.0),
                rayon_threads: rayon::current_num_threads(),
            });
            return Ok(DelayedPropagationResult::default());
        }

        let total_synapses = synapse_indices.len();
        self.total_synapses_processed += total_synapses as u64;

        let use_fast_path = self.area_mp_driven_psp.is_empty()
            && self.area_psp_uniform_distribution.is_empty()
            && self.closed_gates.is_empty()
            && !trace_cfg.enabled;

        if use_fast_path {
            let compute_start = profile_enabled.then(std::time::Instant::now);
            let contributions: Vec<(u32, NeuronId, CorticalID, SynapticContribution)> =
                synapse_indices
                    .par_iter()
                    .filter_map(|&syn_idx| {
                        let target_neuron = NeuronId(synapse_storage.target_neurons()[syn_idx]);
                        let cortical_area = *self.neuron_to_area.get(&target_neuron)?;
                        let delay_bursts = synapse_storage.delay_bursts()[syn_idx].max(1) as u32;
                        let weight = synapse_storage.weights()[syn_idx];
                        let psp = synapse_storage.postsynaptic_potentials()[syn_idx];
                        let synapse_type = match synapse_storage.types()[syn_idx] {
                            0 => FeagiSynapseType::Excitatory,
                            _ => FeagiSynapseType::Inhibitory,
                        };

                        Some((
                            delay_bursts,
                            target_neuron,
                            cortical_area,
                            SynapticContribution(compute_synaptic_contribution(
                                weight,
                                psp,
                                synapse_type,
                            )),
                        ))
                    })
                    .collect();
            let compute_ms = compute_start
                .map(|start| start.elapsed().as_secs_f64() * 1000.0)
                .unwrap_or(0.0);

            let assoc_by_delay: AHashMap<u32, AHashMap<u32, f32>> = synapse_indices
                .par_iter()
                .filter_map(|&syn_idx| {
                    if !synapse_storage.valid_mask()[syn_idx] {
                        return None;
                    }
                    let target = NeuronId(synapse_storage.target_neurons()[syn_idx]);
                    if target.0 < MEMORY_NEURON_ID_START {
                        return None;
                    }
                    if (synapse_storage.edge_flags()[syn_idx] & SYNAPSE_EDGE_ASSOCIATIVE_MEMORY)
                        == 0
                    {
                        return None;
                    }
                    let delay_bursts = synapse_storage.delay_bursts()[syn_idx].max(1) as u32;
                    let weight = synapse_storage.weights()[syn_idx];
                    let psp = synapse_storage.postsynaptic_potentials()[syn_idx];
                    let synapse_type = match synapse_storage.types()[syn_idx] {
                        0 => FeagiSynapseType::Excitatory,
                        _ => FeagiSynapseType::Inhibitory,
                    };
                    let c = compute_synaptic_contribution(weight, psp, synapse_type);
                    Some((delay_bursts, target.0, c))
                })
                .fold(
                    AHashMap::<u32, AHashMap<u32, f32>>::new,
                    |mut acc, (delay, id, v)| {
                        *acc.entry(delay).or_default().entry(id).or_insert(0.0) += v;
                        acc
                    },
                )
                .reduce(AHashMap::<u32, AHashMap<u32, f32>>::new, |mut a, b| {
                    for (delay, inner) in b {
                        let e = a.entry(delay).or_default();
                        for (k, v) in inner {
                            *e.entry(k).or_insert(0.0) += v;
                        }
                    }
                    a
                });

            let group_start = profile_enabled.then(std::time::Instant::now);
            let fcl_by_delay = fold_contributions_by_delay(contributions);
            let group_ms = group_start
                .map(|start| start.elapsed().as_secs_f64() * 1000.0)
                .unwrap_or(0.0);

            let contrib_count: usize = fcl_by_delay
                .values()
                .map(|v| v.values().map(|x| x.len()).sum::<usize>())
                .sum();

            self.last_profile = Some(PropagationProfile {
                fired_neurons: fired_neurons.len(),
                synapse_indices: total_synapses,
                unique_sources: 0,
                contributions: contrib_count,
                gather_ms,
                metadata_ms: 0.0,
                compute_ms,
                group_ms,
                total_ms: total_start
                    .map(|start| start.elapsed().as_secs_f64() * 1000.0)
                    .unwrap_or(0.0),
                rayon_threads: rayon::current_num_threads(),
            });

            return Ok(DelayedPropagationResult {
                fcl_by_delay,
                memory_associative_by_delay: assoc_by_delay,
            });
        }

        // PRE-COMPUTE: Source neuron metadata (area, properties, synapse counts)
        // This eliminates 4 HashMap lookups per synapse in the hot loop
        struct SourceNeuronMetadata {
            area: CorticalID,
            mp_driven: bool,
            uniform: bool,
            synapse_count: usize,
        }

        let metadata_start = profile_enabled.then(std::time::Instant::now);
        let source_metadata: AHashMap<NeuronId, SourceNeuronMetadata> = synapse_indices
            .par_iter()
            // CRITICAL: `synapse_indices` comes from `self.synapse_index`, which is only rebuilt
            // via an explicit `rebuild_synapse_index()` call (see `remove_synapse` /
            // `remove_synapses_between` / `remove_synapses_from_sources_to_targets` docs). Between
            // a deletion and the next rebuild, stale (invalid) indices remain in the index. The
            // contribution loop below already skips invalid synapses via `valid_mask`, but this
            // outgoing-synapse count feeds the `psp_uniform_distribution = false` divisor, so it
            // must also exclude invalid synapses or surviving synapses get short-changed (PSP
            // divided by a stale, inflated count).
            .filter(|&&syn_idx| synapse_storage.valid_mask()[syn_idx])
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
            .reduce(AHashMap::new, |mut a, b| {
                for (id, (area, count)) in b {
                    let entry = a.entry(id).or_insert_with(|| (area, 0));
                    entry.1 += count;
                    if entry.0.is_none() {
                        entry.0 = area;
                    }
                }
                a
            })
            .into_iter()
            .filter_map(|(source_id, (area_opt, synapse_count))| {
                let area = area_opt?;
                let mp_driven = self.area_mp_driven_psp.get(&area).copied().unwrap_or(false);
                let uniform = self
                    .area_psp_uniform_distribution
                    .get(&area)
                    .copied()
                    .unwrap_or(false);
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
        let metadata_ms = metadata_start
            .map(|start| start.elapsed().as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        // PHASE 2: COMPUTE - Calculate contributions in parallel (TRUE SIMD!)
        // This is where Python spent 165ms doing inefficient numpy ops
        // ZERO-COPY: Access StdSynapseArray fields directly (Structure-of-Arrays)
        let compute_start = profile_enabled.then(std::time::Instant::now);
        let synapse_match_count = AtomicUsize::new(0);
        let contributions: Vec<(u32, NeuronId, CorticalID, SynapticContribution)> = synapse_indices
            .par_iter()
            .filter_map(|&syn_idx| {
                // Skip invalid synapses (already filtered by build_synapse_index, but double-check)
                if !synapse_storage.valid_mask()[syn_idx] {
                    return None;
                }

                let delay_bursts = synapse_storage.delay_bursts()[syn_idx].max(1) as u32;

                // Get target neuron from SoA
                let target_neuron = NeuronId(synapse_storage.target_neurons()[syn_idx]);

                // Get target cortical_area area (single lookup, can't optimize further - each synapse has unique target)
                let cortical_area = *self.neuron_to_area.get(&target_neuron)?;

                // Get source neuron
                let source_neuron = NeuronId(synapse_storage.source_neurons()[syn_idx]);

                // Get pre-computed source neuron metadata (eliminates 4 HashMap lookups per synapse!)
                let source_meta = source_metadata.get(&source_neuron)?;

                // Conditional gate check: if this (src_area, dst_area) pair has a closed
                // gate, the synapse contributes nothing (transistor OFF state).
                if self.closed_gates.contains(&(source_meta.area, cortical_area)) {
                    return None;
                }

                // Logging: exclude power sources unless tracing a destination cortical_area area (then
                // power→area drive is often relevant).
                let has_focus = trace_cfg.src_filter.is_some()
                    || trace_cfg.dst_filter.is_some()
                    || trace_cfg.dst_cortical_id_filter.is_some();
                let dst_cortical_ok = trace_cfg
                    .dst_cortical_id_filter
                    .as_ref()
                    .map(|id| *id == cortical_area)
                    .unwrap_or(true);
                let power_src = source_meta.area == *power_cortical_id();
                let allow_power_src = trace_cfg.dst_cortical_id_filter.is_some() && dst_cortical_ok;
                let allow_trace = trace_cfg.enabled
                    && has_focus
                    && dst_cortical_ok
                    && (!power_src || allow_power_src)
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
                    if synapse_verbose {
                        debug!(
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
                    } else {
                        synapse_match_count.fetch_add(1, Ordering::Relaxed);
                    }
                }

                Some((
                    delay_bursts,
                    target_neuron,
                    cortical_area,
                    SynapticContribution(final_contribution),
                ))
            })
            .collect();

        let assoc_by_delay: AHashMap<u32, AHashMap<u32, f32>> = synapse_indices
            .par_iter()
            .filter_map(|&syn_idx| {
                if !synapse_storage.valid_mask()[syn_idx] {
                    return None;
                }
                let target_neuron = NeuronId(synapse_storage.target_neurons()[syn_idx]);
                if target_neuron.0 < MEMORY_NEURON_ID_START {
                    return None;
                }
                if (synapse_storage.edge_flags()[syn_idx] & SYNAPSE_EDGE_ASSOCIATIVE_MEMORY) == 0 {
                    return None;
                }
                let target_area = self.neuron_to_area.get(&target_neuron)?;
                let delay_bursts = synapse_storage.delay_bursts()[syn_idx].max(1) as u32;
                let source_neuron = NeuronId(synapse_storage.source_neurons()[syn_idx]);
                let source_meta = source_metadata.get(&source_neuron)?;

                if self.closed_gates.contains(&(source_meta.area, *target_area)) {
                    return None;
                }

                let base_psp = if source_meta.mp_driven {
                    *neuron_membrane_potentials.get(&source_neuron).unwrap_or_else(|| {
                        panic!(
                            "Invariant violation: missing membrane potential for source neuron {} (mp_driven_psp=true). Refusing fallback to 0.",
                            source_neuron.0
                        )
                    })
                } else {
                    synapse_storage.postsynaptic_potentials()[syn_idx]
                };
                let weight = synapse_storage.weights()[syn_idx];
                let synapse_type = match synapse_storage.types()[syn_idx] {
                    0 => FeagiSynapseType::Excitatory,
                    _ => FeagiSynapseType::Inhibitory,
                };
                let base_contribution =
                    compute_synaptic_contribution(weight, base_psp, synapse_type);
                let final_contribution = if source_meta.uniform {
                    base_contribution
                } else if source_meta.synapse_count > 1 {
                    base_contribution / source_meta.synapse_count as f32
                } else {
                    base_contribution
                };
                Some((delay_bursts, target_neuron.0, final_contribution))
            })
            .fold(
                AHashMap::<u32, AHashMap<u32, f32>>::new,
                |mut acc, (delay, id, v)| {
                    *acc.entry(delay).or_default().entry(id).or_insert(0.0) += v;
                    acc
                },
            )
            .reduce(
                AHashMap::<u32, AHashMap<u32, f32>>::new,
                |mut a, b| {
                    for (delay, inner) in b {
                        let e = a.entry(delay).or_default();
                        for (k, v) in inner {
                            *e.entry(k).or_insert(0.0) += v;
                        }
                    }
                    a
                },
            );

        if trace_cfg.enabled && has_focus && !synapse_verbose {
            let n = synapse_match_count.load(Ordering::Relaxed);
            if n > 0 {
                debug!(
                    target: "feagi-npu-trace",
                    "[SYNAPSE] synapses_matching_focus count={}",
                    n
                );
            }
        }
        let compute_ms = compute_start
            .map(|start| start.elapsed().as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        // PHASE 3: GROUP - Group by delay then cortical_area area
        let group_start = profile_enabled.then(std::time::Instant::now);
        let fcl_by_delay = fold_contributions_by_delay(contributions);
        let contrib_count: usize = fcl_by_delay
            .values()
            .map(|v| v.values().map(|x| x.len()).sum::<usize>())
            .sum();
        let group_ms = group_start
            .map(|start| start.elapsed().as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        self.last_profile = Some(PropagationProfile {
            fired_neurons: fired_neurons.len(),
            synapse_indices: total_synapses,
            unique_sources: source_metadata.len(),
            contributions: contrib_count,
            gather_ms,
            metadata_ms,
            compute_ms,
            group_ms,
            total_ms: total_start
                .map(|start| start.elapsed().as_secs_f64() * 1000.0)
                .unwrap_or(0.0),
            rayon_threads: rayon::current_num_threads(),
        });

        Ok(DelayedPropagationResult {
            fcl_by_delay,
            memory_associative_by_delay: assoc_by_delay,
        })
    }

    /// Legacy single-burst (`delay_bursts == 1`) projection for tests and callers that expect
    /// the pre-delay [`PropagationResult`] shape.
    pub fn propagate(
        &mut self,
        fired_neurons: &[NeuronId],
        synapse_storage: &impl SynapseStorage,
        neuron_membrane_potentials: &AHashMap<NeuronId, f32>,
        memory_associative_psp_out: Option<&mut AHashMap<u32, f32>>,
    ) -> Result<PropagationResult> {
        let mut delayed =
            self.propagate_delayed(fired_neurons, synapse_storage, neuron_membrane_potentials)?;
        if let Some(out) = memory_associative_psp_out {
            out.clear();
            if let Some(m) = delayed.memory_associative_by_delay.remove(&1) {
                for (k, v) in m {
                    out.insert(k, v);
                }
            }
        }
        Ok(delayed.fcl_by_delay.remove(&1).unwrap_or_default())
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
            weights: vec![255.0, 128.0, 200.0],
            postsynaptic_potentials: vec![255.0, 255.0, 200.0],
            types: vec![0, 1, 0], // 0=excitatory, 1=inhibitory
            edge_flags: vec![0, 0, 0],
            delay_bursts: vec![1, 1, 1],
            valid_mask: vec![true, true, true],
            eligibility_traces: vec![0.0, 0.0, 0.0],
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
        use feagi_genome_definitions::::CoreCorticalType;
        let mut mapping = AHashMap::new();
        mapping.insert(NeuronId(1), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(2), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(10), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(11), CoreCorticalType::Power.to_cortical_id());
        engine.set_neuron_mapping(mapping);

        // Propagate from neuron 1
        let fired = vec![NeuronId(1)];
        let neuron_mps = AHashMap::new(); // Empty MPs for this test
        let result = engine
            .propagate(&fired, &synapses, &neuron_mps, None)
            .unwrap();

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

        use feagi_genome_definitions::::CoreCorticalType;
        let mut mapping = AHashMap::new();
        mapping.insert(NeuronId(1), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(2), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(10), CoreCorticalType::Power.to_cortical_id());
        mapping.insert(NeuronId(11), CoreCorticalType::Power.to_cortical_id());
        engine.set_neuron_mapping(mapping);

        // Propagate from multiple neurons in parallel
        let fired = vec![NeuronId(1), NeuronId(2)];
        let neuron_mps = AHashMap::new(); // Empty MPs for this test
        let result = engine
            .propagate(&fired, &synapses, &neuron_mps, None)
            .unwrap();

        let area1_id = CoreCorticalType::Power.to_cortical_id();
        let area1_contributions = result.get(&area1_id).unwrap();
        assert_eq!(area1_contributions.len(), 3); // 2 from neuron 1, 1 from neuron 2
    }
}

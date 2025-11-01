/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Complete Rust NPU
//!
//! Integrates all burst processing phases into a single high-performance NPU.
//!
//! ## Architecture
//! ```text
//! ┌─────────────────────────────────────┐
//! │ RustNPU                            │
//! ├─────────────────────────────────────┤
//! │ - NeuronArray                      │
//! │ - SynapseArray                     │
//! │ - FireCandidateList (FCL)          │
//! │ - FireQueue (current & previous)   │
//! │ - FireLedger                       │
//! │ - SynapticPropagationEngine        │
//! └─────────────────────────────────────┘
//!          ↓
//!     process_burst()
//!          ↓
//! Phase 1: Injection → Phase 2: Dynamics → Phase 3: Archival → Phase 5: Cleanup
//! ```

use crate::fire_ledger::RustFireLedger;
use crate::fire_structures::FireQueue;
use crate::fq_sampler::{FQSampler, SamplingMode};
use crate::neural_dynamics::*;
use crate::synaptic_propagation::SynapticPropagationEngine;
use ahash::AHashMap;
use feagi_types::*;
use tracing::{debug, info, warn, error};

/// Burst processing result
#[derive(Debug, Clone)]
pub struct BurstResult {
    /// Neurons that fired this burst
    pub fired_neurons: Vec<NeuronId>,

    /// Number of neurons that fired
    pub neuron_count: usize,

    /// Burst number
    pub burst: u64,

    /// Performance metrics
    pub power_injections: usize,
    pub synaptic_injections: usize,
    pub neurons_processed: usize,
    pub neurons_in_refractory: usize,
}

/// Complete Rust Neural Processing Unit with Fine-Grained Locking
/// 
/// ## Locking Strategy (Performance-Critical)
/// 
/// This structure uses fine-grained locking to enable concurrent operations:
/// 
/// - **RwLock<NeuronArray>**: Multiple readers (burst processing reads many neurons),
///   exclusive writer (neurogenesis, parameter updates)
/// - **RwLock<SynapseArray>**: Multiple readers (burst processing reads synapses),
///   exclusive writer (synaptogenesis, plasticity)
/// - **Mutex<FireStructures>**: Exclusive access (FCL clear, FQ swap, sensory injection)
/// - **AtomicU64**: Lock-free stats (burst_count, neuron_count, etc.)
/// 
/// ## Benefits
/// 
/// - Sensory injection locks only fire structures, not neurons/synapses
/// - Burst processing can read neurons while sensory data is being injected
/// - Visualization can sample fire queue without blocking burst processing
/// - Stats queries never block any operation
/// 
/// ## Multi-Core Performance
/// 
/// With 30 Hz burst rate + 30 FPS video injection:
/// - **Before**: All operations serialized on one mutex (API unresponsive)
/// - **After**: Concurrent sensory injection + burst processing + API queries
pub struct RustNPU {
    // Core data structures (RwLock: many readers, one writer)
    pub(crate) neuron_array: std::sync::RwLock<NeuronArray>,
    pub(crate) synapse_array: std::sync::RwLock<SynapseArray>,

    // Fire structures (Mutex: exclusive access for FCL/FQ operations)
    pub(crate) fire_structures: std::sync::Mutex<FireStructures>,

    // Cortical area mapping (RwLock: frequent reads, rare writes)
    pub(crate) area_id_to_name: std::sync::RwLock<AHashMap<u32, String>>,

    // Propagation engine (RwLock: burst reads, rare updates)
    pub(crate) propagation_engine: std::sync::RwLock<SynapticPropagationEngine>,

    // Compute backend (Mutex: exclusive access during burst processing)
    // This is the CPU/GPU backend that processes bursts
    // TODO: Integrate backend into process_burst() method to replace direct CPU code
    #[allow(dead_code)]  // Will be used when backend integration is complete
    pub(crate) backend: std::sync::Mutex<Box<dyn crate::backend::ComputeBackend>>,

    // Atomic stats (lock-free reads)
    burst_count: std::sync::atomic::AtomicU64,
    
    // Configuration (AtomicU32 for f32 as u32 bits)
    power_amount: std::sync::atomic::AtomicU32, // f32::to_bits()
}

/// Fire-related structures grouped together for single mutex
pub(crate) struct FireStructures {
    pub(crate) fire_candidate_list: FireCandidateList,
    pub(crate) current_fire_queue: FireQueue,
    pub(crate) previous_fire_queue: FireQueue,
    pub(crate) fire_ledger: RustFireLedger,
    pub(crate) fq_sampler: FQSampler,
    pub(crate) pending_sensory_injections: Vec<(NeuronId, f32)>,
    pub(crate) last_fcl_snapshot: Vec<(NeuronId, f32)>,
}

impl RustNPU {
    /// Create a new Rust NPU with specified capacities
    ///
    /// # Arguments
    /// * `neuron_capacity` - Maximum number of neurons
    /// * `synapse_capacity` - Maximum number of synapses
    /// * `fire_ledger_window` - Fire ledger history window size
    /// * `gpu_config` - Optional GPU configuration (None = default to CPU)
    pub fn new(
        neuron_capacity: usize,
        synapse_capacity: usize,
        fire_ledger_window: usize,
        gpu_config: Option<&crate::backend::GpuConfig>,
    ) -> Self {
        use tracing::info;
        
        // Determine backend based on GPU config
        let (backend_type, backend_config) = if let Some(config) = gpu_config {
            info!("🎮 GPU Configuration:");
            info!("   GPU enabled: {}", config.use_gpu);
            info!("   Hybrid mode: {}", config.hybrid_enabled);
            info!("   GPU threshold: {} synapses", config.gpu_threshold);
            info!("   GPU memory fraction: {:.1}%", config.gpu_memory_fraction * 100.0);
            config.to_backend_selection()
        } else {
            info!("   No GPU config provided, using CPU backend");
            (crate::backend::BackendType::CPU, crate::backend::BackendConfig::default())
        };
        
        info!("   Creating backend: {}", backend_type);
        
        // Create backend
        let backend = crate::backend::create_backend(
            backend_type,
            neuron_capacity,
            synapse_capacity,
            &backend_config,
        ).expect("Failed to create compute backend");
        
        info!("   ✓ Backend selected: {}", backend.backend_name());
        
        Self {
            neuron_array: std::sync::RwLock::new(NeuronArray::new(neuron_capacity)),
            synapse_array: std::sync::RwLock::new(SynapseArray::new(synapse_capacity)),
            fire_structures: std::sync::Mutex::new(FireStructures {
                fire_candidate_list: FireCandidateList::new(),
                current_fire_queue: FireQueue::new(),
                previous_fire_queue: FireQueue::new(),
                fire_ledger: RustFireLedger::new(fire_ledger_window),
                fq_sampler: FQSampler::new(1000.0, SamplingMode::Unified),
                pending_sensory_injections: Vec::with_capacity(10000),
                last_fcl_snapshot: Vec::new(),
            }),
            area_id_to_name: std::sync::RwLock::new(AHashMap::new()),
            propagation_engine: std::sync::RwLock::new(SynapticPropagationEngine::new()),
            backend: std::sync::Mutex::new(backend),
            burst_count: std::sync::atomic::AtomicU64::new(0),
            power_amount: std::sync::atomic::AtomicU32::new(1.0f32.to_bits()),
        }
    }

    /// Set power injection amount (lock-free atomic operation)
    pub fn set_power_amount(&self, amount: f32) {
        self.power_amount.store(amount.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Get power injection amount (lock-free atomic operation)
    pub fn get_power_amount(&self) -> f32 {
        f32::from_bits(self.power_amount.load(std::sync::atomic::Ordering::Relaxed))
    }
    
    /// Get burst count (lock-free atomic operation)
    pub fn get_burst_count(&self) -> u64 {
        self.burst_count.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Increment burst count (lock-free atomic operation)
    fn increment_burst_count(&self) -> u64 {
        self.burst_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    /// Add a neuron to the NPU (LIF model with genome leak only)
    pub fn add_neuron(
        &mut self,
        threshold: f32,
        leak_coefficient: f32,
        resting_potential: f32,
        neuron_type: i32,
        refractory_period: u16,
        excitability: f32,
        consecutive_fire_limit: u16,
        snooze_period: u16,
        mp_charge_accumulation: bool,
        cortical_area: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Result<NeuronId> {
        let neuron_id = self.neuron_array.write().unwrap().add_neuron(
            threshold,
            leak_coefficient,
            resting_potential,
            neuron_type,
            refractory_period,
            excitability,
            consecutive_fire_limit,
            snooze_period,
            mp_charge_accumulation,
            cortical_area,
            x,
            y,
            z,
        )?;

        // CRITICAL: Add to propagation engine's neuron-to-area mapping
        self.propagation_engine
            .write().unwrap()
            .neuron_to_area
            .insert(neuron_id, CorticalAreaId(cortical_area));

        Ok(neuron_id)
    }

    /// Batch add neurons (optimized for neurogenesis)
    ///
    /// Creates multiple neurons in a single operation with optimal performance.
    /// This is 50-100x faster than calling add_neuron() in a loop.
    ///
    /// Performance benefits:
    /// - Single function call overhead (vs N calls)
    /// - Single lock acquisition (vs N locks from Python)
    /// - Contiguous SoA memory writes
    /// - Batch propagation engine updates
    ///
    /// Returns: (neuron_ids, failed_indices)
    pub fn add_neurons_batch(
        &mut self,
        thresholds: Vec<f32>,
        leak_coefficients: Vec<f32>,
        resting_potentials: Vec<f32>,
        neuron_types: Vec<i32>,
        refractory_periods: Vec<u16>,
        excitabilities: Vec<f32>,
        consecutive_fire_limits: Vec<u16>,
        snooze_periods: Vec<u16>,
        mp_charge_accumulations: Vec<bool>,
        cortical_areas: Vec<u32>,
        x_coords: Vec<u32>,
        y_coords: Vec<u32>,
        z_coords: Vec<u32>,
    ) -> (u32, Vec<usize>) {
        let n = x_coords.len();

        // Call the TRUE batch method on neuron_array (100-1000x faster!)
        match self.neuron_array.write().unwrap().add_neurons_batch(
            &thresholds,
            &leak_coefficients,
            &resting_potentials,
            &neuron_types,
            &refractory_periods,
            &excitabilities,
            &consecutive_fire_limits,
            &snooze_periods,
            &mp_charge_accumulations,
            &cortical_areas,
            &x_coords,
            &y_coords,
            &z_coords,
        ) {
            Ok(neuron_ids) => {
                // BULK update propagation engine's neuron-to-area mapping
                // Reserve capacity upfront to minimize rehashing
                use std::time::Instant;
                let prop_start = Instant::now();
                self.propagation_engine.write().unwrap().neuron_to_area.reserve(n);
                let reserve_time = prop_start.elapsed();

                let insert_start = Instant::now();
                for (i, neuron_id) in neuron_ids.iter().enumerate() {
                    self.propagation_engine
                        .write().unwrap().neuron_to_area
                        .insert(*neuron_id, CorticalAreaId(cortical_areas[i]));
                }
                let insert_time = insert_start.elapsed();

                debug!(
                    n,
                    reserve_ns = reserve_time.as_nanos(),
                    inserts_ns = insert_time.as_nanos(),
                    mapping_size = self.propagation_engine.write().unwrap().neuron_to_area.len(),
                    "[PROP-ENGINE] Neuron-to-area mapping updated"
                );

                // ✅ ARCHITECTURE FIX: Return only success COUNT, not full Vec<u32> of IDs
                // Python doesn't need IDs - Rust owns all neuron data!
                // This eliminates expensive PyO3 Vec→list conversion (was 4s bottleneck!)
                (neuron_ids.len() as u32, Vec::new())
            }
            Err(_) => {
                // All failed - return 0 success count and all indices as failed
                (0, (0..n).collect())
            }
        }
    }

    /// Create neurons for a cortical area with uniform properties
    ///
    /// This is the CORRECT architecture - Python passes only scalars, Rust generates everything
    ///
    /// # Arguments
    /// * `cortical_idx` - Cortical area index
    /// * `width` - X dimension
    /// * `height` - Y dimension  
    /// * `depth` - Z dimension
    /// * `neurons_per_voxel` - Neurons per spatial position
    /// * `default_threshold` - Default firing threshold
    /// * `default_leak_coefficient` - Default leak rate
    /// * `default_resting_potential` - Default resting potential
    /// * `default_neuron_type` - Default neuron type
    /// * `default_refractory_period` - Default refractory period
    /// * `default_excitability` - Default excitability
    /// * `default_consecutive_fire_limit` - Default consecutive fire limit
    /// * `default_snooze_period` - Default snooze period
    /// * `default_mp_charge_accumulation` - Default MP charge accumulation flag
    ///
    /// # Returns
    /// * `Ok(count)` - Number of neurons created
    /// * `Err` - If capacity exceeded or other error
    pub fn create_cortical_area_neurons(
        &mut self,
        cortical_idx: u32,
        width: u32,
        height: u32,
        depth: u32,
        neurons_per_voxel: u32,
        default_threshold: f32,
        default_leak_coefficient: f32,
        default_resting_potential: f32,
        default_neuron_type: i32,
        default_refractory_period: u16,
        default_excitability: f32,
        default_consecutive_fire_limit: u16,
        default_snooze_period: u16,
        default_mp_charge_accumulation: bool,
    ) -> Result<u32> {
        use std::time::Instant;
        let fn_start = Instant::now();

        // Calculate total neurons
        let total_neurons = (width * height * depth * neurons_per_voxel) as usize;

        // Performance diagnostic - only visible with --debug flag
        debug!(
            cortical_idx,
            total_neurons,
            "[NEUROGENESIS] Creating neurons for cortical area"
        );

        if total_neurons == 0 {
            return Ok(0);
        }

        let alloc_start = Instant::now();
        // ✅ SIMD-OPTIMIZED: Fill uniform values with bulk operations (LLVM auto-vectorizes!)
        let thresholds = vec![default_threshold; total_neurons];
        let leak_coefficients = vec![default_leak_coefficient; total_neurons];
        let resting_potentials = vec![default_resting_potential; total_neurons];
        let neuron_types = vec![default_neuron_type; total_neurons];
        let refractory_periods = vec![default_refractory_period; total_neurons];
        let excitabilities = vec![default_excitability; total_neurons];
        let consecutive_fire_limits = vec![default_consecutive_fire_limit; total_neurons];
        let snooze_periods = vec![default_snooze_period; total_neurons];
        let mp_charge_accumulations = vec![default_mp_charge_accumulation; total_neurons];
        let cortical_areas = vec![cortical_idx; total_neurons];

        // ✅ OPTIMIZED: Pre-size coordinate vectors, fill with direct indexing (no bounds checking!)
        let mut x_coords = vec![0u32; total_neurons];
        let mut y_coords = vec![0u32; total_neurons];
        let mut z_coords = vec![0u32; total_neurons];

        // Generate coordinates in cache-friendly order with direct writes
        let mut idx = 0;
        for x in 0..width {
            for y in 0..height {
                for z in 0..depth {
                    for _ in 0..neurons_per_voxel {
                        x_coords[idx] = x;
                        y_coords[idx] = y;
                        z_coords[idx] = z;
                        idx += 1;
                    }
                }
            }
        }
        let alloc_time = alloc_start.elapsed();

        let batch_start = Instant::now();
        // Call existing batch creation (already optimized with SIMD)
        let (success_count, failed) = self.add_neurons_batch(
            thresholds,
            leak_coefficients,
            resting_potentials,
            neuron_types,
            refractory_periods,
            excitabilities,
            consecutive_fire_limits,
            snooze_periods,
            mp_charge_accumulations,
            cortical_areas,
            x_coords,
            y_coords,
            z_coords,
        );

        let batch_time = batch_start.elapsed();
        let total_time = fn_start.elapsed();

        // Performance metrics - only visible with --debug flag
        debug!(
            total_neurons,
            alloc_us = alloc_time.as_micros(),
            batch_us = batch_time.as_micros(),
            total_us = total_time.as_micros(),
            "[NEUROGENESIS] Neuron creation timing"
        );

        if !failed.is_empty() {
            return Err(FeagiError::ComputationError(format!(
                "Failed to create {} neurons",
                failed.len()
            )));
        }

        Ok(success_count)
    }

    /// Add a synapse to the NPU
    pub fn add_synapse(
        &mut self,
        source: NeuronId,
        target: NeuronId,
        weight: SynapticWeight,
        conductance: SynapticConductance,
        synapse_type: SynapseType,
    ) -> Result<usize> {
        self.synapse_array
            .write().unwrap()
            .add_synapse(source, target, weight, conductance, synapse_type)
    }

    /// Batch add synapses (SIMD-optimized)
    ///
    /// Creates multiple synapses in a single operation with optimal performance.
    /// This is 50-100x faster than calling add_synapse() in a loop.
    ///
    /// Performance:
    /// - Single function call overhead (vs N calls)
    /// - Contiguous SoA memory writes
    /// - Batch source_index updates
    ///
    /// Returns: (successful_count, failed_indices)
    pub fn add_synapses_batch(
        &mut self,
        sources: Vec<NeuronId>,
        targets: Vec<NeuronId>,
        weights: Vec<SynapticWeight>,
        postsynaptic_potentials: Vec<SynapticConductance>,  // TODO: Rename type to SynapticPSP
        synapse_types: Vec<SynapseType>,
    ) -> (usize, Vec<usize>) {
        // Convert NeuronId/Weight types to raw u32/u8 for SynapseArray
        let source_ids: Vec<u32> = sources.iter().map(|n| n.0).collect();
        let target_ids: Vec<u32> = targets.iter().map(|n| n.0).collect();
        let weight_vals: Vec<u8> = weights.iter().map(|w| w.0).collect();
        let psp_vals: Vec<u8> = postsynaptic_potentials.iter().map(|c| c.0).collect();
        let type_vals: Vec<u8> = synapse_types
            .iter()
            .map(|t| match t {
                SynapseType::Excitatory => 0,
                SynapseType::Inhibitory => 1,
            })
            .collect();

        self.synapse_array.write().unwrap().add_synapses_batch(
            &source_ids,
            &target_ids,
            &weight_vals,
            &psp_vals,
            &type_vals,
        )
    }

    /// Remove a synapse
    pub fn remove_synapse(&mut self, source: NeuronId, target: NeuronId) -> bool {
        self.synapse_array.write().unwrap().remove_synapse(source, target)
    }

    /// Batch remove all synapses from specified source neurons (SIMD-optimized)
    ///
    /// Performance: 50-100x faster than individual deletions for cortical mapping removal
    /// Returns: number of synapses deleted
    pub fn remove_synapses_from_sources(&mut self, sources: Vec<NeuronId>) -> usize {
        let source_ids: Vec<u32> = sources.iter().map(|n| n.0).collect();
        self.synapse_array.write().unwrap().remove_synapses_from_sources(&source_ids)
    }

    /// Batch remove synapses between source and target neuron sets (SIMD-optimized)
    ///
    /// Uses bit-vector filtering for O(1) target membership testing.
    /// Optimal for both few→many and many→many deletion patterns.
    ///
    /// Performance: 20-100x faster than nested loops
    /// Returns: number of synapses deleted
    pub fn remove_synapses_between(
        &mut self,
        sources: Vec<NeuronId>,
        targets: Vec<NeuronId>,
    ) -> usize {
        let source_ids: Vec<u32> = sources.iter().map(|n| n.0).collect();
        let target_ids: Vec<u32> = targets.iter().map(|n| n.0).collect();
        self.synapse_array
            .write().unwrap()
            .remove_synapses_between(&source_ids, &target_ids)
    }

    /// Update synapse weight
    pub fn update_synapse_weight(
        &mut self,
        source: NeuronId,
        target: NeuronId,
        new_weight: SynapticWeight,
    ) -> bool {
        self.synapse_array.write().unwrap().update_weight(source, target, new_weight)
    }

    /// Rebuild indexes after modifications (call after bulk modifications)
    pub fn rebuild_indexes(&mut self) {
        // ZERO-COPY: Pass synapse_array by reference
        let synapse_array_read = self.synapse_array.read().unwrap();
        self.propagation_engine
            .write().unwrap()
            .build_synapse_index(&*synapse_array_read);
    }

    /// Set neuron to cortical area mapping for propagation engine
    pub fn set_neuron_mapping(&mut self, mapping: AHashMap<NeuronId, CorticalAreaId>) {
        self.propagation_engine.write().unwrap().set_neuron_mapping(mapping);
    }

    // ===== SENSORY INJECTION API =====

    /// Inject sensory neurons into FCL (called from Rust sensory threads)
    /// This is the PRIMARY method for Rust-native sensory injection
    pub fn inject_sensory_batch(&mut self, neuron_ids: &[NeuronId], potential: f32) {
        // 🔍 DEBUG: Log first batch injection
        static FIRST_BATCH_LOGGED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !FIRST_BATCH_LOGGED.load(std::sync::atomic::Ordering::Relaxed) && !neuron_ids.is_empty()
        {
            debug!(
                "[NPU-INJECT] 🔍 First batch: count={}, potential={}",
                neuron_ids.len(),
                potential
            );
            info!(
                "[NPU-INJECT]    First 5 NeuronIds: {:?}",
                &neuron_ids[0..neuron_ids.len().min(5)]
            );
            info!(
                "[NPU-INJECT]    FCL size before: {}",
                self.fire_structures.lock().unwrap().fire_candidate_list.len()
            );
            FIRST_BATCH_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        for &neuron_id in neuron_ids {
            self.fire_structures.lock().unwrap().fire_candidate_list.add_candidate(neuron_id, potential);
        }

        // 🔍 DEBUG: Log FCL size after first injection
        static FIRST_BATCH_AFTER_LOGGED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !FIRST_BATCH_AFTER_LOGGED.load(std::sync::atomic::Ordering::Relaxed)
            && !neuron_ids.is_empty()
        {
            info!(
                "[NPU-INJECT]    FCL size after: {}",
                self.fire_structures.lock().unwrap().fire_candidate_list.len()
            );
            FIRST_BATCH_AFTER_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Stage sensory neurons for next burst (thread-safe, prevents FCL clear race)
    /// XYZP data from agents is staged here and injected AFTER fcl.clear() in Phase 1
    pub fn inject_sensory_with_potentials(&mut self, neurons: &[(NeuronId, f32)]) {
        let mut fire_structures = self.fire_structures.lock().unwrap();
        if let Some(pending) = Some(&mut fire_structures.pending_sensory_injections) {
            pending.extend_from_slice(neurons);

            // 🔍 DEBUG: Log first staging
            static FIRST_STAGING_LOGGED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !FIRST_STAGING_LOGGED.load(std::sync::atomic::Ordering::Relaxed)
                && !neurons.is_empty()
            {
                info!("[NPU-STAGE] 🎯 Staged {} sensory neurons for next burst (prevents FCL clear race)", neurons.len());
                info!(
                    "[NPU-STAGE]    Queue now has {} pending injections",
                    pending.len()
                );
                FIRST_STAGING_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    /// Get a clone of the FCL for inspection (debugging only)
    pub fn get_fcl_clone(&self) -> FireCandidateList {
        self.fire_structures.lock().unwrap().fire_candidate_list.clone()
    }

    /// Get last FCL snapshot (captured before clear in previous burst)
    /// Returns Vec of (NeuronId, potential) pairs
    pub fn get_last_fcl_snapshot(&self) -> Vec<(NeuronId, f32)> {
        self.fire_structures.lock().unwrap().last_fcl_snapshot.clone()
    }

    // ===== END SENSORY INJECTION API =====

    // ===== POWER INJECTION =====
    // Power neurons are identified by cortical_idx = 1 in the neuron array
    // No separate list needed - single source of truth!

    /// Process a single burst (MAIN METHOD - FINE-GRAINED LOCKING)
    ///
    /// This is the complete neural processing pipeline:
    /// Phase 1: Injection → Phase 2: Dynamics → Phase 3: Archival →
    /// Phase 4: Queue Swap → Phase 5: FQ Sampling → Phase 6: Cleanup
    ///
    /// 🔋 Power neurons are auto-discovered from neuron_array (cortical_idx = 1)
    ///
    /// ## Fine-Grained Locking Strategy:
    /// - Neuron/synapse arrays: RwLock (concurrent reads during propagation)
    /// - Fire structures: Mutex (exclusive for FCL/FQ operations)
    /// - Burst count: Atomic (lock-free)
    pub fn process_burst(&self) -> Result<BurstResult> {
        let burst_count = self.increment_burst_count();
        let power_amount = self.get_power_amount();

        // Lock neuron/synapse arrays for reading (allows concurrent sensory injection to fire_structures)
        let mut neuron_array = self.neuron_array.write().unwrap();
        let synapse_array = self.synapse_array.read().unwrap();
        let mut propagation_engine = self.propagation_engine.write().unwrap();
        
        // Lock fire structures (FCL, FQ, Fire Ledger)
        let mut fire_structures = self.fire_structures.lock().unwrap();

        // Phase 1: Injection (power + synaptic propagation + staged sensory)
        // Clone previous_fire_queue to avoid multiple borrows
        let previous_fq = fire_structures.previous_fire_queue.clone();
        let pending_mutex = std::sync::Mutex::new(fire_structures.pending_sensory_injections.clone());
        let injection_result = phase1_injection_with_synapses(
            &mut fire_structures.fire_candidate_list,
            &mut neuron_array,
            &mut propagation_engine,
            &previous_fq,
            power_amount,
            &synapse_array,
            &pending_mutex,
        )?;
        fire_structures.pending_sensory_injections = pending_mutex.into_inner().unwrap();

        // Phase 2: Neural Dynamics (membrane potential updates, threshold checks, firing)
        let dynamics_result = process_neural_dynamics(
            &fire_structures.fire_candidate_list,
            &mut neuron_array,
            burst_count,
        )?;

        // Phase 3: Archival (ZERO-COPY archive to Fire Ledger)
        fire_structures.fire_ledger
            .archive_burst(burst_count, &dynamics_result.fire_queue);

        // Phase 4: Swap fire queues (current becomes previous for next burst)
        fire_structures.previous_fire_queue = fire_structures.current_fire_queue.clone();
        fire_structures.current_fire_queue = dynamics_result.fire_queue.clone();

        // Phase 5: Sample fire queue for visualization (FQ Sampler)
        let current_fq_clone = fire_structures.current_fire_queue.clone();
        fire_structures.fq_sampler.sample(&current_fq_clone);

        // Phase 6: Cleanup (snapshot FCL before clearing for API access)
        fire_structures.last_fcl_snapshot = fire_structures.fire_candidate_list.get_all_candidates();
        fire_structures.fire_candidate_list.clear();

        // Build result
        let fired_neurons = fire_structures.current_fire_queue.get_all_neuron_ids();

        Ok(BurstResult {
            neuron_count: fired_neurons.len(),
            fired_neurons,
            burst: burst_count,
            power_injections: injection_result.power_injections,
            synaptic_injections: injection_result.synaptic_injections,
            neurons_processed: dynamics_result.neurons_processed,
            neurons_in_refractory: dynamics_result.neurons_in_refractory,
        })
    }

    // Removed duplicate - using atomic version at line 147

    /// Register a cortical area name for visualization encoding
    /// This mapping is populated during neuroembryogenesis
    pub fn register_cortical_area(&mut self, area_id: u32, cortical_name: String) {
        self.area_id_to_name.write().unwrap().insert(area_id, cortical_name);
    }

    /// Get the cortical area name for a given area_id
    /// Returns None if the area_id is not registered
    pub fn get_cortical_area_name(&self, area_id: u32) -> Option<String> {
        self.area_id_to_name.read().unwrap().get(&area_id).cloned()
    }


    /// Get the cortical area ID for a given cortical name
    /// Returns None if the name is not registered
    pub fn get_cortical_area_id(&self, cortical_name: &str) -> Option<u32> {
        let area_map = self.area_id_to_name.read().unwrap();
        for (&area_id, name) in area_map.iter() {
            if name == cortical_name {
                return Some(area_id);
            }
        }
        None
    }

    // ===== PUBLIC ACCESSORS FOR PYTHON BINDINGS =====

    /// Get neuron at specific coordinate (for Python bindings)
    pub fn get_neuron_id_at_coordinate(&self, cortical_area: u32, x: u32, y: u32, z: u32) -> Option<u32> {
        self.neuron_array.read().unwrap()
            .get_neuron_at_coordinate(cortical_area, x, y, z)
            .map(|id| id.0)
    }

    /// Get neuron property by index (for Python bindings)
    pub fn get_neuron_property_by_index(&self, idx: usize, property: &str) -> Option<f32> {
        let neuron_array = self.neuron_array.read().unwrap();
        if idx >= neuron_array.count {
            return None;
        }
        match property {
            "threshold" => neuron_array.thresholds.get(idx).copied(),
            "leak_coefficient" => neuron_array.leak_coefficients.get(idx).copied(),
            "membrane_potential" => neuron_array.membrane_potentials.get(idx).copied(),
            "resting_potential" => neuron_array.resting_potentials.get(idx).copied(),
            "excitability" => neuron_array.excitabilities.get(idx).copied(),
            _ => None,
        }
    }

    /// Get neuron property u16 by index (for Python bindings)
    pub fn get_neuron_property_u16_by_index(&self, idx: usize, property: &str) -> Option<u16> {
        let neuron_array = self.neuron_array.read().unwrap();
        if idx >= neuron_array.count {
            return None;
        }
        match property {
            "refractory_period" => neuron_array.refractory_periods.get(idx).copied(),
            "consecutive_fire_limit" => neuron_array.consecutive_fire_limits.get(idx).copied(),
            _ => None,
        }
    }

    /// Get neuron array snapshot for FCL inspection (for Python bindings)
    pub fn get_neuron_array_snapshot(&self) -> (usize, Vec<u32>, Vec<bool>) {
        let neuron_array = self.neuron_array.read().unwrap();
        (
            neuron_array.count,
            neuron_array.cortical_areas.clone(),
            neuron_array.valid_mask.clone(),
        )
    }

    /// Get the number of registered cortical areas
    pub fn get_registered_cortical_area_count(&self) -> usize {
        self.area_id_to_name.read().unwrap().len()
    }

    /// Get all registered cortical areas as (idx, name) pairs
    pub fn get_all_cortical_areas(&self) -> Vec<(u32, String)> {
        self.area_id_to_name
            .read().unwrap()
            .iter()
            .map(|(&idx, name)| (idx, name.clone()))
            .collect()
    }

    /// Find neuron ID at specific X,Y,Z coordinates within a cortical area
    /// Returns None if no neuron exists at that position
    pub fn get_neuron_at_coordinates(
        &self,
        cortical_area: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Option<NeuronId> {
        for neuron_idx in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[neuron_idx]
                && self.neuron_array.read().unwrap().cortical_areas[neuron_idx] == cortical_area
            {
                let coord_idx = neuron_idx * 3;
                if self.neuron_array.read().unwrap().coordinates[coord_idx] == x
                    && self.neuron_array.read().unwrap().coordinates[coord_idx + 1] == y
                    && self.neuron_array.read().unwrap().coordinates[coord_idx + 2] == z
                {
                    return Some(NeuronId(neuron_idx as u32));
                }
            }
        }
        None
    }

    /// Inject sensory neurons using cortical area name and XYZ coordinates
    /// This is the high-level API for sensory injection from agents
    pub fn inject_sensory_xyzp(
        &mut self,
        cortical_name: &str,
        xyzp_data: &[(u32, u32, u32, f32)],
    ) -> usize {
        // Find cortical area ID
        let cortical_area = match self.get_cortical_area_id(cortical_name) {
            Some(id) => id,
            None => {
                error!("[NPU] ❌ Unknown cortical area: '{}'", cortical_name);
                error!(
                    "[NPU] ❌ Available cortical areas: {:?}",
                    self.area_id_to_name.read().unwrap().values().collect::<Vec<_>>()
                );
                error!("[NPU] ❌ Total registered: {}", self.area_id_to_name.read().unwrap().len());
                return 0;
            }
        };

        // Convert XYZ coordinates to neuron IDs
        let mut neuron_potential_pairs = Vec::with_capacity(xyzp_data.len());
        let mut found_count = 0;

        for &(x, y, z, potential) in xyzp_data {
            if let Some(neuron_id) = self.get_neuron_at_coordinates(cortical_area, x, y, z) {
                neuron_potential_pairs.push((neuron_id, potential));
                found_count += 1;
            }
        }

        // Inject found neurons
        if !neuron_potential_pairs.is_empty() {
            self.inject_sensory_with_potentials(&neuron_potential_pairs);
        }

        found_count
    }

    /// Export connectome snapshot (for saving to file)
    ///
    /// This captures the complete NPU state including all neurons, synapses,
    /// and runtime state for serialization.
    pub fn export_connectome(&self) -> feagi_connectome_serialization::ConnectomeSnapshot {
        use feagi_connectome_serialization::{
            ConnectomeMetadata, ConnectomeSnapshot, SerializableNeuronArray,
            SerializableSynapseArray,
        };

        // Convert neuron array (lock once and clone all fields)
        let neuron_array = self.neuron_array.read().unwrap();
        let neurons = SerializableNeuronArray {
            count: neuron_array.count,
            capacity: neuron_array.capacity,
            membrane_potentials: neuron_array.membrane_potentials.clone(),
            thresholds: neuron_array.thresholds.clone(),
            leak_coefficients: neuron_array.leak_coefficients.clone(),
            resting_potentials: neuron_array.resting_potentials.clone(),
            neuron_types: neuron_array.neuron_types.clone(),
            refractory_periods: neuron_array.refractory_periods.clone(),
            refractory_countdowns: neuron_array.refractory_countdowns.clone(),
            excitabilities: neuron_array.excitabilities.clone(),
            cortical_areas: neuron_array.cortical_areas.clone(),
            coordinates: neuron_array.coordinates.clone(),
            valid_mask: neuron_array.valid_mask.clone(),
        };
        drop(neuron_array);  // Release lock

        // Convert synapse array (lock once and clone all fields)
        let synapse_array = self.synapse_array.read().unwrap();
        let synapses = SerializableSynapseArray {
            count: synapse_array.count,
            capacity: synapse_array.capacity,
            source_neurons: synapse_array.source_neurons.clone(),
            target_neurons: synapse_array.target_neurons.clone(),
            weights: synapse_array.weights.clone(),
            conductances: synapse_array.postsynaptic_potentials.clone(),
            types: synapse_array.types.clone(),
            valid_mask: synapse_array.valid_mask.clone(),
            source_index: synapse_array.source_index.clone(),
        };
        drop(synapse_array);  // Release lock

        ConnectomeSnapshot {
            version: 1,
            neurons,
            synapses,
            cortical_area_names: self.area_id_to_name.read().unwrap().clone(),
            burst_count: self.get_burst_count(),
            power_amount: self.get_power_amount(),
            fire_ledger_window: 20, // Default value (fire_ledger doesn't expose window)
            metadata: ConnectomeMetadata::default(),
        }
    }

    /// Import connectome snapshot (for loading from file)
    ///
    /// This replaces the entire NPU state with data from a saved connectome.
    /// Import a connectome from a snapshot
    ///
    /// # Arguments
    /// * `snapshot` - The connectome snapshot to import
    ///
    /// # Note
    /// This method uses CPU backend by default for backward compatibility.
    /// Use `import_connectome_with_config()` to specify GPU configuration.
    pub fn import_connectome(snapshot: feagi_connectome_serialization::ConnectomeSnapshot) -> Self {
        Self::import_connectome_with_config(snapshot, None)
    }
    
    /// Import a connectome from a snapshot with optional GPU configuration
    ///
    /// # Arguments
    /// * `snapshot` - The connectome snapshot to import
    /// * `gpu_config` - Optional GPU configuration (None = default to CPU)
    pub fn import_connectome_with_config(
        snapshot: feagi_connectome_serialization::ConnectomeSnapshot,
        gpu_config: Option<&crate::backend::GpuConfig>,
    ) -> Self {
        use tracing::info;
        
        // Convert neuron array
        let mut neuron_array = NeuronArray::new(snapshot.neurons.capacity);
        neuron_array.count = snapshot.neurons.count;
        neuron_array.membrane_potentials = snapshot.neurons.membrane_potentials;
        neuron_array.thresholds = snapshot.neurons.thresholds;
        neuron_array.leak_coefficients = snapshot.neurons.leak_coefficients;
        neuron_array.resting_potentials = snapshot.neurons.resting_potentials;
        neuron_array.neuron_types = snapshot.neurons.neuron_types;
        neuron_array.refractory_periods = snapshot.neurons.refractory_periods;
        neuron_array.refractory_countdowns = snapshot.neurons.refractory_countdowns;
        neuron_array.excitabilities = snapshot.neurons.excitabilities;
        neuron_array.cortical_areas = snapshot.neurons.cortical_areas;
        neuron_array.coordinates = snapshot.neurons.coordinates;
        neuron_array.valid_mask = snapshot.neurons.valid_mask;

        // Convert synapse array
        let mut synapse_array = SynapseArray::new(snapshot.synapses.capacity);
        synapse_array.count = snapshot.synapses.count;
        synapse_array.source_neurons = snapshot.synapses.source_neurons;
        synapse_array.target_neurons = snapshot.synapses.target_neurons;
        synapse_array.weights = snapshot.synapses.weights;
        synapse_array.postsynaptic_potentials = snapshot.synapses.conductances;  // TODO: Rename field in snapshot
        synapse_array.types = snapshot.synapses.types;
        synapse_array.valid_mask = snapshot.synapses.valid_mask;
        synapse_array.source_index = snapshot.synapses.source_index;
        
        // Create backend based on GPU config and actual genome size
        let (backend_type, backend_config) = if let Some(config) = gpu_config {
            info!("🎮 Imported Connectome GPU Configuration:");
            info!("   Neurons: {}, Synapses: {}", neuron_array.count, synapse_array.count);
            info!("   GPU enabled: {}", config.use_gpu);
            info!("   Hybrid mode: {}", config.hybrid_enabled);
            if config.hybrid_enabled {
                info!("   GPU threshold: {} synapses", config.gpu_threshold);
                if synapse_array.count >= config.gpu_threshold {
                    info!("   → Genome ABOVE threshold, GPU will be considered");
                } else {
                    info!("   → Genome BELOW threshold, CPU will be used");
                }
            }
            config.to_backend_selection()
        } else {
            (crate::backend::BackendType::CPU, crate::backend::BackendConfig::default())
        };
        
        // Create backend
        let backend = crate::backend::create_backend(
            backend_type,
            snapshot.neurons.capacity,
            snapshot.synapses.capacity,
            &backend_config,
        ).expect("Failed to create compute backend");
        
        info!("   ✓ Backend created: {}", backend.backend_name());

        Self {
            neuron_array: std::sync::RwLock::new(neuron_array),
            synapse_array: std::sync::RwLock::new(synapse_array),
            fire_structures: std::sync::Mutex::new(FireStructures {
                fire_candidate_list: FireCandidateList::new(),
                current_fire_queue: FireQueue::new(),
                previous_fire_queue: FireQueue::new(),
                fire_ledger: RustFireLedger::new(snapshot.fire_ledger_window),
                fq_sampler: FQSampler::new(1000.0, SamplingMode::Unified),
                pending_sensory_injections: Vec::with_capacity(10000),
                last_fcl_snapshot: Vec::new(),
            }),
            area_id_to_name: std::sync::RwLock::new(snapshot.cortical_area_names),
            propagation_engine: std::sync::RwLock::new(SynapticPropagationEngine::new()),
            backend: std::sync::Mutex::new(backend),
            burst_count: std::sync::atomic::AtomicU64::new(snapshot.burst_count),
            power_amount: std::sync::atomic::AtomicU32::new(snapshot.power_amount.to_bits()),
        }
    }

    /// Get all neuron positions for a cortical area (for fast batch lookups)
    /// Returns Vec<(neuron_id, x, y, z)>
    pub fn get_neuron_positions_in_cortical_area(
        &self,
        cortical_area: u32,
    ) -> Vec<(u32, u32, u32, u32)> {
        let mut positions = Vec::new();

        for neuron_id in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[neuron_id]
                && self.neuron_array.read().unwrap().cortical_areas[neuron_id] == cortical_area
            {
                // Coordinates stored as flat array: [x0, y0, z0, x1, y1, z1, ...]
                let coord_idx = neuron_id * 3;
                positions.push((
                    neuron_id as u32,
                    self.neuron_array.read().unwrap().coordinates[coord_idx],
                    self.neuron_array.read().unwrap().coordinates[coord_idx + 1],
                    self.neuron_array.read().unwrap().coordinates[coord_idx + 2],
                ));
            }
        }

        positions
    }

    /// Update excitability for a single neuron (for live parameter changes)
    /// Returns true if successful, false if neuron doesn't exist
    pub fn update_neuron_excitability(&mut self, neuron_id: u32, excitability: f32) -> bool {
        let idx = neuron_id as usize;
        if idx >= self.neuron_array.read().unwrap().count || !self.neuron_array.read().unwrap().valid_mask[idx] {
            return false;
        }

        self.neuron_array.write().unwrap().excitabilities[idx] = excitability.clamp(0.0, 1.0);
        true
    }
    
    /// Update firing threshold for a specific neuron
    /// Returns true if successful, false if neuron doesn't exist
    pub fn update_neuron_threshold(&mut self, neuron_id: u32, threshold: f32) -> bool {
        let idx = neuron_id as usize;
        if idx >= self.neuron_array.read().unwrap().count || !self.neuron_array.read().unwrap().valid_mask[idx] {
            return false;
        }

        self.neuron_array.write().unwrap().thresholds[idx] = threshold;
        true
    }
    
    /// Update leak coefficient for a specific neuron
    /// Returns true if successful, false if neuron doesn't exist
    pub fn update_neuron_leak(&mut self, neuron_id: u32, leak_coefficient: f32) -> bool {
        let idx = neuron_id as usize;
        if idx >= self.neuron_array.read().unwrap().count || !self.neuron_array.read().unwrap().valid_mask[idx] {
            return false;
        }

        self.neuron_array.write().unwrap().leak_coefficients[idx] = leak_coefficient.clamp(0.0, 1.0);
        true
    }
    
    /// Update resting potential for a specific neuron
    /// Returns true if successful, false if neuron doesn't exist
    pub fn update_neuron_resting_potential(&mut self, neuron_id: u32, resting_potential: f32) -> bool {
        let idx = neuron_id as usize;
        if idx >= self.neuron_array.read().unwrap().count || !self.neuron_array.read().unwrap().valid_mask[idx] {
            return false;
        }

        self.neuron_array.write().unwrap().resting_potentials[idx] = resting_potential;
        true
    }

    /// Update excitability for all neurons in a cortical area (for bulk parameter changes)
    /// Returns number of neurons updated
    pub fn update_cortical_area_excitability(
        &mut self,
        cortical_area: u32,
        excitability: f32,
    ) -> usize {
        let clamped_excitability = excitability.clamp(0.0, 1.0);
        let mut updated_count = 0;

        // CRITICAL: Iterate by ARRAY INDEX (not neuron_id!)
        for idx in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[idx]
                && self.neuron_array.read().unwrap().cortical_areas[idx] == cortical_area
            {
                self.neuron_array.write().unwrap().excitabilities[idx] = clamped_excitability;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Update refractory period for all neurons in a cortical area (bulk parameter change)
    pub fn update_cortical_area_refractory_period(
        &mut self,
        cortical_area: u32,
        refractory_period: u16,
    ) -> usize {
        info!("[RUST-UPDATE] update_cortical_area_refractory_period: cortical_area={}, refractory_period={}", 
                 cortical_area, refractory_period);

        let mut updated_count = 0;

        // CRITICAL: Iterate by ARRAY INDEX (not neuron_id!)
        for idx in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[idx]
                && self.neuron_array.read().unwrap().cortical_areas[idx] == cortical_area
            {
                // Get the actual neuron_id for this array index
                let neuron_id = self.neuron_array.read().unwrap().index_to_neuron_id[idx];

                // Update base refractory period (used when neuron fires)
                self.neuron_array.write().unwrap().refractory_periods[idx] = refractory_period;

                // CRITICAL FIX: Do NOT set countdown here!
                // The countdown should only be set AFTER a neuron fires.
                // Setting it now would block the neuron immediately, which is backward.
                //
                // Correct behavior:
                // 1. Neuron fires → countdown = refractory_period
                // 2. Next burst: countdown > 0 → BLOCKED
                // 3. Decrement countdown each burst
                // 4. When countdown = 0 → neuron can fire again
                //
                // If we set countdown=refractory_period NOW (before firing),
                // the neuron would be blocked for N bursts FIRST, then fire.
                // That's backward!

                // Only clear countdown if setting refractory to 0 (allow immediate firing)
                if refractory_period == 0 {
                    self.neuron_array.write().unwrap().refractory_countdowns[idx] = 0;
                }

                // Reset consecutive fire count when applying a new period to avoid
                // stale state causing unexpected immediate extended refractory.
                self.neuron_array.write().unwrap().consecutive_fire_counts[idx] = 0;

                updated_count += 1;

                // Log first few neurons (show actual neuron_id, not array index!)
                if updated_count <= 3 {
                    info!(
                        "[RUST-BATCH-UPDATE]   Neuron {}: refractory_period={}, countdown={}",
                        neuron_id, refractory_period, self.neuron_array.read().unwrap().refractory_countdowns[idx]
                    );
                }
            }
        }

        updated_count
    }

    /// Update threshold for all neurons in a cortical area (bulk parameter change)
    pub fn update_cortical_area_threshold(&mut self, cortical_area: u32, threshold: f32) -> usize {
        let mut updated_count = 0;

        // CRITICAL: Iterate by ARRAY INDEX (not neuron_id!)
        for idx in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[idx]
                && self.neuron_array.read().unwrap().cortical_areas[idx] == cortical_area
            {
                self.neuron_array.write().unwrap().thresholds[idx] = threshold;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Update leak coefficient for all neurons in a cortical area (bulk parameter change)
    pub fn update_cortical_area_leak(&mut self, cortical_area: u32, leak: f32) -> usize {
        let mut updated_count = 0;

        // CRITICAL: Iterate by ARRAY INDEX (not neuron_id!)
        for idx in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[idx]
                && self.neuron_array.read().unwrap().cortical_areas[idx] == cortical_area
            {
                self.neuron_array.write().unwrap().leak_coefficients[idx] = leak;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Update consecutive fire limit for all neurons in a cortical area (bulk parameter change)
    pub fn update_cortical_area_consecutive_fire_limit(
        &mut self,
        cortical_area: u32,
        limit: u16,
    ) -> usize {
        let mut updated_count = 0;

        // CRITICAL: Iterate by ARRAY INDEX (not neuron_id!)
        for idx in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[idx]
                && self.neuron_array.read().unwrap().cortical_areas[idx] == cortical_area
            {
                self.neuron_array.write().unwrap().consecutive_fire_limits[idx] = limit;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Update snooze period (extended refractory) for all neurons in a cortical area (bulk parameter change)
    pub fn update_cortical_area_snooze_period(
        &mut self,
        cortical_area: u32,
        snooze_period: u16,
    ) -> usize {
        let mut updated_count = 0;

        // CRITICAL: Iterate by ARRAY INDEX (not neuron_id!)
        for idx in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[idx]
                && self.neuron_array.read().unwrap().cortical_areas[idx] == cortical_area
            {
                self.neuron_array.write().unwrap().snooze_periods[idx] = snooze_period;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update refractory period for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_refractory_period(&mut self, neuron_ids: &[u32], values: &[u16]) -> usize {
        info!(
            "[RUST-BATCH-UPDATE] batch_update_refractory_period: {} neurons",
            neuron_ids.len()
        );

        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                // Update base period
                self.neuron_array.write().unwrap().refractory_periods[idx] = *value;
                // Enforce immediately: set countdown to new period (or 0)
                if *value > 0 {
                    self.neuron_array.write().unwrap().refractory_countdowns[idx] = *value;
                } else {
                    self.neuron_array.write().unwrap().refractory_countdowns[idx] = 0;
                }
                // Reset consecutive fire count to avoid stale extended refractory state
                self.neuron_array.write().unwrap().consecutive_fire_counts[idx] = 0;
                updated_count += 1;

                // Log first few neurons and any that match our monitored neuron 16438
                if updated_count <= 3 || *neuron_id == 16438 {
                    info!(
                        "[RUST-BATCH-UPDATE]   Neuron {}: refractory_period={}, countdown={}",
                        neuron_id, value, self.neuron_array.read().unwrap().refractory_countdowns[idx]
                    );
                }
            }
        }

        updated_count
    }

    /// Batch update threshold for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_threshold(&mut self, neuron_ids: &[u32], values: &[f32]) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().thresholds[idx] = *value;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update leak coefficient for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_leak_coefficient(&mut self, neuron_ids: &[u32], values: &[f32]) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().leak_coefficients[idx] = value.clamp(0.0, 1.0);
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update consecutive fire limit for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_consecutive_fire_limit(
        &mut self,
        neuron_ids: &[u32],
        values: &[u16],
    ) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().consecutive_fire_limits[idx] = *value;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update snooze period (extended refractory) for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_snooze_period(&mut self, neuron_ids: &[u32], values: &[u16]) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().snooze_periods[idx] = *value;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update membrane potential for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_membrane_potential(&mut self, neuron_ids: &[u32], values: &[f32]) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().membrane_potentials[idx] = *value;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update resting potential for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_resting_potential(&mut self, neuron_ids: &[u32], values: &[f32]) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().resting_potentials[idx] = *value;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update excitability for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_excitability(&mut self, neuron_ids: &[u32], values: &[f32]) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().excitabilities[idx] = value.clamp(0.0, 1.0);
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update neuron type for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_neuron_type(&mut self, neuron_ids: &[u32], values: &[i32]) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().neuron_types[idx] = *value;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Batch update MP charge accumulation for multiple neurons
    /// Returns number of neurons updated
    pub fn batch_update_mp_charge_accumulation(
        &mut self,
        neuron_ids: &[u32],
        values: &[bool],
    ) -> usize {
        if neuron_ids.len() != values.len() {
            return 0;
        }

        let mut updated_count = 0;
        for (neuron_id, value) in neuron_ids.iter().zip(values.iter()) {
            let idx = *neuron_id as usize;
            if idx < self.neuron_array.read().unwrap().count && self.neuron_array.read().unwrap().valid_mask[idx] {
                self.neuron_array.write().unwrap().mp_charge_accumulation[idx] = *value;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Update MP charge accumulation for all neurons in a cortical area
    /// Returns number of neurons updated
    pub fn update_cortical_area_mp_charge_accumulation(
        &mut self,
        cortical_area: u32,
        mp_charge_accumulation: bool,
    ) -> usize {
        let mut updated_count = 0;

        // CRITICAL: Iterate by ARRAY INDEX (not neuron_id!)
        for idx in 0..self.neuron_array.read().unwrap().count {
            if self.neuron_array.read().unwrap().valid_mask[idx]
                && self.neuron_array.read().unwrap().cortical_areas[idx] == cortical_area
            {
                self.neuron_array.write().unwrap().mp_charge_accumulation[idx] = mp_charge_accumulation;
                updated_count += 1;
            }
        }

        updated_count
    }

    /// Delete a neuron (mark as invalid)
    /// Returns true if successful, false if neuron out of bounds
    pub fn delete_neuron(&mut self, neuron_id: u32) -> bool {
        let idx = neuron_id as usize;
        if idx >= self.neuron_array.read().unwrap().count {
            return false;
        }

        self.neuron_array.write().unwrap().valid_mask[idx] = false;
        true
    }

    /// Check if a neuron exists and is valid (not deleted)
    pub fn is_neuron_valid(&self, neuron_id: u32) -> bool {
        let idx = neuron_id as usize;
        let neuron_array = self.neuron_array.read().unwrap();
        idx < neuron_array.count && neuron_array.valid_mask[idx]
    }

    /// Get neuron coordinates (x, y, z)
    pub fn get_neuron_coordinates(&self, neuron_id: u32) -> (u32, u32, u32) {
        self.neuron_array.read().unwrap().get_coordinates(NeuronId(neuron_id))
    }

    /// Get cortical area for a neuron
    pub fn get_neuron_cortical_area(&self, neuron_id: u32) -> u32 {
        self.neuron_array.read().unwrap().get_cortical_area(NeuronId(neuron_id)).0
    }

    /// Get all neuron IDs in a specific cortical area
    pub fn get_neurons_in_cortical_area(&self, cortical_idx: u32) -> Vec<u32> {
        self.neuron_array.read().unwrap().get_neurons_in_cortical_area(cortical_idx)
    }

    /// Get total number of active neurons
    pub fn get_neuron_count(&self) -> usize {
        self.neuron_array.read().unwrap().get_neuron_count()
    }

    /// Get synapse count (valid only)
    pub fn get_synapse_count(&self) -> usize {
        self.synapse_array.read().unwrap().valid_count()
    }

    /// Get all outgoing synapses from a source neuron
    /// Returns Vec of (target_neuron_id, weight)
    pub fn get_outgoing_synapses(&self, source_neuron_id: u32) -> Vec<(u32, u8, u8, u8)> {
        let source = NeuronId(source_neuron_id);

        // Look up synapse indices for this source neuron
        let prop_engine = self.propagation_engine.read().unwrap();
        let synapse_indices = match prop_engine.synapse_index.get(&source) {
            Some(indices) => indices,
            None => return Vec::new(), // No synapses from this neuron
        };

        // Collect all valid synapses with full properties
        let mut outgoing = Vec::new();
        for &syn_idx in synapse_indices {
            if syn_idx < self.synapse_array.read().unwrap().count && self.synapse_array.read().unwrap().valid_mask[syn_idx] {
                let target = self.synapse_array.read().unwrap().target_neurons[syn_idx];
                let weight = self.synapse_array.read().unwrap().weights[syn_idx];
                let psp = self.synapse_array.read().unwrap().postsynaptic_potentials[syn_idx];
                let synapse_type = self.synapse_array.read().unwrap().types[syn_idx];
                outgoing.push((target, weight, psp, synapse_type));
            }
        }

        outgoing
    }

    /// Get incoming synapses for a neuron (neuron is the target)
    /// Returns Vec<(source_neuron_id, weight, conductance, synapse_type)>
    pub fn get_incoming_synapses(&self, target_neuron_id: u32) -> Vec<(u32, u8, u8, u8)> {
        let mut synapses = Vec::new();

        // Iterate through all synapses to find ones targeting this neuron
        // Note: This is O(n) - we could optimize with a target_index HashMap if needed
        for i in 0..self.synapse_array.read().unwrap().count {
            if self.synapse_array.read().unwrap().valid_mask[i]
                && self.synapse_array.read().unwrap().target_neurons[i] == target_neuron_id
            {
                synapses.push((
                    self.synapse_array.read().unwrap().source_neurons[i],
                    self.synapse_array.read().unwrap().weights[i],
                    self.synapse_array.read().unwrap().postsynaptic_potentials[i],
                    self.synapse_array.read().unwrap().types[i],
                ));
            }
        }

        synapses
    }

    /// Get neuron state for diagnostics (CFC, extended refractory, potential, etc.)
    /// Returns (cfc, cfc_limit, extended_refrac_period, potential, threshold, refrac_countdown)
    pub fn get_neuron_state(&self, neuron_id: NeuronId) -> Option<(u16, u16, u16, f32, f32, u16)> {
        // neuron_id == array index (direct access)
        let idx = neuron_id.0 as usize;
        if idx >= self.neuron_array.read().unwrap().count || !self.neuron_array.read().unwrap().valid_mask[idx] {
            return None;
        }

        Some((
            self.neuron_array.read().unwrap().consecutive_fire_counts[idx],
            self.neuron_array.read().unwrap().consecutive_fire_limits[idx],
            self.neuron_array.read().unwrap().snooze_periods[idx], // Extended refractory period (additive)
            self.neuron_array.read().unwrap().membrane_potentials[idx],
            self.neuron_array.read().unwrap().thresholds[idx],
            self.neuron_array.read().unwrap().refractory_countdowns[idx],
        ))
    }
}

/// Phase 1 injection result
///
/// Migration status: Metrics struct for burst processing. Will be used for monitoring
/// and debugging once telemetry system is migrated from Python.
/// Warning about unused struct is expected during migration.
#[derive(Debug)]
#[allow(dead_code)]  // In development - used for monitoring/debugging
struct InjectionResult {
    power_injections: usize,
    synaptic_injections: usize,
    sensory_injections: usize,
}

/// Phase 1 injection with automatic power neuron discovery
///
/// 🔋 Power neurons are identified by cortical_idx = 1 (_power area)
/// No separate list - scans neuron array directly!
fn phase1_injection_with_synapses(
    fcl: &mut FireCandidateList,
    neuron_array: &mut NeuronArray,
    propagation_engine: &mut SynapticPropagationEngine,
    previous_fire_queue: &FireQueue,
    power_amount: f32,
    synapse_array: &SynapseArray,
    pending_sensory: &std::sync::Mutex<Vec<(NeuronId, f32)>>,
) -> Result<InjectionResult> {
    // Clear FCL from previous burst
    fcl.clear();

    // CRITICAL FIX: Reset membrane potentials for neurons with mp_charge_accumulation=false
    // This prevents ghost potential accumulation and self-stimulation bugs
    //
    // Behavior:
    // - mp_acc=true: Neuron keeps its potential across bursts (integrator behavior)
    // - mp_acc=false: Neuron resets to 0.0 at start of each burst (coincidence detector)
    //
    // This ensures neurons only fire from CURRENT BURST stimulation, not accumulated history
    for idx in 0..neuron_array.count {
        if neuron_array.valid_mask[idx] && !neuron_array.mp_charge_accumulation[idx] {
            // Reset membrane potential for non-accumulating neurons
            neuron_array.membrane_potentials[idx] = 0.0;
        }
    }

    let mut power_count = 0;
    let mut synaptic_count = 0;
    let mut sensory_count = 0;

    // 0. Drain pending sensory injections (AFTER clear, BEFORE power/synapses)
    if let Ok(mut pending) = pending_sensory.lock() {
        if !pending.is_empty() {
            // 🔍 DEBUG: Log first sensory injection
            static FIRST_SENSORY_LOG: std::sync::Once = std::sync::Once::new();
            FIRST_SENSORY_LOG.call_once(|| {
                info!("╔══════════════════════════════════════════════════════════════");
                info!("║ [SENSORY-INJECTION] 🎬 DRAINING STAGED SENSORY DATA");
                info!(
                    "║ Injecting {} neurons AFTER FCL clear (prevents race)",
                    pending.len()
                );
                info!("╚══════════════════════════════════════════════════════════════");
            });

            for (neuron_id, potential) in pending.drain(..) {
                fcl.add_candidate(neuron_id, potential);
                sensory_count += 1;
            }
        }
    }

    // 1. Power Injection - Scan neuron array for cortical_idx = 1
    static FIRST_LOG: std::sync::Once = std::sync::Once::new();
    FIRST_LOG.call_once(|| {
        info!("╔══════════════════════════════════════════════════════════════");
        info!("║ [POWER-INJECTION] 🔋 AUTO-DISCOVERING POWER NEURONS");
        info!("║ Scanning neuron array for cortical_idx = 1 (_power area)");
        info!("╚══════════════════════════════════════════════════════════════");
    });

    // 🔍 DIAGNOSTIC: Log neuron array state on first scan with neurons
    static DIAGNOSTIC_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !DIAGNOSTIC_LOGGED.load(Ordering::Relaxed) && neuron_array.count > 0 {
        info!("[POWER-DIAGNOSTIC] Neuron array has {} neurons", neuron_array.count);
        
        // Sample first 20 neurons to see their cortical_areas
        let sample_count = neuron_array.count.min(20);
        let mut cortical_area_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for i in 0..sample_count {
            if neuron_array.valid_mask[i] {
                let cortical_area = neuron_array.cortical_areas[i];
                *cortical_area_counts.entry(cortical_area).or_insert(0) += 1;
            }
        }
        info!("[POWER-DIAGNOSTIC] First {} neurons cortical_area distribution: {:?}", sample_count, cortical_area_counts);
        DIAGNOSTIC_LOGGED.store(true, Ordering::Relaxed);
    }

    // Scan all neurons for _power cortical area (cortical_idx = 1)
    for array_idx in 0..neuron_array.count {
        let neuron_id = neuron_array.index_to_neuron_id[array_idx];
        if array_idx < neuron_array.count && neuron_array.valid_mask[array_idx] {
            let cortical_area = neuron_array.cortical_areas[array_idx];

            // Check if this is a power neuron (cortical_area = 1)
            if cortical_area == 1 {
                fcl.add_candidate(NeuronId(neuron_id), power_amount);
                power_count += 1;
            }
        }
    }

    // Log first injection and track power neuron count changes
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    static FIRST_INJECTION: AtomicBool = AtomicBool::new(false);
    static LAST_POWER_COUNT: AtomicUsize = AtomicUsize::new(0);

    let last_count = LAST_POWER_COUNT.load(Ordering::Relaxed);

    if !FIRST_INJECTION.load(Ordering::Relaxed) && power_count > 0 {
        // First burst with power neurons found
        info!(
            "[POWER-INJECTION] ✅ Injected {} power neurons into FCL",
            power_count
        );
        FIRST_INJECTION.store(true, Ordering::Relaxed);
        LAST_POWER_COUNT.store(power_count, Ordering::Relaxed);
    } else if power_count == 0 && FIRST_INJECTION.load(Ordering::Relaxed) && last_count > 0 {
        // Power neurons disappeared after working
        error!(
            "[POWER-INJECTION] ❌ ERROR: Power neurons DISAPPEARED! (was {}, now 0)",
            last_count
        );
        LAST_POWER_COUNT.store(0, Ordering::Relaxed);
    } else if power_count == 0 && !FIRST_INJECTION.load(Ordering::Relaxed) {
        // First burst with no power neurons (pre-embryogenesis)
        warn!("[POWER-INJECTION] ⚠️ No power neurons found yet (cortical_idx=1 '_power' area not created or empty) - will auto-discover after genome load");
        FIRST_INJECTION.store(true, Ordering::Relaxed);
        LAST_POWER_COUNT.store(0, Ordering::Relaxed);
    } else if power_count > 0 && last_count == 0 {
        // Power neurons APPEARED after being absent (0→N transition) - CRITICAL LOG
        info!(
            "[POWER-INJECTION] ✅ Power neurons NOW ACTIVE! Injected {} neurons into FCL (was 0, genome loaded successfully)",
            power_count
        );
        LAST_POWER_COUNT.store(power_count, Ordering::Relaxed);
    } else if power_count != last_count && power_count > 0 && last_count > 0 {
        // Power neuron count changed (N→M transition where both are non-zero)
        info!(
            "[POWER-INJECTION] ℹ️  Power neuron count changed: {} → {} neurons",
            last_count, power_count
        );
        LAST_POWER_COUNT.store(power_count, Ordering::Relaxed);
    }

    // 2. Synaptic Propagation
    if !previous_fire_queue.is_empty() {
        let fired_ids = previous_fire_queue.get_all_neuron_ids();

        // Call synaptic propagation engine (ZERO-COPY: pass synapse_array by reference)
        let propagation_result = propagation_engine.propagate(&fired_ids, synapse_array)?;

        // Inject propagated potentials into FCL
        for (_cortical_area, targets) in propagation_result {
            for &(target_neuron_id, contribution) in &targets {
                fcl.add_candidate(target_neuron_id, contribution.0); // Extract f32 from SynapticContribution
                synaptic_count += 1;
            }
        }
    }

    Ok(InjectionResult {
        power_injections: power_count,
        synaptic_injections: synaptic_count,
        sensory_injections: sensory_count,
    })
}

// ═══════════════════════════════════════════════════════════
// Fire Ledger API (Extension of RustNPU impl)
// ═══════════════════════════════════════════════════════════
impl RustNPU {
    /// Get firing history for a cortical area from Fire Ledger
    /// Returns Vec of (timestep, Vec<neuron_id>) tuples, newest first
    pub fn get_fire_ledger_history(
        &self,
        cortical_idx: u32,
        lookback_steps: usize,
    ) -> Vec<(u64, Vec<u32>)> {
        self.fire_structures.lock().unwrap().fire_ledger.get_history(cortical_idx, lookback_steps)
    }

    /// Get Fire Ledger window size for a cortical area
    pub fn get_fire_ledger_window_size(&self, cortical_idx: u32) -> usize {
        self.fire_structures.lock().unwrap().fire_ledger.get_area_window_size(cortical_idx)
    }

    /// Configure Fire Ledger window size for a specific cortical area
    pub fn configure_fire_ledger_window(&mut self, cortical_idx: u32, window_size: usize) {
        self.fire_structures.lock().unwrap().fire_ledger
            .configure_area_window(cortical_idx, window_size);
    }

    /// Get all configured Fire Ledger window sizes
    pub fn get_all_fire_ledger_configs(&self) -> Vec<(u32, usize)> {
        self.fire_structures.lock().unwrap().fire_ledger.get_all_window_configs()
    }
}

// ═══════════════════════════════════════════════════════════
// FQ Sampler API (Entry Point #2: Motor/Visualization Output)
// ═══════════════════════════════════════════════════════════
impl RustNPU {
    /// Sample the current Fire Queue for visualization/motor output
    ///
    /// Returns None if:
    /// - Rate limit not met
    /// - Fire Queue is empty
    /// - Burst already sampled (deduplication)
    ///
    /// Returns HashMap of cortical_idx -> area data
    ///
    /// ⚠️ DEPRECATED: This method triggers deduplication and may return None if burst already sampled.
    /// Use `get_latest_fire_queue_sample()` instead for non-consuming reads.
    pub fn sample_fire_queue(
        &mut self,
    ) -> Option<AHashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>> {
        let mut fire_structures = self.fire_structures.lock().unwrap();
        let current_fq_clone = fire_structures.current_fire_queue.clone();
        let sample_result = fire_structures.fq_sampler.sample(&current_fq_clone)?;
        drop(fire_structures);

        // Convert to Python-friendly format
        let mut result = AHashMap::new();
        for (cortical_idx, area_data) in sample_result.areas {
            result.insert(
                cortical_idx,
                (
                    area_data.neuron_ids,
                    area_data.coordinates_x,
                    area_data.coordinates_y,
                    area_data.coordinates_z,
                    area_data.potentials,
                ),
            );
        }

        Some(result)
    }

    /// Get the latest cached Fire Queue sample (non-consuming read)
    ///
    /// This returns the most recent sample WITHOUT triggering rate limiting or deduplication.
    /// Perfect for Python wrappers and SHM writers that need to read the same burst multiple times.
    ///
    /// Returns None if no sample has been taken yet (no bursts processed).
    pub fn get_latest_fire_queue_sample(
        &self,
    ) -> Option<AHashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>> {
        let fire_structures = self.fire_structures.lock().unwrap();
        let sample_result = fire_structures.fq_sampler.get_latest_sample()?;

        // Convert to Python-friendly format
        let mut result = AHashMap::new();
        for (cortical_idx, area_data) in &sample_result.areas {
            result.insert(
                *cortical_idx,
                (
                    area_data.neuron_ids.clone(),
                    area_data.coordinates_x.clone(),
                    area_data.coordinates_y.clone(),
                    area_data.coordinates_z.clone(),
                    area_data.potentials.clone(),
                ),
            );
        }

        Some(result)
    }

    /// Force sample the Fire Queue (for burst loop, bypasses rate limiting)
    ///
    /// This is used by the burst loop to sample on every burst, regardless of the FQ sampler's
    /// configured rate limit. The rate limiting is meant for external consumers, not the burst loop itself.
    pub fn force_sample_fire_queue(
        &mut self,
    ) -> Option<AHashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>> {
        // FIXED: Use get_current_fire_queue() instead of accessing private fields
        Some(self.get_current_fire_queue())
    }

    /// Get current Fire Queue directly (bypasses FQ Sampler rate limiting)
    /// Used by FCL endpoint to get real-time firing data without sampling delays
    pub fn get_current_fire_queue(
        &self,
    ) -> AHashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)> {
        let mut result = AHashMap::new();

        // Convert current Fire Queue to the same format as sample_fire_queue
        for (cortical_idx, neurons) in &self.fire_structures.lock().unwrap().current_fire_queue.neurons_by_area {
            let mut neuron_ids = Vec::with_capacity(neurons.len());
            let mut coords_x = Vec::with_capacity(neurons.len());
            let mut coords_y = Vec::with_capacity(neurons.len());
            let mut coords_z = Vec::with_capacity(neurons.len());
            let mut potentials = Vec::with_capacity(neurons.len());

            for neuron in neurons {
                neuron_ids.push(neuron.neuron_id.0);
                coords_x.push(neuron.x);
                coords_y.push(neuron.y);
                coords_z.push(neuron.z);
                potentials.push(neuron.membrane_potential);
            }

            result.insert(
                *cortical_idx,
                (neuron_ids, coords_x, coords_y, coords_z, potentials),
            );
        }

        result
    }

    /// Set FQ Sampler frequency (Hz)
    pub fn set_fq_sampler_frequency(&mut self, frequency_hz: f64) {
        self.fire_structures.lock().unwrap().fq_sampler.set_sample_frequency(frequency_hz);
    }

    /// Get FQ Sampler frequency (Hz)
    pub fn get_fq_sampler_frequency(&self) -> f64 {
        self.fire_structures.lock().unwrap().fq_sampler.get_sample_frequency()
    }

    /// Set visualization subscriber state
    pub fn set_visualization_subscribers(&mut self, has_subscribers: bool) {
        self.fire_structures.lock().unwrap().fq_sampler
            .set_visualization_subscribers(has_subscribers);
    }

    /// Check if visualization subscribers are connected
    pub fn has_visualization_subscribers(&self) -> bool {
        self.fire_structures.lock().unwrap().fq_sampler.has_visualization_subscribers()
    }

    /// Set motor subscriber state
    pub fn set_motor_subscribers(&mut self, has_subscribers: bool) {
        self.fire_structures.lock().unwrap().fq_sampler.set_motor_subscribers(has_subscribers);
    }

    /// Check if motor subscribers are connected
    pub fn has_motor_subscribers(&self) -> bool {
        self.fire_structures.lock().unwrap().fq_sampler.has_motor_subscribers()
    }

    /// Get total FQ Sampler samples taken
    pub fn get_fq_sampler_samples_taken(&self) -> u64 {
        self.fire_structures.lock().unwrap().fq_sampler.get_samples_taken()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════
    // Core NPU Creation & Initialization
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_npu_creation() {
        let npu = RustNPU::new(1000, 10000, 20);
        assert_eq!(npu.get_neuron_count(), 0);
        assert_eq!(npu.get_synapse_count(), 0);
        assert_eq!(npu.get_burst_count(), 0);
    }

    #[test]
    fn test_npu_creation_with_zero_capacity() {
        let npu = RustNPU::new(0, 0, 0);
        assert_eq!(npu.get_neuron_count(), 0);
        assert_eq!(npu.get_synapse_count(), 0);
    }

    #[test]
    fn test_npu_creation_with_large_capacity() {
        let npu = RustNPU::new(1_000_000, 10_000_000, 100);
        assert_eq!(npu.get_neuron_count(), 0);
    }

    // ═══════════════════════════════════════════════════════════
    // Neuron Management
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_add_neurons() {
        let mut npu = RustNPU::new(1000, 10000, 20);

        let id1 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        let id2 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 1, 0, 0)
            .unwrap();

        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(npu.get_neuron_count(), 2);
    }

    #[test]
    fn test_add_neuron_sequential_ids() {
        let mut npu = RustNPU::new(100, 1000, 10);

        for i in 0..10 {
            let id = npu
                .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, i, 0, 0)
                .unwrap();
            assert_eq!(id.0, i);
        }

        assert_eq!(npu.get_neuron_count(), 10);
    }

    #[test]
    fn test_add_neuron_different_parameters() {
        let mut npu = RustNPU::new(100, 1000, 10);

        // High threshold
        let _n1 = npu
            .add_neuron(10.0, 0.0, 0.0, 0, 0, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        // High leak
        let _n2 = npu
            .add_neuron(1.0, 0.9, 0.0, 0, 0, 1.0, 0, 0, true, 1, 1, 0, 0)
            .unwrap();

        // Long refractory period
        let _n3 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 100, 1.0, 0, 0, true, 1, 2, 0, 0)
            .unwrap();

        // Low excitability
        let _n4 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 0.1, 0, 0, true, 1, 3, 0, 0)
            .unwrap();

        assert_eq!(npu.get_neuron_count(), 4);
    }

    #[test]
    fn test_add_neuron_different_cortical_areas() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let _power = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        let _area2 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, 0, 0, 0)
            .unwrap();
        let _area3 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 3, 0, 0, 0)
            .unwrap();

        assert_eq!(npu.get_neuron_count(), 3);
    }

    #[test]
    fn test_add_neuron_3d_coordinates() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let _n1 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 5, 10, 15)
            .unwrap();

        assert_eq!(npu.get_neuron_count(), 1);
    }

    // ═══════════════════════════════════════════════════════════
    // Synapse Management
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_add_synapses() {
        let mut npu = RustNPU::new(1000, 10000, 20);

        let n1 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        let n2 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 1, 0, 0)
            .unwrap();

        npu.add_synapse(
            n1,
            n2,
            SynapticWeight(128),
            SynapticConductance(255),
            SynapseType::Excitatory,
        )
        .unwrap();

        assert_eq!(npu.get_synapse_count(), 1);
    }

    #[test]
    fn test_add_multiple_synapses() {
        let mut npu = RustNPU::new(1000, 10000, 20);

        let n1 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        let n2 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 1, 0, 0)
            .unwrap();
        let n3 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 2, 0, 0)
            .unwrap();

        npu.add_synapse(
            n1,
            n2,
            SynapticWeight(128),
            SynapticConductance(255),
            SynapseType::Excitatory,
        )
        .unwrap();
        npu.add_synapse(
            n1,
            n3,
            SynapticWeight(64),
            SynapticConductance(128),
            SynapseType::Excitatory,
        )
        .unwrap();
        npu.add_synapse(
            n2,
            n3,
            SynapticWeight(32),
            SynapticConductance(64),
            SynapseType::Inhibitory,
        )
        .unwrap();

        assert_eq!(npu.get_synapse_count(), 3);
    }

    #[test]
    fn test_add_inhibitory_synapse() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let n1 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        let n2 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 1, 0, 0)
            .unwrap();

        npu.add_synapse(
            n1,
            n2,
            SynapticWeight(128),
            SynapticConductance(255),
            SynapseType::Inhibitory,
        )
        .unwrap();

        assert_eq!(npu.get_synapse_count(), 1);
    }

    #[test]
    fn test_synapse_removal() {
        let mut npu = RustNPU::new(1000, 10000, 20);

        let n1 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        let n2 = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 1, 0, 0)
            .unwrap();

        npu.add_synapse(
            n1,
            n2,
            SynapticWeight(128),
            SynapticConductance(255),
            SynapseType::Excitatory,
        )
        .unwrap();
        assert_eq!(npu.get_synapse_count(), 1);

        assert!(npu.remove_synapse(n1, n2));
        assert_eq!(npu.get_synapse_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_synapse() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let n1 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        let n2 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 1, 0, 0)
            .unwrap();

        assert!(!npu.remove_synapse(n1, n2));
    }

    // ═══════════════════════════════════════════════════════════
    // Burst Processing & Power Injection
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_burst_processing() {
        let mut npu = RustNPU::new(1000, 10000, 20);

        // Add a power neuron
        let _power_neuron = npu
            .add_neuron(1.0, 0.1, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        // Process burst with power injection
        let result = npu.process_burst().unwrap();

        assert_eq!(result.burst, 1);
        assert_eq!(result.power_injections, 1);
        assert_eq!(result.neuron_count, 1);
    }

    #[test]
    fn test_burst_counter_increments() {
        let mut npu = RustNPU::new(100, 1000, 10);

        for i in 1..=10 {
            let result = npu.process_burst().unwrap();
            assert_eq!(result.burst, i as u64);
            assert_eq!(npu.get_burst_count(), i as u64);
        }
    }

    #[test]
    fn test_power_injection_auto_discovery() {
        let mut npu = RustNPU::new(100, 1000, 10);

        // Add 5 power neurons (cortical_area=1)
        for i in 0..5 {
            npu.add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, i, 0, 0)
                .unwrap();
        }

        // Add 5 regular neurons (cortical_area=2)
        for i in 0..5 {
            npu.add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, i, 0, 0)
                .unwrap();
        }

        let result = npu.process_burst().unwrap();

        // Should inject only cortical_area=1 neurons
        assert_eq!(result.power_injections, 5);
    }

    #[test]
    fn test_set_power_amount() {
        let mut npu = RustNPU::new(100, 1000, 10);

        // Add power neuron with high threshold
        npu.add_neuron(5.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        // Set high power amount
        npu.set_power_amount(10.0);

        // Should fire immediately (10.0 > 5.0 threshold)
        let result = npu.process_burst().unwrap();
        assert_eq!(result.neuron_count, 1);
    }

    #[test]
    fn test_empty_burst_no_power() {
        let mut npu = RustNPU::new(100, 1000, 10);

        // Add only regular neurons (no power area)
        npu.add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, 0, 0, 0)
            .unwrap();

        let result = npu.process_burst().unwrap();

        assert_eq!(result.power_injections, 0);
    }

    #[test]
    fn test_power_injection_zero_to_n_transition() {
        // Test the startup race condition: burst loop starts before genome load
        // This simulates what happens in production when burst engine starts before embryogenesis
        let mut npu = RustNPU::new(100, 1000, 10);
        npu.set_power_amount(0.5);

        // Burst 1: No power neurons yet (pre-embryogenesis)
        let result1 = npu.process_burst().unwrap();
        assert_eq!(result1.power_injections, 0, "No power neurons before embryogenesis");

        // Simulate genome load: Add power neurons
        for i in 0..10 {
            npu.add_neuron(0.5, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, i, 0, 0)
                .unwrap();
        }

        // Burst 2: Power neurons now present (0→N transition) - should log and inject!
        let result2 = npu.process_burst().unwrap();
        assert_eq!(result2.power_injections, 10, "Should inject all 10 power neurons after genome load");

        // Burst 3: Should still inject power neurons consistently
        let result3 = npu.process_burst().unwrap();
        assert_eq!(result3.power_injections, 10, "Should continue injecting power neurons on every burst");
    }

    // ═══════════════════════════════════════════════════════════
    // Sensory Input Injection
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_inject_sensory_input() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let neuron = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, 0, 0, 0)
            .unwrap();

        npu.inject_sensory_with_potentials(&[(neuron, 0.5)]);

        // Sensory input is staged until next burst
        let _result = npu.process_burst().unwrap();
    }

    #[test]
    fn test_inject_multiple_sensory_inputs() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let n1 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, 0, 0, 0)
            .unwrap();
        let n2 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, 1, 0, 0)
            .unwrap();
        let n3 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, 2, 0, 0)
            .unwrap();

        npu.inject_sensory_with_potentials(&[(n1, 0.5), (n2, 0.3), (n3, 0.8)]);

        let _result = npu.process_burst().unwrap();
    }

    #[test]
    fn test_sensory_accumulation_on_same_neuron() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let neuron = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, 0, 0, 0)
            .unwrap();

        npu.inject_sensory_with_potentials(&[(neuron, 0.3)]);
        npu.inject_sensory_with_potentials(&[(neuron, 0.3)]);
        npu.inject_sensory_with_potentials(&[(neuron, 0.3)]);

        let _result = npu.process_burst().unwrap();
        // Should accumulate 0.9 potential
    }

    // ═══════════════════════════════════════════════════════════
    // Fire Ledger Tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_fire_ledger_recording() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let _neuron = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        // Process burst
        npu.process_burst().unwrap();

        // Check fire ledger
        let history = npu.get_fire_ledger_history(1, 10);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_fire_ledger_window_configuration() {
        let mut npu = RustNPU::new(100, 1000, 10);

        npu.configure_fire_ledger_window(1, 50);

        let window_size = npu.get_fire_ledger_window_size(1);
        assert_eq!(window_size, 50);
    }

    // ═══════════════════════════════════════════════════════════
    // FQ Sampler Tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_fq_sampler_rate_limiting() {
        let mut npu = RustNPU::new(100, 1000, 10);

        npu.add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        npu.set_visualization_subscribers(true);

        npu.process_burst().unwrap();

        // Should be able to sample
        let _sample = npu.sample_fire_queue();
        // Rate limiting may prevent sampling
    }

    #[test]
    fn test_fq_sampler_motor_subscribers() {
        let mut npu = RustNPU::new(100, 1000, 10);

        assert!(!npu.has_motor_subscribers());

        npu.set_motor_subscribers(true);
        assert!(npu.has_motor_subscribers());

        npu.set_motor_subscribers(false);
        assert!(!npu.has_motor_subscribers());
    }

    #[test]
    fn test_fq_sampler_viz_subscribers() {
        let mut npu = RustNPU::new(100, 1000, 10);

        assert!(!npu.has_visualization_subscribers());

        npu.set_visualization_subscribers(true);
        assert!(npu.has_visualization_subscribers());

        npu.set_visualization_subscribers(false);
        assert!(!npu.has_visualization_subscribers());
    }

    #[test]
    fn test_get_latest_fire_queue_sample() {
        let mut npu = RustNPU::new(100, 1000, 10);

        npu.add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        // Before any burst
        assert!(npu.get_latest_fire_queue_sample().is_none());

        npu.process_burst().unwrap();

        // After burst, may have sample
        let _sample = npu.get_latest_fire_queue_sample();
    }

    // ═══════════════════════════════════════════════════════════
    // Area Name Mapping
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_register_cortical_area_name() {
        let mut npu = RustNPU::new(100, 1000, 10);

        npu.register_cortical_area(1, "visual_cortex".to_string());
        npu.register_cortical_area(2, "motor_cortex".to_string());

        // Names are registered successfully
    }

    // ═══════════════════════════════════════════════════════════
    // Edge Cases & Error Handling
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_add_synapse_to_nonexistent_neuron() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let n1 = npu
            .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        let nonexistent = NeuronId(999);

        // Note: add_synapse does NOT validate neuron existence for performance
        // Synapses to nonexistent neurons are silently ignored during propagation
        let result = npu.add_synapse(
            n1,
            nonexistent,
            SynapticWeight(128),
            SynapticConductance(255),
            SynapseType::Excitatory,
        );

        assert!(result.is_ok()); // No validation for performance
        assert_eq!(npu.get_synapse_count(), 1);
    }

    #[test]
    fn test_burst_with_empty_npu() {
        let mut npu = RustNPU::new(100, 1000, 10);

        let result = npu.process_burst().unwrap();

        assert_eq!(result.burst, 1);
        assert_eq!(result.neuron_count, 0);
        assert_eq!(result.power_injections, 0);
    }

    #[test]
    fn test_large_sensory_batch() {
        let mut npu = RustNPU::new(1000, 10000, 10);

        // Add 100 neurons
        let mut neurons = Vec::new();
        for i in 0..100 {
            let neuron = npu
                .add_neuron(1.0, 0.0, 0.0, 0, 5, 1.0, 0, 0, true, 2, i, 0, 0)
                .unwrap();
            neurons.push((neuron, 0.5));
        }

        npu.inject_sensory_with_potentials(&neurons);

        let _result = npu.process_burst().unwrap();
    }
}

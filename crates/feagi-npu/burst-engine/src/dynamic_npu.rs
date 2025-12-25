// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Runtime type dispatch for NPU
//!
//! Provides `DynamicNPU` enum for runtime selection between f32 and INT8 precision.

use crate::backend::{CPUBackend, ComputeBackend};
use crate::npu::RustNPU;
use feagi_npu_neural::types::*;
use feagi_npu_runtime::{NeuronStorage, Runtime, SynapseStorage};

// Import StdRuntime for the default type alias
#[cfg(feature = "std")]
use feagi_npu_runtime::StdRuntime;

/// Dynamic NPU that dispatches to f32 or INT8 at runtime
///
/// This enum allows the system to choose NPU precision based on genome configuration
/// at runtime, while still using compile-time monomorphization for performance.
///
/// ## Generic Type Parameters
/// - `R: Runtime`: The runtime implementation (StdRuntime, EmbeddedRuntime, etc.)
/// - `B: ComputeBackend`: The compute backend (CPUBackend, CUDABackend, etc.)
pub enum DynamicNPUGeneric<R: Runtime, B>
where
    B: ComputeBackend<f32, R::NeuronStorage<f32>, R::SynapseStorage>
        + ComputeBackend<INT8Value, R::NeuronStorage<INT8Value>, R::SynapseStorage>,
{
    /// 32-bit floating point NPU (highest precision)
    F32(RustNPU<R, f32, B>),

    /// 8-bit integer NPU (42% memory reduction)
    INT8(RustNPU<R, INT8Value, B>),
}

/// Default DynamicNPU type alias using StdRuntime and CPUBackend
///
/// This is the most common configuration for desktop/server deployments.
/// For other platforms, use `DynamicNPUGeneric<YourRuntime, YourBackend>` directly.
#[cfg(feature = "std")]
pub type DynamicNPU = DynamicNPUGeneric<StdRuntime, CPUBackend>;

/// Type alias for fire queue sample data structure
type FireQueueSample = ahash::AHashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>;

/// Macro for dispatching methods to the correct NPU variant
macro_rules! dispatch {
    ($self:expr, $method:ident($($args:expr),*)) => {
        match $self {
            DynamicNPUGeneric::F32(npu) => npu.$method($($args),*),
            DynamicNPUGeneric::INT8(npu) => npu.$method($($args),*),
        }
    };
}

/// Macro for dispatching mutable methods
macro_rules! dispatch_mut {
    ($self:expr, $method:ident($($args:expr),*)) => {
        match $self {
            DynamicNPUGeneric::F32(npu) => npu.$method($($args),*),
            DynamicNPUGeneric::INT8(npu) => npu.$method($($args),*),
        }
    };
}

impl<R: Runtime, B> DynamicNPUGeneric<R, B>
where
    B: ComputeBackend<f32, R::NeuronStorage<f32>, R::SynapseStorage>
        + ComputeBackend<INT8Value, R::NeuronStorage<INT8Value>, R::SynapseStorage>,
{
    /// Create new f32 NPU with provided runtime and backend
    pub fn new_f32(
        runtime: R,
        backend: B,
        neuron_capacity: usize,
        synapse_capacity: usize,
        fire_ledger_window: usize,
    ) -> Result<Self> {
        let npu = RustNPU::new(
            runtime,
            backend,
            neuron_capacity,
            synapse_capacity,
            fire_ledger_window,
        )?;
        Ok(DynamicNPUGeneric::F32(npu))
    }

    /// Create new INT8 NPU with provided runtime and backend
    pub fn new_int8(
        runtime: R,
        backend: B,
        neuron_capacity: usize,
        synapse_capacity: usize,
        fire_ledger_window: usize,
    ) -> Result<Self> {
        let npu = RustNPU::new(
            runtime,
            backend,
            neuron_capacity,
            synapse_capacity,
            fire_ledger_window,
        )?;
        Ok(DynamicNPUGeneric::INT8(npu))
    }

    // Common NPU methods (delegate to underlying implementation)

    #[allow(clippy::too_many_arguments)]
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
        match self {
            DynamicNPUGeneric::F32(npu) => npu.add_neuron(
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
            ),
            DynamicNPUGeneric::INT8(npu) => npu.add_neuron(
                INT8Value::from_f32(threshold),
                leak_coefficient,
                INT8Value::from_f32(resting_potential),
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
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
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
        match self {
            DynamicNPUGeneric::F32(npu) => npu.add_neurons_batch(
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
            ),
            DynamicNPUGeneric::INT8(npu) => {
                let thresholds_int8: Vec<INT8Value> =
                    thresholds.into_iter().map(INT8Value::from_f32).collect();
                let resting_int8: Vec<INT8Value> = resting_potentials
                    .into_iter()
                    .map(INT8Value::from_f32)
                    .collect();
                npu.add_neurons_batch(
                    thresholds_int8,
                    leak_coefficients,
                    resting_int8,
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
                )
            }
        }
    }

    pub fn add_synapse(
        &mut self,
        source: NeuronId,
        target: NeuronId,
        weight: SynapticWeight,
        conductance: SynapticConductance,
        synapse_type: SynapseType,
    ) -> Result<usize> {
        dispatch_mut!(
            self,
            add_synapse(source, target, weight, conductance, synapse_type)
        )
    }

    pub fn get_neurons_in_cortical_area(&self, cortical_idx: u32) -> Vec<u32> {
        dispatch!(self, get_neurons_in_cortical_area(cortical_idx))
    }

    pub fn get_neuron_coordinates(&self, neuron_id: u32) -> Option<(u32, u32, u32)> {
        dispatch!(self, get_neuron_coordinates(neuron_id))
    }

    pub fn get_neuron_count(&self) -> usize {
        dispatch!(self, get_neuron_count())
    }

    pub fn get_synapse_count(&self) -> usize {
        dispatch!(self, get_synapse_count())
    }

    pub fn get_cortical_area_neuron_count(&self, cortical_area: u32) -> usize {
        dispatch!(self, get_cortical_area_neuron_count(cortical_area))
    }

    pub fn process_burst(&self) -> Result<crate::npu::BurstResult> {
        dispatch!(self, process_burst())
    }

    pub fn register_cortical_area(&mut self, area_id: u32, name: String) {
        dispatch_mut!(self, register_cortical_area(area_id, name))
    }

    pub fn inject_sensory_xyzp_by_id(
        &mut self,
        cortical_id: &feagi_structures::genomic::cortical_area::CorticalID,
        xyzp_data: &[(u32, u32, u32, f32)],
    ) -> usize {
        dispatch_mut!(self, inject_sensory_xyzp_by_id(cortical_id, xyzp_data))
    }

    pub fn set_power_amount(&mut self, amount: f32) {
        dispatch_mut!(self, set_power_amount(amount))
    }

    pub fn get_burst_count(&self) -> u64 {
        dispatch!(self, get_burst_count())
    }

    #[allow(clippy::too_many_arguments)]
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
        match self {
            DynamicNPUGeneric::F32(npu) => npu.create_cortical_area_neurons(
                cortical_idx,
                width,
                height,
                depth,
                neurons_per_voxel,
                default_threshold,
                default_leak_coefficient,
                default_resting_potential,
                default_neuron_type,
                default_refractory_period,
                default_excitability,
                default_consecutive_fire_limit,
                default_snooze_period,
                default_mp_charge_accumulation,
            ),
            DynamicNPUGeneric::INT8(npu) => npu.create_cortical_area_neurons(
                cortical_idx,
                width,
                height,
                depth,
                neurons_per_voxel,
                default_threshold,
                default_leak_coefficient,
                default_resting_potential,
                default_neuron_type,
                default_refractory_period,
                default_excitability,
                default_consecutive_fire_limit,
                default_snooze_period,
                default_mp_charge_accumulation,
            ),
        }
    }

    pub fn get_neuron_state(&self, neuron_id: NeuronId) -> Option<(u16, u16, u16, f32, f32, u16)> {
        dispatch!(self, get_neuron_state(neuron_id))
    }

    pub fn get_neuron_id_at_coordinate(
        &self,
        cortical_area: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Option<u32> {
        dispatch!(self, get_neuron_id_at_coordinate(cortical_area, x, y, z))
    }

    pub fn get_neuron_cortical_area(&self, neuron_id: u32) -> u32 {
        dispatch!(self, get_neuron_cortical_area(neuron_id))
    }

    pub fn delete_neuron(&mut self, neuron_id: u32) -> bool {
        dispatch_mut!(self, delete_neuron(neuron_id))
    }

    pub fn is_neuron_valid(&self, neuron_id: u32) -> bool {
        let idx = neuron_id as usize;
        match self {
            DynamicNPUGeneric::F32(npu) => {
                let storage = npu.neuron_storage.read().unwrap();
                idx < storage.count() && storage.valid_mask()[idx]
            }
            DynamicNPUGeneric::INT8(npu) => {
                let storage = npu.neuron_storage.read().unwrap();
                idx < storage.count() && storage.valid_mask()[idx]
            }
        }
    }

    pub fn update_neuron_threshold(&mut self, neuron_id: u32, threshold: f32) -> bool {
        match self {
            DynamicNPUGeneric::F32(npu) => npu.update_neuron_threshold(neuron_id, threshold),
            DynamicNPUGeneric::INT8(npu) => {
                npu.update_neuron_threshold(neuron_id, INT8Value::from_f32(threshold))
            }
        }
    }

    pub fn update_cortical_area_leak(&mut self, cortical_area: u32, leak: f32) -> usize {
        dispatch_mut!(self, update_cortical_area_leak(cortical_area, leak))
    }

    pub fn update_cortical_area_excitability(
        &mut self,
        cortical_area: u32,
        excitability: f32,
    ) -> usize {
        dispatch_mut!(
            self,
            update_cortical_area_excitability(cortical_area, excitability)
        )
    }

    // Single neuron update methods (by neuron_id)
    pub fn update_neuron_leak(&mut self, neuron_id: u32, leak: f32) -> bool {
        match self {
            DynamicNPUGeneric::F32(npu) => npu.update_neuron_leak(neuron_id, leak),
            DynamicNPUGeneric::INT8(npu) => npu.update_neuron_leak(neuron_id, leak),
        }
    }

    pub fn update_neuron_excitability(&mut self, neuron_id: u32, excitability: f32) -> bool {
        match self {
            DynamicNPUGeneric::F32(npu) => npu.update_neuron_excitability(neuron_id, excitability),
            DynamicNPUGeneric::INT8(npu) => npu.update_neuron_excitability(neuron_id, excitability),
        }
    }

    pub fn update_neuron_resting_potential(
        &mut self,
        neuron_id: u32,
        resting_potential: f32,
    ) -> bool {
        match self {
            DynamicNPUGeneric::F32(npu) => {
                npu.update_neuron_resting_potential(neuron_id, resting_potential)
            }
            DynamicNPUGeneric::INT8(npu) => npu
                .update_neuron_resting_potential(neuron_id, INT8Value::from_f32(resting_potential)),
        }
    }

    pub fn remove_synapse(&mut self, source: NeuronId, target: NeuronId) -> bool {
        dispatch_mut!(self, remove_synapse(source, target))
    }

    pub fn update_synapse_weight(
        &mut self,
        source: NeuronId,
        target: NeuronId,
        new_weight: SynapticWeight,
    ) -> bool {
        dispatch_mut!(self, update_synapse_weight(source, target, new_weight))
    }

    pub fn rebuild_synapse_index(&mut self) {
        dispatch_mut!(self, rebuild_synapse_index())
    }

    pub fn get_neuron_capacity(&self) -> usize {
        match self {
            DynamicNPUGeneric::F32(npu) => {
                let storage = npu.neuron_storage.read().unwrap();
                NeuronStorage::capacity(&*storage)
            }
            DynamicNPUGeneric::INT8(npu) => {
                let storage = npu.neuron_storage.read().unwrap();
                NeuronStorage::capacity(&*storage)
            }
        }
    }

    pub fn get_synapse_capacity(&self) -> usize {
        match self {
            DynamicNPUGeneric::F32(npu) => {
                let storage = npu.synapse_storage.read().unwrap();
                SynapseStorage::capacity(&*storage)
            }
            DynamicNPUGeneric::INT8(npu) => {
                let storage = npu.synapse_storage.read().unwrap();
                SynapseStorage::capacity(&*storage)
            }
        }
    }

    pub fn get_neuron_property_by_index(&self, idx: usize, property: &str) -> Option<f32> {
        dispatch!(self, get_neuron_property_by_index(idx, property))
    }

    pub fn get_neuron_property_u16_by_index(&self, idx: usize, property: &str) -> Option<u16> {
        dispatch!(self, get_neuron_property_u16_by_index(idx, property))
    }

    pub fn get_incoming_synapses(&self, _neuron_id: u32) -> Vec<(u32, u8, u8, u8)> {
        // This method doesn't exist in RustNPU - return empty for now
        Vec::new()
    }

    pub fn get_outgoing_synapses(&self, _neuron_id: u32) -> Vec<(u32, u8, u8, u8)> {
        // This method doesn't exist in RustNPU - return empty for now
        Vec::new()
    }

    pub fn get_cortical_area_name(&self, area_id: u32) -> Option<String> {
        dispatch!(self, get_cortical_area_name(area_id))
    }

    pub fn update_cortical_area_threshold(&mut self, cortical_area: u32, threshold: f32) -> usize {
        dispatch_mut!(
            self,
            update_cortical_area_threshold(cortical_area, threshold)
        )
    }

    pub fn update_cortical_area_refractory_period(
        &mut self,
        cortical_area: u32,
        period: u16,
    ) -> usize {
        dispatch_mut!(
            self,
            update_cortical_area_refractory_period(cortical_area, period)
        )
    }

    pub fn update_cortical_area_snooze_period(&mut self, cortical_area: u32, period: u16) -> usize {
        dispatch_mut!(
            self,
            update_cortical_area_snooze_period(cortical_area, period)
        )
    }

    pub fn update_cortical_area_consecutive_fire_limit(
        &mut self,
        cortical_area: u32,
        limit: u16,
    ) -> usize {
        dispatch_mut!(
            self,
            update_cortical_area_consecutive_fire_limit(cortical_area, limit)
        )
    }

    pub fn update_cortical_area_mp_charge_accumulation(
        &mut self,
        cortical_area: u32,
        accumulation: bool,
    ) -> usize {
        dispatch_mut!(
            self,
            update_cortical_area_mp_charge_accumulation(cortical_area, accumulation)
        )
    }

    pub fn sample_fire_queue(&mut self) -> Option<FireQueueSample> {
        match self {
            DynamicNPUGeneric::F32(npu) => npu.sample_fire_queue(),
            DynamicNPUGeneric::INT8(npu) => npu.sample_fire_queue(),
        }
    }

    pub fn force_sample_fire_queue(&mut self) -> Option<FireQueueSample> {
        dispatch_mut!(self, force_sample_fire_queue())
    }

    pub fn get_last_fcl_snapshot(&self) -> Vec<(feagi_npu_neural::types::NeuronId, f32)> {
        dispatch!(self, get_last_fcl_snapshot())
    }

    pub fn inject_sensory_with_potentials(
        &mut self,
        neurons: &[(feagi_npu_neural::types::NeuronId, f32)],
    ) {
        match self {
            DynamicNPUGeneric::F32(npu) => npu.inject_sensory_with_potentials(neurons),
            DynamicNPUGeneric::INT8(npu) => npu.inject_sensory_with_potentials(neurons),
        }
    }

    pub fn configure_fire_ledger_window(&mut self, cortical_idx: u32, window_size: usize) {
        match self {
            DynamicNPUGeneric::F32(npu) => {
                npu.configure_fire_ledger_window(cortical_idx, window_size)
            }
            DynamicNPUGeneric::INT8(npu) => {
                npu.configure_fire_ledger_window(cortical_idx, window_size)
            }
        }
    }

    pub fn get_all_fire_ledger_configs(&self) -> Vec<(u32, usize)> {
        match self {
            DynamicNPUGeneric::F32(npu) => npu.get_all_fire_ledger_configs(),
            DynamicNPUGeneric::INT8(npu) => npu.get_all_fire_ledger_configs(),
        }
    }

    pub fn batch_get_neuron_ids_from_coordinates(
        &self,
        area_id: u32,
        coords: &[(u32, u32, u32)],
    ) -> Vec<feagi_npu_neural::types::NeuronId> {
        dispatch!(self, batch_get_neuron_ids_from_coordinates(area_id, coords))
    }

    pub fn get_registered_cortical_area_count(&self) -> usize {
        dispatch!(self, get_registered_cortical_area_count())
    }

    pub fn is_genome_loaded(&self) -> bool {
        dispatch!(self, is_genome_loaded())
    }

    pub fn get_power_amount(&self) -> f32 {
        dispatch!(self, get_power_amount())
    }

    pub fn neuron_count(&self) -> usize {
        dispatch!(self, get_neuron_count())
    }

    pub fn set_psp_uniform_distribution_flags(
        &mut self,
        flags: ahash::AHashMap<feagi_structures::genomic::cortical_area::CorticalID, bool>,
    ) {
        dispatch_mut!(self, set_psp_uniform_distribution_flags(flags))
    }

    pub fn set_mp_driven_psp_flags(
        &mut self,
        flags: ahash::AHashMap<feagi_structures::genomic::cortical_area::CorticalID, bool>,
    ) {
        dispatch_mut!(self, set_mp_driven_psp_flags(flags))
    }
}

// ============================================================================
// Backward Compatibility: StdRuntime + CPUBackend Convenience Methods
// ============================================================================

#[cfg(test)]
impl DynamicNPUGeneric<feagi_npu_runtime::StdRuntime, CPUBackend> {
    /// Create new f32 NPU with StdRuntime and CPUBackend (backward compatible)
    ///
    /// # Deprecation Notice
    /// This is a convenience method for backward compatibility.
    /// New code should use the generic `DynamicNPUGeneric::new_f32()` and pass runtime/backend explicitly.
    pub fn new_f32_std_cpu(
        neuron_capacity: usize,
        synapse_capacity: usize,
        fire_ledger_window: usize,
    ) -> Result<Self> {
        Self::new_f32(
            StdRuntime::new(),
            CPUBackend::new(),
            neuron_capacity,
            synapse_capacity,
            fire_ledger_window,
        )
    }

    /// Create new INT8 NPU with StdRuntime and CPUBackend (backward compatible)
    ///
    /// # Deprecation Notice
    /// This is a convenience method for backward compatibility.
    /// New code should use the generic `DynamicNPUGeneric::new_int8()` and pass runtime/backend explicitly.
    pub fn new_int8_std_cpu(
        neuron_capacity: usize,
        synapse_capacity: usize,
        fire_ledger_window: usize,
    ) -> Result<Self> {
        Self::new_int8(
            StdRuntime::new(),
            CPUBackend::new(),
            neuron_capacity,
            synapse_capacity,
            fire_ledger_window,
        )
    }
}

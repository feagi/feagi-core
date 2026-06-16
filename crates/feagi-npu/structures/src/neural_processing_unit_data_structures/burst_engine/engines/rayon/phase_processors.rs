use rayon::prelude::*;
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use feagi_structures::feagi_data::SupportsUintOps;
use crate::neural_processing_unit_data_structures::burst_engine::BurstEngineJustCompletedPhase;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::neuron_model_traits::neuron_model_processor::NeuronModelProcessorWithBurstHistoryCPU;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::phase_processing::{BurstEnginePhaseBurstCounterIndexIncrement, BurstEnginePhaseProcessor};
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::neuron_models::feagi_standard::processor::FeagiStandardModelProcessorCPU;
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::npu_data::BurstEngineDataRayon;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedBurstEngineBurstIndex, NPUWrappedNeuronCorticalLocalIndex, NPUWrappedNeuronMPQuantIndex, NPUWrappedNeuronMembranePotential};

/// Wraps a mutable slice base pointer so Rayon's `Fn` parallel closures can capture it.
/// SAFETY: Caller must ensure parallel index access is disjoint.
struct ParallelMutSlicePtr<T>(*mut T);
unsafe impl<T> Send for ParallelMutSlicePtr<T> {}
unsafe impl<T> Sync for ParallelMutSlicePtr<T> {}

impl<T> ParallelMutSlicePtr<T> {
    fn new(slice: &mut [T]) -> Self {
        Self(slice.as_mut_ptr())
    }

    unsafe fn get_mut(&self, index: usize) -> &mut T {
        // SAFETY: Caller guarantees `index` is in bounds and not aliased across threads.
        unsafe { &mut *self.0.add(index) }
    }
}

/// Wraps an immutable slice base pointer so Rayon's `Fn` parallel closures can capture it.
struct ParallelSlicePtr<T>(*const T);
unsafe impl<T> Send for ParallelSlicePtr<T> {}
unsafe impl<T> Sync for ParallelSlicePtr<T> {}

impl<T> ParallelSlicePtr<T> {
    fn new(slice: &[T]) -> Self {
        Self(slice.as_ptr())
    }

    unsafe fn get(&self, index: usize) -> &T {
        // SAFETY: Caller guarantees `index` is in bounds.
        unsafe { &*self.0.add(index) }
    }
}

pub trait BurstEnginePhaseProcessorCPU<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{
    fn process_phase(data: &mut BurstEngineDataRayon<FGQ>)
                     -> BurstEngineJustCompletedPhase;
}



pub struct BurstEnginePhaseBurstCounterIndexIncrementRayon;

impl<FGQ> BurstEnginePhaseProcessor<FGQ> for BurstEnginePhaseBurstCounterIndexIncrementRayon where FGQ: FeagiGlobalQuantization, {}

impl<FGQ> BurstEnginePhaseBurstCounterIndexIncrement<FGQ> for BurstEnginePhaseBurstCounterIndexIncrementRayon where FGQ: FeagiGlobalQuantization, {}

impl<FGQ> BurstEnginePhaseProcessorCPU<FGQ> for BurstEnginePhaseBurstCounterIndexIncrementRayon
where
    FGQ: FeagiGlobalQuantization,
{
    fn process_phase(data: &mut BurstEngineDataRayon<FGQ>) -> BurstEngineJustCompletedPhase {
        if data.burst_index == NPUWrappedBurstEngineBurstIndex::QUANT_MAX
        {
            // overflow!
            data.burst_index = NPUWrappedBurstEngineBurstIndex::QUANT_MAX / NPUWrappedBurstEngineBurstIndex::wrap(  FGQ::GlobalBurstIndexQuant::from_usize_unchecked(2));
            data.did_burst_index_overflow = true;
        }
        else {
            data.burst_index += NPUWrappedBurstEngineBurstIndex::QUANT_ONE;
            data.did_burst_index_overflow = false;
        }

        BurstEngineJustCompletedPhase::BurstCounterIndexIncrement
    }
}


// TODO NeuronModelUpdatedForBurstIndexRollover

// TODO SynapseModelUpdatedForBurstIndexRollover

// TODO FCLConsolidation



pub struct NeuronDynamicsNoPreCondenseRayon;

impl<FGQ> BurstEnginePhaseProcessor<FGQ> for NeuronDynamicsNoPreCondenseRayon where FGQ: FeagiGlobalQuantization, {}

impl<FGQ> BurstEnginePhaseBurstCounterIndexIncrement<FGQ> for NeuronDynamicsNoPreCondenseRayon where FGQ: FeagiGlobalQuantization, {}

impl<FGQ> BurstEnginePhaseProcessorCPU<FGQ> for NeuronDynamicsNoPreCondenseRayon
where
    FGQ: FeagiGlobalQuantization,
{
    fn process_phase(data: &mut BurstEngineDataRayon<FGQ>) -> BurstEngineJustCompletedPhase {
        let current_burst_index = data.burst_index;

        // Raw pointers are required because Rayon's for_each closure must be Fn, not FnMut
        // Mutable borrows of data fields cannot be captured so disjoint parallel writes are
        // routed through pointers whose targets are guaranteed notoverlapping by precomputed indexes
        let neuron_engine_cortical_indexes = ParallelSlicePtr::new(&data.neuron_engine_cortical_indexes);
        let cortical_context_lookups = ParallelSlicePtr::new(&data.cortical_context_lookups);
        let cortical_layouts_dimensional = ParallelSlicePtr::new(&data.cortical_layouts.dimensional);
        let neuron_model_cortical_data = ParallelSlicePtr::new(&data.neuron_model_cortical_data);
        let neuron_history = ParallelMutSlicePtr::new(&mut data.neuron_history);
        let neuron_model_neuron_data = ParallelMutSlicePtr::new(&mut data.neuron_model_neuron_data);
        let neuron_potentials = ParallelMutSlicePtr::new(&mut data.neuron_potentials.float_32);

        data.neuron_fcls.float_32.par_iter_mut().enumerate().for_each(|(neuron_type_index_u, fcl_value)| {
            if *fcl_value == NPUWrappedNeuronMembranePotential::QUANT_ZERO {
                return;
            }

            let neuron_mp_quant_index = NPUWrappedNeuronMPQuantIndex::wrap(
                FGQ::NeuronIndexCountQuant::from_usize_unchecked(neuron_type_index_u),
            );
            let neuron_mp_quant_index_usize = neuron_mp_quant_index.to_usize();

            // SAFETY: Parallel iterations mutate disjoint index ranges. Read-only slices are never written.
            // All pointers originate from `data` and remain valid for the duration of this `par_iter_mut`.
            unsafe {
                let engine_cortical_area_index_of_neuron =
                    neuron_engine_cortical_indexes.get(neuron_mp_quant_index_usize);

                let cortical_context_lookup =
                    cortical_context_lookups.get(engine_cortical_area_index_of_neuron.to_usize());

                let neuron_index_local = NPUWrappedNeuronCorticalLocalIndex::wrap(neuron_mp_quant_index.unwrap())
                    - cortical_context_lookup.mp_quant_to_local_neuron_index_offset;

                let neuron_history =
                    neuron_history.get_mut(neuron_type_index_u - cortical_context_lookup.mp_quant_to_neuron_history_index_offset.to_usize());


                // TODO not always dimensional!
                let dimensional =
                    cortical_layouts_dimensional.get(cortical_context_lookup.cortical_layout_index.to_usize());
                let feagi_standard_cortical_data = neuron_model_cortical_data
                    .get(cortical_context_lookup.neuron_model_cortical_data_index.to_usize());

                let neuron_model_data = neuron_model_neuron_data.get_mut(neuron_mp_quant_index_usize);
                let neuron_potential = neuron_potentials.get_mut(neuron_mp_quant_index_usize);


                let is_firing =
                    FeagiStandardModelProcessorCPU::process_neuron_potential_for_dimensional_cortical_configuration(
                        fcl_value,
                        &neuron_index_local,
                        &current_burst_index,
                        &neuron_history.burst_index_of_last_input,
                        &neuron_history.burst_index_of_last_firing,
                        dimensional,
                        feagi_standard_cortical_data,
                        neuron_model_data,
                        neuron_potential,
                    );

                *fcl_value = NPUWrappedNeuronMembranePotential::QUANT_ZERO;

                neuron_history.burst_index_of_last_input = current_burst_index;
                if is_firing {
                    neuron_history.burst_index_of_last_firing = current_burst_index;
                }
            }
        });

        BurstEngineJustCompletedPhase::NeuronDynamics
    }
}


// TODO UpdateFiringNeuronBitfield

// TODO CountFiringNeuronsPerCorticalArea

// TODO PreSynapseDataExchange

// TODO FiringNeuronConsolidation



pub struct SynapseDynamicsNoPreCondenseRayon;

impl<FGQ> BurstEnginePhaseProcessor<FGQ> for SynapseDynamicsNoPreCondenseRayon where FGQ: FeagiGlobalQuantization, {}

impl<FGQ> BurstEnginePhaseBurstCounterIndexIncrement<FGQ> for SynapseDynamicsNoPreCondenseRayon where FGQ: FeagiGlobalQuantization, {}

impl<FGQ> BurstEnginePhaseProcessorCPU<FGQ> for SynapseDynamicsNoPreCondenseRayon
where
    FGQ: FeagiGlobalQuantization,
{
    fn process_phase(data: &mut BurstEngineDataRayon<FGQ>) -> BurstEngineJustCompletedPhase {
        let current_burst_index = data.burst_index;

        // Raw pointers are required because Rayon's for_each closure must be Fn, not FnMut
        // Mutable borrows of data fields cannot be captured so disjoint parallel writes are
        // routed through pointers whose targets are guaranteed notoverlapping by precomputed indexes
        let neuron_engine_cortical_indexes = ParallelSlicePtr::new(&data.neuron_engine_cortical_indexes);
        let cortical_context_lookups = ParallelSlicePtr::new(&data.cortical_context_lookups);
        let cortical_layouts_dimensional = ParallelSlicePtr::new(&data.cortical_layouts.dimensional);
        let neuron_model_cortical_data = ParallelSlicePtr::new(&data.neuron_model_cortical_data);
        let neuron_history = ParallelMutSlicePtr::new(&mut data.neuron_history);
        let neuron_model_neuron_data = ParallelMutSlicePtr::new(&mut data.neuron_model_neuron_data);
        let neuron_potentials = ParallelMutSlicePtr::new(&mut data.neuron_potentials.float_32);

        data.neuron_fcls.float_32.par_iter_mut().enumerate().for_each(|(neuron_type_index_u, fcl_value)| {
            if *fcl_value == NPUWrappedNeuronMembranePotential::QUANT_ZERO {
                return;
            }

            let neuron_mp_quant_index = NPUWrappedNeuronMPQuantIndex::wrap(
                FGQ::NeuronIndexCountQuant::from_usize_unchecked(neuron_type_index_u),
            );
            let neuron_mp_quant_index_usize = neuron_mp_quant_index.to_usize();

            // SAFETY: Parallel iterations mutate disjoint index ranges. Read-only slices are never written.
            // All pointers originate from `data` and remain valid for the duration of this `par_iter_mut`.
            unsafe {
                let engine_cortical_area_index_of_neuron =
                    neuron_engine_cortical_indexes.get(neuron_mp_quant_index_usize);

                let cortical_context_lookup =
                    cortical_context_lookups.get(engine_cortical_area_index_of_neuron.to_usize());

                let neuron_index_local = NPUWrappedNeuronCorticalLocalIndex::wrap(neuron_mp_quant_index.unwrap())
                    - cortical_context_lookup.mp_quant_to_local_neuron_index_offset;

                let neuron_history =
                    neuron_history.get_mut(cortical_context_lookup.mp_quant_to_neuron_history_index_offset.to_usize());

                // TODO not always dimensional!
                let dimensional =
                    cortical_layouts_dimensional.get(cortical_context_lookup.cortical_layout_index.to_usize());
                let feagi_standard_cortical_data = neuron_model_cortical_data
                    .get(cortical_context_lookup.neuron_model_cortical_data_index.to_usize());

                let neuron_model_data = neuron_model_neuron_data.get_mut(neuron_mp_quant_index_usize);
                let neuron_potential = neuron_potentials.get_mut(neuron_mp_quant_index_usize);
            }
        }
        );

                                                                      BurstEngineJustCompletedPhase::SynapseDynamics
    }
}
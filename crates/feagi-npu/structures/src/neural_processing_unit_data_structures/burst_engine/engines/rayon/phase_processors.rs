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

        data.neuron_fcls.float_32.par_iter_mut().enumerate().for_each(|(neuron_type_index_u, fcl_value)| {
            if *fcl_value == NPUWrappedNeuronMembranePotential::QUANT_ZERO {
                return
            }

            let current_burst_index = &data.burst_index;

            let neuron_mp_quant_index = NPUWrappedNeuronMPQuantIndex::wrap(
                FGQ::NeuronIndexCountQuant::from_usize_unchecked(neuron_type_index_u),
            );



            let engine_cortical_area_index_of_neuron = &data.neuron_engine_cortical_indexes[neuron_mp_quant_index.to_usize()];

            let cortical_context_lookup = &data.cortical_context_lookups[engine_cortical_area_index_of_neuron.to_usize()];

            let neuron_index_local = NPUWrappedNeuronCorticalLocalIndex::wrap(neuron_mp_quant_index.unwrap()) - cortical_context_lookup.mp_quant_to_local_neuron_index_offset;
            let mut neuron_history = &mut data.neuron_history[cortical_context_lookup.neuron_history_index.to_usize()];

            // TODO not always dimensional!
            let dimensional = &data.cortical_layouts.dimensional[cortical_context_lookup.cortical_layout_index.to_usize()];
            let feagi_standard_cortical_data = &data.neuron_model_cortical_data[cortical_context_lookup.neuron_model_cortical_data_index.to_usize()];

            let neuron_model_data = &mut data.neuron_model_neuron_data[neuron_mp_quant_index.to_usize()];

            let neuron_potential = &mut data.neuron_potentials.float_32[neuron_mp_quant_index.to_usize()];


            let is_firing = FeagiStandardModelProcessorCPU::process_neuron_potential_for_dimensional_cortical_configuration(
                fcl_value,
                &neuron_index_local,
                current_burst_index,
                &neuron_history.burst_index_of_last_input,
                &neuron_history.burst_index_of_last_firing,
                dimensional,
                feagi_standard_cortical_data,
                neuron_model_data,
                neuron_potential
            );

            *fcl_value = NPUWrappedNeuronMembranePotential::QUANT_ZERO;

            neuron_history.burst_index_of_last_input = *current_burst_index;
            if is_firing {
                neuron_history.burst_index_of_last_firing = *current_burst_index;
            }

        });




        BurstEngineJustCompletedPhase::UpdateFiringNeuronBitfield
    }
}
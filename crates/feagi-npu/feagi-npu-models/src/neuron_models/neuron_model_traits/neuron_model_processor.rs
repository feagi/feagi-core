use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::extendable_quantizations::NeuronModelQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_npu_common::descriptors::cortical_area_descriptors::CorticalAreaLayoutDataDimensional;
use feagi_npu_common::wrapped_values::{BurstIndex, NeuronCorticalIndex, NeuronMembranePotential};
use crate::neuron_models::neuron_model_traits::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};

/// Root base trait for defining neuron firing and other dynamics. Does NOT store actual data,
pub trait NeuronModelProcessorBase<FGQ, NMQ, NMCD, NMND>:
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>
{
    /// TODO neuron init func?
    
    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_cortical_data_for_burst_index_rollover(
        &self,
        _cortical_area_data: &mut NMCD)
    {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }

    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_neuron_data_for_burst_index_rollover(
        &self,
        _neuron_linear_index: &NeuronCorticalIndex<FGQ::NeuronIndexCountQuant>,
        _neuron_model_data: &mut NMND)
    {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }
}

pub trait NeuronModelProcessorWithoutHistory<FGQ, NMQ, NMCD, NMND>:
NeuronModelProcessorBase<FGQ, NMQ, NMCD, NMND>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>
{
    /// Neuron received input potential. Process it, updating any internal states and update
    /// this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false
    fn process_neuron_potential_for_dimensional_cortical_configuration
    (
        incoming_potential: &NeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NeuronCorticalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FGQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalAreaLayoutDataDimensional<FGQ>,
        cortical_area_data: &NMCD,
        neuron_model_data: &mut NMND,
        this_neuron_potential: &mut NeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    ) -> bool;
}


pub trait NeuronModelProcessorWithHistory<FGQ, NMQ, NMCD, NMND>:
NeuronModelProcessorBase<FGQ, NMQ, NMCD, NMND>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>
{
    fn process_neuron_potential_for_dimensional_cortical_configuration
    (
        incoming_potential: &NeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NeuronCorticalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_activity: &BurstIndex<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_firing: &BurstIndex<FGQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalAreaLayoutDataDimensional<FGQ>,
        cortical_area_data: &NMCD,
        neuron_model_data: &mut NMND,
        this_neuron_potential: &mut NeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    ) -> bool;
}

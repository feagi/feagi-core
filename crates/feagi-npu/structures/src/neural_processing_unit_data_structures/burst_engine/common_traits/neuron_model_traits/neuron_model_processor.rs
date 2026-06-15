use feagi_structures::feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use feagi_structures::feagi_data::quantization_levels::extendable_quantizations::NeuronModelQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::descriptor_flags::cortical_area_layout::CorticalLayoutDimensionalCPU;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedNeuronCorticalLocalIndex, NPUWrappedNeuronMembranePotential};
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::neuron_model_traits::neuron_model_cortical_data::{NeuronModelCorticalData, NeuronModelCorticalDataCPU};
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::neuron_model_traits::neuron_model_neuron_data::{NeuronModelNeuronData, NeuronModelNeuronDataCPU};
use crate::npu_descriptors::NPUGlobalBurstCounter;

/// Root base trait for defining neuron firing and other dynamics. Does NOT store actual data,
pub trait NeuronModelProcessorBase<FGQ, NMQ, NMCD, NMND>:
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>
{
    // Methods for
    // blank neuron instantiation
    // resetting cortical/neuron fields for burst index rollover
}

pub trait NeuronModelProcessor<FGQ, NMQ, NMCD, NMND>:
NeuronModelProcessorBase<FGQ, NMQ, NMCD, NMND>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>
{
    // Methods for
    // neuron firing (for various cortical configuration types),
}


pub trait NeuronModelProcessorWithHistory<FGQ, NMQ, NMCD, NMND>:
NeuronModelProcessorBase<FGQ, NMQ, NMCD, NMND>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>
{
    // Methods for
    // neuron firing (for various cortical configuration types),

}



//region CPU Traits

/// Root base trait for defining neuron firing and other dynamics on the CPU.
/// Does NOT store actual data
pub trait NeuronModelProcessorBaseCPU<FGQ, NMQ, NMCD, NMND>:
NeuronModelProcessorBase<FGQ, NMQ, NMCD, NMND>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalDataCPU<NMQ>,
    NMND: NeuronModelNeuronDataCPU<NMQ>
{
    
    // TODO creation may need to be removed, it doesnt belong here

    //region Dimensional Layout

    /// Creates / inits a neuron in a dimensional cortical area
    fn create_blank_cortical_area_of_cortical_configuration_dimensional(
        cortical_area_layout: &CorticalLayoutDimensionalCPU<FGQ, NMQ::CorticalPotentialQuant>,
    ) -> NMCD;

    /// Creates / inits a neuron in a dimensional cortical area
    fn create_blank_neuron_of_cortical_configuration_dimensional(
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_area_layout: &CorticalLayoutDimensionalCPU<FGQ, NMQ::CorticalPotentialQuant>,
        cortical_area_data: &NMCD,
    ) -> NMND;

    //endregion
    
    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_cortical_data_for_burst_index_rollover(
        &self,
        cortical_area_data: &mut NMCD)
    {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }

    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_neuron_data_for_burst_index_rollover(
        &self,
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        neuron_model_data: &mut NMND)
    {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }

}

/// trait for defining neuron firing and other dynamics on the CPU.
/// Does NOT store actual data
pub trait NeuronModelProcessorCPU<FGQ, NMQ, NMCD, NMND>:
NeuronModelProcessorBaseCPU<FGQ, NMQ, NMCD, NMND>
+ NeuronModelProcessorWithHistory<FGQ, NMQ, NMCD, NMND>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalDataCPU<NMQ>,
    NMND: NeuronModelNeuronDataCPU<NMQ>
{
    /// Neuron received input potential. Process it, updating any internal states and update
    /// this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false
    fn process_neuron_potential_for_dimensional_cortical_configuration
    (
        incoming_potential: &NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalLayoutDimensionalCPU<FGQ, NMQ::CorticalPotentialQuant>,
        cortical_area_data: &NMCD,
        neuron_model_data: &mut NMND,
        this_neuron_potential: &mut NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    ) -> bool;
}


/// trait for defining neuron firing and other dynamics on the CPU.
/// Does NOT store actual data
pub trait NeuronModelProcessorWithBurstHistoryCPU<FGQ, NMQ, NMCD, NMND>:
NeuronModelProcessorBaseCPU<FGQ, NMQ, NMCD, NMND>
+ NeuronModelProcessorWithHistory<FGQ, NMQ, NMCD, NMND>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalDataCPU<NMQ>,
    NMND: NeuronModelNeuronDataCPU<NMQ>
{
    /// Neuron received input potential. Process it, updating any internal states and update
    /// this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false
    fn process_neuron_potential_for_dimensional_cortical_configuration
    (
        incoming_potential: &NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_activity: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        burst_index_of_last_firing: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_layout_dimensional: &CorticalLayoutDimensionalCPU<FGQ, NMQ::CorticalPotentialQuant>,
        cortical_area_data: &NMCD,
        neuron_model_data: &mut NMND,
        this_neuron_potential: &mut NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    ) -> bool;
}



//endregion
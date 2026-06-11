use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_types::cortical_area_layout::{CorticalConfigurationBase, CorticalConfigurationDimensional};
use crate::neural_processing_unit_data_structures::cpu_wrappers::{NPUWrappedNeuronCorticalLocalIndex, NPUWrappedNeuronMembranePotential};
use crate::neural_processing_unit_data_structures::neuron_models::neuron_model_cortical_data::{NeuronModelCorticalData, NeuronModelCorticalDataCPU};
use crate::neural_processing_unit_data_structures::neuron_models::neuron_model_neuron_data::{NeuronModelNeuronData, NeuronModelNeuronDataCPU};
use crate::npu_descriptors::NPUGlobalBurstCounter;

/// Root base trait for defining neuron firing and other dynamics. Does NOT store actual data,
pub trait NeuronModelProcessor<FGQ, NMQ, NMCD, NMND>:
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<FGQ, NMQ>,
    NMND: NeuronModelNeuronData<FGQ, NMQ, NMCD>
{
    // Methods for
    // blank neuron instantiation
    // neuron firing (for various cortical configuration types),
    // resetting cortical/neuron fields for burst index rollover

}




//region CPU Traits

/// Root base trait for defining neuron firing and other dynamics on the CPU.
/// Does NOT store actual data
pub trait NeuronModelProcessorCPU<FGQ, NMQ, NMCD, NMND>:
NeuronModelProcessor<FGQ, NMQ, NMCD, NMND>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalDataCPU<FGQ, NMQ>,
    NMND: NeuronModelNeuronDataCPU<FGQ, NMQ, NMCD>
{
    /// Neuron received input potential. Process it, updating any internal states and update
    /// this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false
    fn process_neuron_potential_for_dimensional_cortical_configuration
    (
        &self,
        incoming_potential: &NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>,
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_area_dimensional_configuration: &CCB,
        cortical_area_data: &NMCD,
        neuron_model_data: &mut NMND,
        this_neuron_potential: &mut NPUWrappedNeuronMembranePotential<<NMQ::CorticalPotentialQuant as CorticalPotentialQuantization>::NeuronPotentialQuant>
    ) -> bool;



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

//endregion
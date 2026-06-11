use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_types::cortical_area_layout::CorticalConfigurationBase;
use crate::neural_processing_unit_data_structures::cpu_wrappers::NPUWrappedNeuronCorticalLocalIndex;
use crate::neural_processing_unit_data_structures::neuron_models::neuron_model_cortical_data::{NeuronModelCorticalData, NeuronModelCorticalDataCPU};
use crate::npu_descriptors::NPUGlobalBurstCounter;

/// Root trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model. This should be extended with only the per neuron data
pub trait NeuronModelNeuronData<FGQ, NMQ, NMCD>:
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<FGQ, NMQ>
{
    // As per CorticalAreasIndexQuantization, this takes in GlobalBurstIndexQuant,
    // and NeuronIndexCountQuant. These are not settable by the model and instead picked by
    // FEAGI's NPU

    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    // Implement any per-neuron level data members

    // No methods!
}


//region CPU Specific Trait


/// Root CPU trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model
pub trait NeuronModelNeuronDataCPU<FGQ, NMQ, NMCD>:
NeuronModelNeuronData<FGQ, NMQ, NMCD>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalDataCPU<FGQ, NMQ>,
{
    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    // Implement any per-neuron level data members (or make members pub)
    
    /// Creates / inits a neuron in a dimensional cortical area
    fn create_blank_neuron_of_cortical_configuration_dimensional(
        neuron_linear_index: &NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_area_configuration: &CCB,
        cortical_area_data: &NMCD,
    ) -> Self;

}






//endregion
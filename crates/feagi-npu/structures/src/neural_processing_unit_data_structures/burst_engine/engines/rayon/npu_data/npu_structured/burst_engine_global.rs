use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::descriptor_flags::NeuronModelQuantDescriptorsCPU;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedFCLCMPQuantIndex, NPUWrappedNeuronIndexBurstEngineIndex, NPUWrappedNeuronMPQuantIndex};

/// Stores a burst engine level neuron index with the quant flag, for trivial conversion to
/// the mp quant level. This is useful for arrays that should be iterated on the engine level but
/// we need ot access the quant level neurons rapidly
pub struct BurstEngineNeuronIndexWithQuant<FGQ: FeagiGlobalQuantization>
{
    pub burst_index: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
    pub quant_flag: NeuronModelQuantDescriptorsCPU
}

/// Exists for every neuron that has multiple inputs, defines the region of the FCLC to sum to
/// get the fcl value (which is to be stored at the neuron index). MP quant typed so flag is given,
/// although this array itself is burst engine global for processing reasons
pub struct FCLMappingsToFCLC<FGQ: FeagiGlobalQuantization>
{
    FCLC_start_index: NPUWrappedFCLCMPQuantIndex<FGQ::NeuronIndexCountQuant>,
    FCLC_length: NPUWrappedFCLCMPQuantIndex<FGQ::NeuronIndexCountQuant>,
    FCL_neuron_index: NPUWrappedNeuronMPQuantIndex<FGQ::NeuronIndexCountQuant>,
    neuron_mp_type_flag: NeuronModelQuantDescriptorsCPU // due to padding, we will always have some free bytes, so might as well...
}



/// Denotes the last time a specific neuron fired or had an input activity at all. As not all
/// neuron models use this, has its own indexing
pub struct NeuronHistory<FGQ: FeagiGlobalQuantization>
{
    pub burst_index_of_last_input: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
    pub burst_index_of_last_firing: NPUWrappedNeuronIndexBurstEngineIndex<FGQ::NeuronIndexCountQuant>,
}
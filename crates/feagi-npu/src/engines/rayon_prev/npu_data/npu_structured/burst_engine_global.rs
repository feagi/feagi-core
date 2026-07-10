use feagi_structures::feagi_data::create_quantized_index_count_wrapper;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::cortical_area_layout::{CorticalLayoutBase, CorticalLayoutDimensional};
use crate::neural_processing_unit_data_structures::burst_engine::descriptor_flags::NeuronModelQuantDescriptorsCPU;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedBurstEngineBurstIndex, NPUWrappedCorticalAreaDimensions, NPUWrappedCorticalLayoutIndex, NPUWrappedFCLCMPQuantIndex, NPUWrappedNeuronCorticalLocalIndex, NPUWrappedNeuronIndexBurstEngineIndex, NPUWrappedNeuronMPQuantIndex, NPUWrappedNeuronNeuronModelMPQuantIndex};

create_quantized_index_count_wrapper!(NPUWrappedNeuronHistoryIndex);
create_quantized_index_count_wrapper!(NPUWrappedCorticalContextLookupIndex);
create_quantized_index_count_wrapper!(NPUWrappedEngineSynapseIndexLength);


/// Stores a burst engine level neuron index with the quant flag, for trivial conversion to
/// the mp quant level. This is useful for arrays that should be iterated on the engine level but
/// we need ot access the quant level neurons rapidly
pub struct BurstEngineNeuronIndexWithQuant<FIQ: FeagiGlobalQuantization>
{
    pub burst_index: NPUWrappedNeuronIndexBurstEngineIndex<FIQ::NeuronIndexCountQuant>,
    pub quant_flag: NeuronModelQuantDescriptorsCPU
}

/// Exists for every neuron that has multiple inputs, defines the region of the FCLC to sum to
/// get the fcl value (which is to be stored at the neuron index). MP quant typed so flag is given,
/// although this array itself is burst engine global for processing reasons
pub struct FCLMappingsToFCLC<FIQ: FeagiGlobalQuantization>
{
    FCLC_start_index: NPUWrappedFCLCMPQuantIndex<FIQ::NeuronIndexCountQuant>,
    FCLC_length: NPUWrappedFCLCMPQuantIndex<FIQ::NeuronIndexCountQuant>,
    FCL_neuron_index: NPUWrappedNeuronMPQuantIndex<FIQ::NeuronIndexCountQuant>,
    neuron_mp_type_flag: NeuronModelQuantDescriptorsCPU // due to padding, we will always have some free bytes, so might as well...
}



/// Denotes the last time a specific neuron fired or had an input activity at all. As not all
/// neuron models use this, has its own indexing
pub struct NeuronHistory<FIQ: FeagiGlobalQuantization>
{
    pub burst_index_of_last_input: NPUWrappedBurstEngineBurstIndex<FIQ::GlobalBurstIndexQuant>,
    pub burst_index_of_last_firing: NPUWrappedBurstEngineBurstIndex<FIQ::GlobalBurstIndexQuant>,
}

///
pub struct CorticalLayouts<FIQ>
where
    FIQ: FeagiGlobalQuantization,
{
    pub dimensional: Vec<CorticalLayoutDimensionalCPU<FIQ>>,
}

//region Sub Elements



impl<FIQ> CorticalLayoutDimensionalCPU<FIQ>
where
    FIQ: FeagiGlobalQuantization,
{
    pub fn new(dimensions: NPUWrappedCorticalAreaDimensions<FIQ::NeuronIndexCountQuant>) -> Self {
        Self { dimensions }
    }
}

impl<FIQ> CorticalLayoutBase<FIQ> for CorticalLayoutDimensionalCPU<FIQ>
where FIQ: FeagiGlobalQuantization, {}

impl<FIQ> CorticalLayoutDimensional<FIQ> for CorticalLayoutDimensionalCPU<FIQ>
where FIQ: FeagiGlobalQuantization, {}



// TODO other types?

//endregion


/// Contains indexes and offsets for various properties of a cortical_area area. Indexed by
/// Engine Cortical Index
pub struct CorticalContextLookup<FIQ: FeagiGlobalQuantization>
{
    // NOTE: For byte alignment reasons, put neuron stuff first, as neuron quantization >= cortical_area area quantization
    /// Subtract the this from a neurons mp quant index to the get the cortical_area area local index
    pub mp_quant_to_local_neuron_index_offset: NPUWrappedNeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    pub mp_quant_to_neuron_history_index_offset: NPUWrappedNeuronHistoryIndex<FIQ::NeuronIndexCountQuant>, // Only valid if the neuron model needs history. Otherwise this will just be 0

    pub cortical_layout_index: NPUWrappedCorticalLayoutIndex<FIQ::CorticalAreaIndexCountQuant>, // Neuron Flags will disclose what type of layout
    pub neuron_model_cortical_data_index: NPUWrappedNeuronNeuronModelMPQuantIndex<FIQ::CorticalAreaIndexCountQuant>,
    // NOTE: Base psp potential is a separate array with 1-1 cortical_area engine index lookup, we don't need it here
}


pub struct SynapseRangeMappingFromNeuron<FIQ: FeagiGlobalQuantization>
{
    pub synapse_start_index: NPUWrappedEngineSynapseIndexLength<FIQ::SynapseIndexCountQuant>,
    pub synapse_start_length: NPUWrappedEngineSynapseIndexLength<FIQ::SynapseIndexCountQuant>,
    pub source_neuron_index: NPUWrappedNeuronIndexBurstEngineIndex<FIQ::NeuronIndexCountQuant>,
    //pub source_neuron_quant_descriptor: NeuronModelQuantDescriptorsCPU // TODO
}

pub struct SynapseDef {
    weight: f32
}
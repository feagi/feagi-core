use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_types::descriptor_flags::{NeuronModelCorticalDescriptorsCPU, NeuronModelQuantDescriptorsCPU};
use crate::neural_processing_unit_data_structures::cpu_wrappers::NPUWrappedCorticalAreaBurstEngineIndex;

/// For each engine neuron index, map to the engine cortical index, the neuron model cortical
/// descriptor, and the neuron model quant descriptor
pub trait CorticalDescriptorLookupTable<FGQ: FeagiGlobalQuantization> {}

//region CPU implementation

#[repr(C)]
pub struct CorticalDescriptorLookupTableCPU<FGQ: FeagiGlobalQuantization, const ELEMENT_END_PADDING: usize>
{
    pub cortical_descriptors: Vec<EngineNeuronIndexToCorticalDescriptorElementCPU<FGQ, ELEMENT_END_PADDING>>,
    _padding: [u8; 8]
}

impl<FGQ: FeagiGlobalQuantization, const ELEMENT_END_PADDING: usize> CorticalDescriptorLookupTable<FGQ> for CorticalDescriptorLookupTableCPU<FGQ, ELEMENT_END_PADDING> {}

#[repr(C)]
pub struct EngineNeuronIndexToCorticalDescriptorElementCPU<FGQ: FeagiGlobalQuantization, const END_PADDING: usize> {
    pub engine_cortical_index: NPUWrappedCorticalAreaBurstEngineIndex<FGQ::CorticalAreaIndexCountQuant>,
    pub neuron_model_cortical_descriptor: NeuronModelCorticalDescriptorsCPU,
    pub neuron_model_quant_descriptor: NeuronModelQuantDescriptorsCPU,
    _padding: [u8; END_PADDING],
}

//endregion
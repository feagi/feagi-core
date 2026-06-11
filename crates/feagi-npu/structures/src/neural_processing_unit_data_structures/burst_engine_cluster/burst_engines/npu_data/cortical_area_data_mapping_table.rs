use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::cpu_wrappers::{NPUWrappedCorticalAreaBurstEngineIndex, NPUWrappedCorticalLayoutLayoutIndex, NPUWrappedNeuronCorticalLocalIndex, NPUWrappedNeuronNeuronModelMPQuantIndex};

/// Indexed by Engine Cortical Index, returns a struct that denotes offsets for converting engine
/// neuron indexes into other neuron indexes local and relevant for that cortical area
pub trait CorticalAreaDataMappingTable<FGQ: FeagiGlobalQuantization> {}





pub struct CorticalAreaDataMappingTableCPU<FGQ: FeagiGlobalQuantization, const ELEMENT_END_PADDING: usize> {
    mappings: Vec<CorticalAreaDataMappingElementCPU<FGQ, ELEMENT_END_PADDING>>,
}



pub struct CorticalAreaDataMappingElementCPU<FGQ: FeagiGlobalQuantization, const END_PADDING: usize>
{
    pub neuron_engine_index_to_cortical_local_offset: NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
    pub neuron_engine_index_to_model_quant_offset: NPUWrappedNeuronNeuronModelMPQuantIndex<FGQ::NeuronIndexCountQuant>,
    pub number_neurons_in_this_cortical_area: NPUWrappedNeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>,
    pub cortical_layout_index: NPUWrappedCorticalLayoutLayoutIndex<FGQ::CorticalAreaIndexCountQuant>,
    _padding: [u8; END_PADDING],
}

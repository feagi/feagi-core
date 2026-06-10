use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::calculate_struct_padding::calculate_byte_alignment_padding;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::cortical_neuron::NPUNeuronIndexQuantizationLocal;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::fcl_extensions::NPUPrimaryFCLCQuantizationLocalIndex;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::packed_cortical_descriptor::PackedCorticalDescriptor;

#[repr(C)]
pub struct FCLCToFCLMappingTableCPU<FGQ: FeagiGlobalQuantization>
{
    pub mappings: Vec<FCLCToFCLMappingElementCPU<FGQ>>
}


#[repr(C)]
pub struct FCLCToFCLMappingElementCPU<FGQ: FeagiGlobalQuantization>
{
    pub fclc_read_start_index: NPUPrimaryFCLCQuantizationLocalIndex<FGQ::FireCandidateListCacheIndexCountQuant>,
    pub fclc_read_length: NPUPrimaryFCLCQuantizationLocalIndex<FGQ::FireCandidateListCacheIndexCountQuant>,
    pub fcl_target_index: NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant>,
    pub packed_cortical_descriptor: PackedCorticalDescriptor,
    _padding: [u8; calculate_byte_alignment_padding((size_of::<u32>() * 3) + size_of::<u8>())],
}







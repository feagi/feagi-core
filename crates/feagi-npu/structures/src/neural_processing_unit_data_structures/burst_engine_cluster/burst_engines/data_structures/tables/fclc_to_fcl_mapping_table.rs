use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
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
    _padding: [u8; Self::END_PADDING_LENGTH],
}


impl<FGQ: FeagiGlobalQuantization> FCLCToFCLMappingElementCPU<FGQ>
{
    const END_PADDING_LENGTH: usize = Self::calculate_end_padding();

    pub const fn calculate_end_padding() -> usize
    {
        let cur_size: usize = 4 + 4 + 4 + 1; // TODO actual logic!
        if cur_size <= 4 {
            return 4 - cur_size
        } else if cur_size <= 8 {
            return 8 - cur_size
        } else if cur_size <= 16 {
            return 16 - cur_size
        }
        32 - cur_size
    }
}






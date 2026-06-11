use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::calculate_struct_padding::calculate_byte_alignment_padding;
use crate::neural_processing_unit_data_structures::cpu_wrappers::{NPUWrappedFCLCMPQuantIndex, NPUWrappedNeuronMPQuantIndex};
use crate::neural_processing_unit_data_structures::packed_cortical_descriptor::PackedCorticalDescriptor;

pub trait FCLCToFCLMappingTable<FGQ: FeagiGlobalQuantization> {}

/// A Read Only table that
#[repr(C)]
pub struct FCLCToFCLMappingTableCPU<FGQ: FeagiGlobalQuantization, const ELEMENT_END_PADDING_SIZE: usize>
{
    pub float_32: Vec<FCLCToFCLMappingElementCPU<FGQ, ELEMENT_END_PADDING_SIZE>>,
    _padding_1: [u8; calculate_byte_alignment_padding(size_of::<Vec<u8>>())], // data type irrelevant, have one per vector
    // TODO f8
    // TODO f16
    // TODO f64
    // TODO u8?
    // NOTE: all vectors will have the same element end padding
}

impl<FGQ: FeagiGlobalQuantization, const ELEMENT_END_PADDING_SIZE: usize> FCLCToFCLMappingTable<FGQ> for FCLCToFCLMappingTableCPU<FGQ, ELEMENT_END_PADDING_SIZE> {}



#[repr(C)]
pub struct FCLCToFCLMappingElementCPU<FGQ: FeagiGlobalQuantization, const END_PADDING_SIZE: usize>
{
    pub fclc_read_start_index: NPUWrappedFCLCMPQuantIndex<FGQ::FireCandidateListCacheIndexCountQuant>,
    pub fclc_read_length: NPUWrappedFCLCMPQuantIndex<FGQ::FireCandidateListCacheIndexCountQuant>,
    pub fcl_target_index: NPUWrappedNeuronMPQuantIndex<FGQ::NeuronIndexCountQuant>,
    pub packed_cortical_descriptor: PackedCorticalDescriptor,
    _padding: [u8; END_PADDING_SIZE],
}







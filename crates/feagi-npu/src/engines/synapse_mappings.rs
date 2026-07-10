use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;

#[repr(C)]
pub(crate) struct NPUSynapseMappingOneToOneCPU<FIQ>
where
    FIQ: FeagiGlobalQuantization
{
    pub fclc_index_start: FIQ::SynapseIndexCountQuant,
    pub fclc_read_length: FIQ::SynapseIndexCountQuant,
    pub synapse_one_to_one_map_index_start: FIQ::SynapseIndexCountQuant,
    pub synapse_one_to_one_map_read_length: FIQ::SynapseIndexCountQuant,
}
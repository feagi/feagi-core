use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;

#[repr(C)]
pub(crate) struct NPUSynapseMappingOneToOneCPU<FGQ>
where
    FGQ: FeagiGlobalQuantization
{
    pub fclc_index_start: FGQ::SynapseIndexCountQuant,
    pub fclc_read_length: FGQ::SynapseIndexCountQuant,
    pub synapse_one_to_one_map_index_start: FGQ::SynapseIndexCountQuant,
    pub synapse_one_to_one_map_read_length: FGQ::SynapseIndexCountQuant,
}
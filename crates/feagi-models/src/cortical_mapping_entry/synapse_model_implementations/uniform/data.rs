use feagi_data::create_wrapped_quantized_decimal;
use crate::cortical_mapping_entry::synapse::cortical_mapping_entry_data::SynapseModelCorticalMappingEntryData;
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelQuantization;
create_wrapped_quantized_decimal!(
    /// A multiplier synapse value, applies some scale to incoming signal
   pub UniformSynapseMultiplier);

#[derive(Debug, Clone, Default, Copy)]
pub struct UniformSynapseModelCorticalMappingEntryData<SMQ>
where
    SMQ: UniformSynapseModelQuantization,
{
    pub post_synaptic_multiplier: UniformSynapseMultiplier<SMQ::JunctionPotentialQuant>,
}

impl<SMQ> SynapseModelCorticalMappingEntryData<SMQ> for UniformSynapseModelCorticalMappingEntryData<SMQ> where SMQ: UniformSynapseModelQuantization {}

impl<SMQ> UniformSynapseModelCorticalMappingEntryData<SMQ>
where
    SMQ: UniformSynapseModelQuantization,
{
    pub fn new(post_synaptic_multiplier: UniformSynapseMultiplier<SMQ::JunctionPotentialQuant>) -> Self {
        Self { post_synaptic_multiplier }
    }
}


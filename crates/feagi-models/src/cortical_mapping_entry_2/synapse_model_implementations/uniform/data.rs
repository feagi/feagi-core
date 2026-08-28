use crate::cortical_mapping_entry::synapse::cortical_mapping_entry_data::SynapseModelCorticalMappingEntryData;
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelQuantization;
use feagi_data::create_wrapped_quantized_decimal;
use feagi_data::values::quantizable::QuantizedDecimalTrait;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

create_wrapped_quantized_decimal!(
    /// A multiplier synapse value, applies some scale to incoming signal
   pub UniformSynapseMultiplier);

impl<Q> Serialize for UniformSynapseMultiplier<Q>
where
    Q: QuantizedDecimalTrait + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.deref().serialize(serializer)
    }
}

impl<'de, Q> Deserialize<'de> for UniformSynapseMultiplier<Q>
where
    Q: QuantizedDecimalTrait + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Q::deserialize(deserializer).map(Self::new)
    }
}

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

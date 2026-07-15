use crate::synapse_models::models::uniform_weight::quantization::UniformSynapseModelQuantization;
use crate::synapse_models::shared::data::SynapseModelAxonBundleData;
use feagi_data::create_wrapped_quantized_decimal;

create_wrapped_quantized_decimal!(
    /// A multiplier synapse value, applies some scale to incoming signal
   pub UniformSynapseMultiplier);

// This synapse model has no per synapse data at all

#[derive(Debug, Clone)]
pub struct BasicSynapseModelAxonBundleData<SMQ>
where
    SMQ: UniformSynapseModelQuantization,
{
    pub multiplier: UniformSynapseMultiplier<SMQ::MultiplierQuant>,
}

impl<SMQ> SynapseModelAxonBundleData<SMQ> for BasicSynapseModelAxonBundleData<SMQ> where SMQ: UniformSynapseModelQuantization {}

impl<SMQ> BasicSynapseModelAxonBundleData<SMQ>
where
    SMQ: UniformSynapseModelQuantization,
{
    pub fn new(multiplier: UniformSynapseMultiplier<SMQ::MultiplierQuant>) -> Self {
        Self { multiplier }
    }
}

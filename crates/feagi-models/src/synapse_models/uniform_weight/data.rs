use crate::synapse_models::uniform_weight::quantization::BasicSynapseModelQuantization;
use crate::synapse_models::shared::synapse_model_data::SynapseModelAxonBundleData;
use feagi_data::create_wrapped_quantized_decimal;

create_wrapped_quantized_decimal!(
    /// A multiplier synapse value, applies some scale to incoming signal
   pub UniformSynapseMultiplier);

#[derive(Debug, Copy, Clone)]
pub struct BasicSynapseModelAxonBundleData<SMQ>
where
    SMQ: BasicSynapseModelQuantization,
{
    pub multiplier: UniformSynapseMultiplier<SMQ::MultiplierQuant>,
}

impl<SMQ> SynapseModelAxonBundleData<SMQ> for BasicSynapseModelAxonBundleData<SMQ> where SMQ: BasicSynapseModelQuantization {}

impl<SMQ> BasicSynapseModelAxonBundleData<SMQ>
where
    SMQ: BasicSynapseModelQuantization,
{
    pub fn new(multiplier: UniformSynapseMultiplier<SMQ::MultiplierQuant>) -> Self {
        Self { multiplier }
    }
}

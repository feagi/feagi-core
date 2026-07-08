use crate::synapse_models::basic_synapse::quantization::BasicSynapseModelQuantization;
use crate::synapse_models::synapse_model_traits::synapse_model_data::SynapseModelAxonBundleData;
use feagi_data::create_wrapped_quantized_decimal;

create_wrapped_quantized_decimal!(
    /// A multiplier synapse value, applies some scale to incoming signal
   pub BasicSynapseMultiplier);

#[derive(Debug, Copy, Clone)]
pub struct BasicSynapseModelAxonBundleData<SMQ>
where
    SMQ: BasicSynapseModelQuantization,
{
    pub multiplier: BasicSynapseMultiplier<SMQ::MultiplierQuant>,
}

impl<SMQ> SynapseModelAxonBundleData<SMQ> for BasicSynapseModelAxonBundleData<SMQ> where
    SMQ: BasicSynapseModelQuantization
{
}

impl<SMQ> BasicSynapseModelAxonBundleData<SMQ>
where
    SMQ: BasicSynapseModelQuantization,
{
    pub fn new(multiplier: BasicSynapseMultiplier<SMQ::MultiplierQuant>) -> Self {
        Self { multiplier }
    }
}

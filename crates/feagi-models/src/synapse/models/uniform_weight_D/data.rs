use crate::synapse::models::uniform_weight_D::quantization::UniformSynapseModelQuantization;
use crate::synapse::synapse_model_data::SynapseModelAxonBundleData;


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

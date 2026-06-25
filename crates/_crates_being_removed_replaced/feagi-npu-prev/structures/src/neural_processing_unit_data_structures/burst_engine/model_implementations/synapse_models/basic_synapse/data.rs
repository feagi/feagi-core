use feagi_structures::feagi_data::create_quantized_decimal_wrapper;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::synapse_model_traits::synapse_model_axon_bundle_data::{SynapseModelAxonBundleData, SynapseModelAxonBundleDataCPU};
use crate::neural_processing_unit_data_structures::burst_engine::model_implementations::synapse_models::basic_synapse::quantization::BasicSynapseModelQuantization;

create_quantized_decimal_wrapper!(NPUWrappedBasicSynapseMultiplier);

#[derive(Debug, Copy, Clone)]
pub struct BasicSynapseModelAxonBundleDataCPU<SMQ>
where
    SMQ: BasicSynapseModelQuantization
{
    pub multiplier: NPUWrappedBasicSynapseMultiplier<SMQ::MultiplierQuant>,
}

impl<SMQ> SynapseModelAxonBundleData<SMQ> for BasicSynapseModelAxonBundleDataCPU<SMQ>
where
    SMQ: BasicSynapseModelQuantization,
{}

impl<SMQ> SynapseModelAxonBundleDataCPU<SMQ> for BasicSynapseModelAxonBundleDataCPU<SMQ>
where
    SMQ: BasicSynapseModelQuantization
{}

impl<SMQ> BasicSynapseModelAxonBundleDataCPU<SMQ>
where
    SMQ: BasicSynapseModelQuantization
{
    pub fn new(multiplier: NPUWrappedBasicSynapseMultiplier<SMQ::MultiplierQuant>) -> Self {
        Self {
            multiplier,
        }
    }
}

use feagi_structures::feagi_data::create_quantized_index_count_wrapper;
use feagi_structures::feagi_data::feagi_ecs::element::{FeagiECSElementDevice};
use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalIndexQuantization;



create_quantized_index_count_wrapper!(SynapseIndex);
create_quantized_index_count_wrapper!(AxonBundleIndex);


/// Root Synapse Data Trait. Denotes any data / context per synapse outside of neuron mapping
/// that is per synapse
pub trait SynapseDataCommon<FGIQ>:
FeagiECSElementDevice
where
    FGIQ: FeagiGlobalIndexQuantization,
{
    // Cant store data for unknown device

    // Extend this with the FeagiECS Element and then define your type
}


/// Root Axon Bundle Data Trait. Denotes any data / context per synapse outside of neuron mapping
/// that is per Axon Bundle
pub trait AxonBundleDataCommon<FGIQ, SynapseType>:
FeagiECSElementDevice
where
    FGIQ: FeagiGlobalIndexQuantization,
    SynapseType: SynapseDataCommon<FGIQ>,
{
    // Cant store data for unknown device
}

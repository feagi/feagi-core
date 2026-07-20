use crate::synapse::shared::quantization::{SynapseModelQuantization};

/// Root trait for all synapse axon bundle implementations, which is essentially any data related
/// to the synapses of a single mapping. This should be extended only with axon bundle level data
pub trait SynapseModelAxonBundleData<SMQ>: Clone
where
    SMQ: SynapseModelQuantization,
{
    // implement any axon bundle level data
}

/// Root trait for all synapse data implementations, which is essentially any data related
/// to each individual synapse of a mapping
pub trait SynapseModelSynapseData<SMQ>: Clone
where
    SMQ: SynapseModelQuantization,
{
    // implement any per synapse data
}

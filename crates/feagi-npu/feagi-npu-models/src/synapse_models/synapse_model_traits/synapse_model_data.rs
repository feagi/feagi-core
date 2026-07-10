/// Defines the quantization used in an axon bundle for synapse dynamics. As synapses do not have
/// any required fields, there are not required fields.
/// DO NOT IMPLEMENT THIS IN ACTUAL DATA STRUCTURES! THIS IS ONLY INTENDED TO CARRY QUANTIZATION
/// CONTEXTS
pub trait SynapseModelQuantization {}

/// Root trait for all synapse axon bundle implementations, which is essentially any data related
/// to the synapses of a single mapping. This should be extended only with axon bundle level data
pub trait SynapseModelAxonBundleData<SMQ>
where
    SMQ: SynapseModelQuantization,
{
    // implement any axon bundle level data
}

/// Root trait for all synapse data implementations, which is essentially any data related
/// to each individual synapse of a mapping
pub trait SynapseModelSynapseData<SMQ>
where
    SMQ: SynapseModelQuantization,
{
    // implement any per synapse data
}

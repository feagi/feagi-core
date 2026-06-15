use feagi_structures::feagi_data::quantization_levels::extendable_quantizations::SynapseModelQuantization;

/// Root trait for all synapse axon bundle implementations, which is essentially any data related
/// to the synapses of a single mapping. This should be extended only with axon bundle level data
pub trait SynapseModelAxonBundleData<SMQ>
where
    SMQ: SynapseModelQuantization,
{
    
    // implement any axon bundle level data
}


//region CPU Specific Trait

/// Root CPU trait for SynapseModelAxonBundleData
pub trait SynapseModelAxonBundleDataCPU<SMQ>:
SynapseModelAxonBundleData<SMQ>
where
    SMQ: SynapseModelQuantization,
{

    // implement any axon bundle level data
}

//endregion
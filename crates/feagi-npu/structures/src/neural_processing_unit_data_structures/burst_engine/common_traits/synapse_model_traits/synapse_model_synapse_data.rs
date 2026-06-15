use feagi_structures::feagi_data::quantization_levels::extendable_quantizations::SynapseModelQuantization;

/// Root trait for all synapse data implementations, which is essentially any data related
/// to each individual synapse of a mapping
pub trait SynapseModelSynapseData<SMQ>
where
    SMQ: SynapseModelQuantization,
{

    // implement any axon bundle level data
}


//region CPU Specific Trait

/// Root CPU trait for SynapseModelSynapseData
pub trait SynapseModelSynapseDataCPU<SMQ>:
SynapseModelSynapseData<SMQ>
where
    SMQ: SynapseModelQuantization,
{

    // implement any axon bundle level data
}

//endregion
use feagi_data::quantization_levels::extendable_quantizations::NeuronModelQuantization;

/// Root trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model. This should be extended with only the per neuron data
pub trait NeuronModelNeuronData<NMQ>:
where
    NMQ: NeuronModelQuantization,
{
    // As per CorticalAreasIndexQuantization, this takes in GlobalBurstIndexQuant,
    // and NeuronIndexCountQuant. These are not settable by the model and instead picked by
    // FEAGI's NPU

    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    // Implement any per-neuron level data members

    // No methods!
}


//region CPU Specific Trait


/// Root CPU trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model
pub trait NeuronModelNeuronDataCPU<NMQ>:
NeuronModelNeuronData<NMQ>
where
    NMQ: NeuronModelQuantization,
{
    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    // Implement any per-neuron level data members (or make members pub)
}


//endregion
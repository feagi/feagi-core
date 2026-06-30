use feagi_data::quantization_levels::extendable_quantizations::NeuronModelQuantization;

/// Root trait for all cortical data implementations, essentially any cortical level data shared
/// by all neurons in a cortical area of a given neuron model. This should be extended with only
/// the cortical level data
pub trait NeuronModelCorticalData<NMQ>
where
    NMQ: NeuronModelQuantization,
{
    /// Set to true if the neuron model needs to be informed if the global burst index counter is
    /// about to overflow. Otherwise, set to false
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool;
    
    /// Set to true if the neuron model can run under Dimensional type cortical areas
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_DIMENSIONAL: bool;
    
    // TODO other cortical configuration types

    // Implement any cortical level data

    // No methods!
    
}

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

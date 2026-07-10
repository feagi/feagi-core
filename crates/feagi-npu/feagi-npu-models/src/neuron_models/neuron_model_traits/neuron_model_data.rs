use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;

/// Root trait for all cortical_area data implementations, essentially any cortical_area level data shared
/// by all neurons in a cortical_area area of a given neuron model. This should be extended with only
/// the cortical_area level data
pub trait NeuronModelCorticalData<CPQ>
where
    CPQ: CorticalPotentialQuantization,
{
    /// Set to true if the neuron model needs to be informed if the global burst index counter is
    /// about to overflow. Otherwise, set to false
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool;

    /// Set to true if the neuron model can run under Dimensional type cortical_area areas
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_DIMENSIONAL: bool;

    // TODO other cortical_area configuration types

    // Implement any cortical_area level data

    // No methods!
}

/// Root trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model. This should be extended with only the per neuron data
pub trait NeuronModelNeuronData<CPQ>
where
    CPQ: CorticalPotentialQuantization,
{
    // As per CorticalAreasIndexQuantization, this takes in GlobalBurstIndexQuant,
    // and NeuronIndexCountQuant. These are not settable by the model and instead picked by
    // FEAGI's NPU

    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    // Implement any per-neuron level data members

    // No methods!
}

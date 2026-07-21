use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::neuron::common_structs::model_and_quantization::NestedNeuronModelTypeAndQuantization;

/// Root trait for all cortical_area data implementations, essentially any cortical_area level data shared
/// by all neurons in a cortical_area area of a given neuron model. This should be extended with only
/// the cortical_area level data
pub trait NeuronModelCorticalData<CPQ>: Clone
where
    CPQ: CorticalPotentialQuantization,
{
    const LEVEL: NestedNeuronModelTypeAndQuantization;
}




/// Root trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model. This should be extended with only the per neuron data
pub trait NeuronModelNeuronData<CPQ>: Clone
where
    CPQ: CorticalPotentialQuantization,
{
    const LEVEL: NestedNeuronModelTypeAndQuantization;
    
    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be! History is also passed in separately if enabled!
}


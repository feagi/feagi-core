use crate::cortical_area::neuron::neuron_model::generated_enums::{
    NeuronModelType, NeuronModelTypeAndQuantizationNested, NeuronModelTypeAndQuantizationPacked,
};
use crate::cortical_area::neuron::neuron_model::quantization::NeuronModelQuantization;

/// Root trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model. This should be extended with only the per neuron data. Note that the "default" trait is used for memory purposes and any
/// values specified in default will not actually be used.
pub trait NeuronModelNeuronData<NMQ>: Clone + Default + Copy
where
    NMQ: NeuronModelQuantization,
{
    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be! History is also passed in separately if enabled!

    /// If the neuron model has per neuron model. This will always be the case except if your model
    /// uses `EmptyPerNeuronData`
    const NEURON_MODEL_USES_PER_NEURON_DATA: bool = true;

    /// A flat enum denoting what type of neuron model this is
    const NEURON_MODEL: NeuronModelType = NMQ::NEURON_MODEL;
    /// A flat enum value denoting the quantization level of this neuron model instance
    const NEURON_QUANTIZATION: NMQ::QuantLevelType = NMQ::NEURON_QUANTIZATION;
    /// A nested enum that denotes both the neuron model and the quantization at runtime.
    const NESTED_NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationNested = NMQ::NESTED_NEURON_MODEL_AND_QUANTIZATION;
    /// A flat enum (byte) that denotes both the neuron model and the quantization at runtime. Mainly
    /// useful for NPU
    const PACKED_NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationPacked = NMQ::PACKED_NEURON_MODEL_AND_QUANTIZATION;
}

/// A neuron "implementation" to use if your neuron model does not need to store per neuron data
/// outside of what FEAGI automatically can allocate (membrane potential, firing history)
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyNeuronData;

impl<NMQ: NeuronModelQuantization> NeuronModelNeuronData<NMQ> for EmptyNeuronData {
    // This struct explicitly is meant to denote not using this
    const NEURON_MODEL_USES_PER_NEURON_DATA: bool = false;
}

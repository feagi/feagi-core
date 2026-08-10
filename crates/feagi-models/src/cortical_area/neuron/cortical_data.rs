use crate::cortical_area::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::cortical_area::neuron_model_implementations::generated_enums::{
    NeuronModelType, NeuronModelTypeAndQuantizationNested, NeuronModelTypeAndQuantizationPacked,
};

/// Root trait for all cortical_area data implementations, essentially any cortical_area level data shared
/// by all neurons in a cortical_area area of a given neuron model. This should be extended with only
/// the cortical_area level data. Note that the "default" trait is used for memory purposes and any
/// values specified in default will not actually be used.
pub trait NeuronModelCorticalData<NMQ>: Clone + Default + Copy
where
    NMQ: NeuronModelQuantization,
{
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

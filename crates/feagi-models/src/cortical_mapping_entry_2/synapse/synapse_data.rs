use crate::cortical_mapping_entry::synapse::synapse_model_quantization::SynapseModelQuantization;
use crate::cortical_mapping_entry::synapse_model_implementations::generated_enums::{
    SynapseModelType, SynapseModelTypeAndQuantizationNested, SynapseModelTypeAndQuantizationPacked,
};

/// Root trait for all synapse data implementations, which is essentially any data related
/// to each individual synapse of a mapping. Note that the "default" trait is used for memory
/// purposes and any values specified in default will not actually be used.
pub trait SynapseModelSynapseData<SMQ>: Clone + Copy
where
    SMQ: SynapseModelQuantization,
{
    const SYNAPSE_MODEL_USES_PER_SYNAPSE_DATA: bool = true;

    /// A flat enum value denoting what type of synapse model this synapse model instance is
    const SYNAPSE_MODEL: SynapseModelType = SMQ::SYNAPSE_MODEL;
    /// A flat enum value denoting the quantization level of this synapse model instance
    const SYNAPSE_QUANTIZATION: SMQ::QuantLevelType = SMQ::SYNAPSE_QUANTIZATION;
    /// A nested enum that denotes both the synapse model and the quantization at runtime.
    const NESTED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationNested = SMQ::NESTED_SYNAPSE_MODEL_AND_QUANTIZATION;
    /// A flat enum (byte) that denotes both the synapse model and the quantization at runtime.
    /// Useful for some burst engines
    const PACKED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationPacked = SMQ::PACKED_SYNAPSE_MODEL_AND_QUANTIZATION;
}

/// A synapse "implementation" to use if your synapse model does not need to store per synapse data
/// outside of what FEAGI automatically can allocate
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyPerSynapseData;

impl<SMQ: SynapseModelQuantization> SynapseModelSynapseData<SMQ> for EmptyPerSynapseData {
    // This struct explicitly is meant to denote not using this
    const SYNAPSE_MODEL_USES_PER_SYNAPSE_DATA: bool = false;
}

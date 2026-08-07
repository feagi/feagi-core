use crate::cortical_mapping_entry::synapse_model_implementations::generated_enums::{SynapseModelType, SynapseModelTypeAndQuantizationNested, SynapseModelTypeAndQuantizationPacked};
use crate::cortical_mapping_entry::synapse::synapse_model_quantization::SynapseModelQuantization;

/// Root trait for all cortical mapping entry implementations, which is essentially any data shared 
/// among all synapses within a single mapping. Note that the "default" trait is used for memory 
/// purposes and any values specified in default will not actually be used.
pub trait SynapseModelCorticalMappingEntryData<SMQ>: Clone + Default + Copy
where
    SMQ: SynapseModelQuantization,
{
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
use crate::synapse::synapse_model_data::{SynapseCorticalMappingEntryData, SynapseModelSynapseData};
use crate::synapse::synapse_model_quantization::SynapseModelQuantization;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::synapse::model_capabilities::source_fire_history::NeuronFireHistory;
use crate::synapse::model_generated::model_type_and_quantization::{SynapseModelTypeAndQuantizationNested, SynapseModelTypeAndQuantizationPacked, SynapseModelType};

/// Root base trait for defining synapse firing and alteration of
/// transmitting synapse potentials between synapses. Does NOT store actual data,
pub trait SynapseModel<FIQ, SMQ>
where
    FIQ: FeagiIndexQuantization,
    SMQ: SynapseModelQuantization,
{
    /// The type of cortical mapping entry data this model uses
    type CorticalMappingEntryData: SynapseCorticalMappingEntryData<SMQ>;
    /// The per synapse data needed by this synapse model. To have none, use `EmptyPerSynapseData`
    type SynapseData: SynapseModelSynapseData<SMQ>;

    /// The type of fire history source neurons need to have
    type SourceFireHistory: NeuronFireHistory<FIQ>;

    // Proxied properties, here to make using this easier
    /// A flat enum value denoting what type of synapse model this synapse model instance is
    const SYNAPSE_MODEL: SynapseModelType = SMQ::SYNAPSE_MODEL;
    /// A flat enum value denoting the quantization level of this synapse model instance
    const SYNAPSE_QUANTIZATION: SMQ::QuantLevelType = SMQ::SYNAPSE_QUANTIZATION;
    /// A nested enum that denotes both the synapse model and the quantization at runtime.
    const NESTED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationNested = SMQ::NESTED_SYNAPSE_MODEL_AND_QUANTIZATION;
    /// A flat enum (byte) that denotes both the synapse model and the quantization at runtime.
    /// Useful for some burst engines
    const PACKED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationPacked = SMQ::PACKED_SYNAPSE_MODEL_AND_QUANTIZATION;
    /// If the synapse model has per synapse model. This will always be the case except if your model
    /// uses `EmptyPerSynapseData`
    const SYNAPSE_MODEL_USES_PER_SYNAPSE_DATA: bool = Self::SynapseData::SYNAPSE_MODEL_USES_PER_SYNAPSE_DATA;

    /// A single synapse processes incoming data and updates it
    fn synapse_process_incoming_signal(
        incoming_potential: &NeuronMembranePotential<SMQ::JunctionPotentialQuant>,
        mapping_entry_data: &Self::CorticalMappingEntryData,
        source_fire_history: &Self::SourceFireHistory,
    ) -> NeuronMembranePotential<SMQ::JunctionPotentialQuant>;

    //fn get_psp_uniformity_weight(axon_bundle_data: &Self::CorticalMappingEntryData) -> CPQIn::MembranePotentialQuant; // TODO discuss this
}

use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::synapse::interfacing::model_and_quantization::SynapseModelTypeAndQuantization;

/// Add / Remove / Edit Cortical Mappings
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPURequestParametersCorticalMapping {
    AppendMappingEntry{
        source: CorticalID,
        destination: CorticalID,
        new_mapping_synapse_type: SynapseModelTypeAndQuantization,
        neuron_iterator: Box<impl Iterator<(usize, usize)>>
    }
}


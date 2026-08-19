use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::neuron_model::genome_compose::cortical_writer_by_model_quant::CorticalWriterByModelQuant;
use feagi_models::neuron_model::neuron_model_implementations::feagi_advanced::composers::FeagiAdvancedModelCorticalWriter;
use feagi_models::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer_by_model_quant::CorticalMappingEntryWriterByModelQuant;
use feagi_models::wrapped_index_collections::CorticalEngineIndex;

/// Instructs a burst engine what type of data processing to do
pub enum KernelCommand {
    /// Full burst of neuron dynamics then synapse dynamics
    FullBurst,
    // TODO later add semi burst types for cross engine collab
}

pub enum EngineConnectomeEditRequest<FIQ: FeagiIndexQuantization> {
    AddCorticalArea {
        writer: CorticalWriterByModelQuant
    },
    AddMapping {
        source: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
        destination: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
        writer: CorticalMappingEntryWriterByModelQuant
    }
}

/// Return changes made following a successful edit, such as the indexes of created structures
pub enum EngineConnectomeEditResponse<FIQ: FeagiIndexQuantization> {
    AddedCorticalArea {
        new_area_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>
    },
    AddedMapping {
        // TODO what should we put in here?
    }
}
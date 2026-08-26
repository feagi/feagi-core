use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_genomic_context::cortical_area::CorticalID;
use crate::standard::npu::burst_engine::wrapped_indexes::EngineCorticalIndex;

/// Instructs a burst engine what type of data processing to do
pub enum KernelCommand {
    /// Full burst of neuron dynamics then synapse dynamics
    FullBurst,
    // TODO later add semi burst types for cross engine collab
}

pub enum EngineConnectomeEditRequest<FIQ: FeagiIndexQuantization> {
    AddCorticalArea {
        writer: (),//CorticalWriterByModelQuant
    },
    AddMapping {
        source: EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>,
        destination: EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>,
        writer: ()//CorticalMappingEntryWriterByModelQuant
    }
}

/// Return changes made following a successful edit, such as the indexes of created structures
pub enum EngineConnectomeEditResponse<FIQ: FeagiIndexQuantization> {
    AddedCorticalArea {
        new_area_index: EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>
    },
    AddedMapping {
        // TODO what should we put in here?
    }
}
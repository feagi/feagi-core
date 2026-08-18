use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_index_collections::CorticalEngineIndex;

/// Instructs a burst engine what type of data processing to do
pub enum KernelCommand {
    /// Full burst of neuron dynamics then synapse dynamics
    FullBurst,
    // TODO later add semi burst types for cross engine collab
}

pub enum EngineConnectomeEditRequest<FIQ: FeagiIndexQuantization> {
    AddCorticalArea {
        // TODO contexts
    }
}

/// Return changes made following a successful edit, such as the indexes of created structures
pub enum EngineConnectomeEditResponse<FIQ: FeagiIndexQuantization> {
    AddedCorticalArea {
        new_area_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>
    }
}
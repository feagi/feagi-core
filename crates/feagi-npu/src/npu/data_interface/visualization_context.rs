use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_index_collections::CorticalEngineIndex;

/// Gives context on how to decode visualization data from the byte array
pub struct VoxelVisualizationContext<FIQ: FeagiIndexQuantization> {
    pub cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
}
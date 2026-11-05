use feagi_data::feagi_data_neuron::quantization_levels::FeagiIndexQuantization;
use feagi_models::wrapped_index_collections::CorticalEngineIndex;

/// Gives context on how to decode visualization data from the byte array
pub struct VoxelVisualizationContext<FIQ: FeagiIndexQuantization> {
    pub cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
}
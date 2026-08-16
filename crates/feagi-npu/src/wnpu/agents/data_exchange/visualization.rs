use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensions;
use feagi_data::values::quantizable::QuantizedUnsignedIntegerTrait;
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::wrapped_index_collections::NeuronEngineByteIndex;

// TODO iteration 
// TODO this whole thing is not efficient
/// For a given frame, has all voxel visualiations
#[derive(Clone)]
pub struct VoxelVisualization<NeuronIndex: QuantizedUnsignedIntegerTrait> {
    pub areas: Vec<VisualizationHeader<NeuronIndex>>,
    pub bytes: Vec<u8>,
}


// TODO this is not a well padded header
/// Describes what cortical area is found where in voxel visualization data
#[derive(Clone)]
pub struct VisualizationHeader<NeuronIndex: QuantizedUnsignedIntegerTrait> {
    pub cortical_id: CorticalID,
    pub voxel_dims: NeuronVoxelDimensions<NeuronIndex>,
    pub byte_index_start: NeuronEngineByteIndex<NeuronIndex>,
    pub byte_index_length: NeuronEngineByteIndex<NeuronIndex>, // includes padding!
}




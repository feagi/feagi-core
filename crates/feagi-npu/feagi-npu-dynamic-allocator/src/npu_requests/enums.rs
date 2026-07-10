
// TODO cortical_area area deletions, mapping deletions!

use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::bit_32::NeuronVoxelDimensions;
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::neuron_models::NeuronModelTypeAndQuantization;

pub(crate) enum NPURequestBase {

    CorticalArea(NPURequestCorticalArea)
}


pub(crate) enum NPURequestBurstEngine {
    InitBurstEngine{}, // TODO pass in devices to mount? Memory / thread restrictions?
}




pub(crate) enum NPURequestCorticalArea {
    CreateCustomArea {
        dimensions: NeuronVoxelDimensions,
        voxel_density: u32,
        cortical_id: CorticalID,
        neuron_model_type_and_quantization: NeuronModelTypeAndQuantization
    }
}


pub(crate) enum NPURequestMapping {

}

// TODO cortical area deletions, mapping deletions!

use feagi_npu_neuron_models::NeuronModelQuantizationAndDeviceClass;
use feagi_structures::neuron_voxels::bit_32::NeuronVoxelDimensions;



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
        neuron_model_quantization_and_device_class: NeuronModelQuantizationAndDeviceClass
    }
}


pub(crate) enum NPURequestMapping {

}
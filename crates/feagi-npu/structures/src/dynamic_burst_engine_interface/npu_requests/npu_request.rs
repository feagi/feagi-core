use feagi_npu_neuron_models::NeuronModelQuantizationAndDeviceClass;
use feagi_structures::feagi_data::create_quantized_index_count_wrapper;
use feagi_structures::neuron_voxels::bit_32::NeuronVoxelDimensions;
use crate::dynamic_burst_engine_interface::npu_requests::enums::{NPURequestBase, NPURequestCorticalAreaCreate};

// TODO switch to Feagi Error instead of option!

create_quantized_index_count_wrapper!(NPURequestID, u32);

/// Defines a request for the NPU to modify something
pub struct NPURequest {
    pub requestee: String,
    request: NPURequestBase
}

impl NPURequest {

    const STRING_REQUESTEE_MAX_LENGTH: usize = 64;

    pub fn cortical_area_create_custom(
        dimensions: NeuronVoxelDimensions,
        voxel_density: u32,
        neuron_model_quantization_and_device_class: NeuronModelQuantizationAndDeviceClass
    ) -> Option<NPURequest>

    {
        if voxel_density == 0 { return None; }

        // TODO dimensions should have a validity check!

        Some(Self {
            requestee: String::new(),
            request: NPURequestBase::Create(NPURequestCorticalAreaCreate { dimensions, voxel_density, neuron_model_quantization_and_device_class })
        } )
    }
    
    
    
    pub(crate) fn get_request(&self) -> &NPURequestBase {
        &self.request
    }
    
}


// TODO delete cortical area, add mappings!




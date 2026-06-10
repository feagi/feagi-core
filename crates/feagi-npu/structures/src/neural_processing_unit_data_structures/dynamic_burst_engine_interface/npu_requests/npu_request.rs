use feagi_structures::feagi_data::create_quantized_index_count_wrapper;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::bit_32::NeuronVoxelDimensions;
use crate::neural_processing_unit_data_structures::dynamic_burst_engine_interface::npu_requests::enums::{NPURequestBase, NPURequestCorticalArea};
use crate::neuron_models::neuron_models::NeuronModelTypeAndQuantization;
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
        cortical_id: CorticalID,
        neuron_model_type_and_quantization: NeuronModelTypeAndQuantization
    ) -> Option<NPURequest>

    {
        if voxel_density == 0 { return None; }

        // TODO dimensions should have a validity check!

        Some(Self {
            requestee: String::new(),
            request: NPURequestBase::CorticalArea(NPURequestCorticalArea::CreateCustomArea{dimensions, voxel_density, cortical_id, neuron_model_type_and_quantization })
        } )
    }



    pub(crate) fn get_request(&self) -> &NPURequestBase {
        &self.request
    }

}


// TODO delete cortical area, add mappings!




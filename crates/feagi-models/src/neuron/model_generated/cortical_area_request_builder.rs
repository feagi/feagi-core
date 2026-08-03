use crate::connectome_requests::connectome_request::{ConnectomeRequest, ConnectomeRequestType};
use crate::neuron::models::feagi_advanced::FeagiAdvancedModelQuantizationLevel;
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensions;
use feagi_data::neurons::NeuronVoxelDensityIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationLevel;
// TODO these should be generated

pub struct CorticalAreaRequestBuilder {
    index_level: FeagiIndexQuantizationLevel,
}

impl CorticalAreaRequestBuilder {
    #[doc(hidden)]
    pub(crate) fn new(index_level: FeagiIndexQuantizationLevel) -> Self {
        CorticalAreaRequestBuilder { index_level }
    }

    /// Request is related to a cortical area of the "feagi advanced" neuron model
    pub fn feagi_advanced(self) -> FeagiAdvancedCreatorRequestBuilder {
        FeagiAdvancedCreatorRequestBuilder {
            index_level: self.index_level,
        }
    }
}

pub struct FeagiAdvancedCreatorRequestBuilder {
    index_level: FeagiIndexQuantizationLevel,
}

impl FeagiAdvancedCreatorRequestBuilder {
    pub fn create_dimensional(
        quantization: FeagiAdvancedModelQuantizationLevel,
        dimensions: NeuronVoxelDimensions<u64>,
        neurons_per_voxel: NeuronVoxelDensityIndex<u64>,
    ) -> Result<ConnectomeRequest, ()> {

        ConnectomeRequest {
            index_level: FeagiIndexQuantizationLevel::Absurd,
            request_type: ConnectomeRequestType::CorticalAreaAddDimensional(
                dimensions
            ),
        }
        
        // TODO quant checks
    }

    pub fn create_formless(
        quantization: FeagiAdvancedModelQuantizationLevel,
        number_neurons: u64,
    )  -> Result<ConnectomeRequest, ()> {
        // TODO quant checks
    }

    pub fn delete(
        cortical_id: u64 // CorticalID can be a u64
    )
}

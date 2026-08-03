use crate::neuron::model_generated::cortical_area_request_builder::CorticalAreaRequestBuilder;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationLevel;
use crate::synapse::model_generated::cortical_mapping_request_builder::CorticalMappingRequestBuilder;

/// Endpoint emitted from an NPU that lets you construct requests to edit the connectome of the
/// NPU with strong compile time typed checking. Rust FTW. Can only be constructed by the NPU
/// so that the context is correct
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectomeRequestBuilder {
    index_level: FeagiIndexQuantizationLevel,
}

impl ConnectomeRequestBuilder {
    /// Created only by NPU to give users an endpoint to create NPU Requests
    #[doc(hidden)]
    pub fn new(index_level: FeagiIndexQuantizationLevel) -> Self {
        Self { index_level }
    }

    /// Make a request related to cortical areas
    pub fn cortical_area(self) -> CorticalAreaRequestBuilder {
        CorticalAreaRequestBuilder::new(self.index_level)
    }

    pub fn cortical_mapping(self) -> CorticalMappingRequestBuilder {
        CorticalMappingRequestBuilder::new(self.index_level)
    }
}

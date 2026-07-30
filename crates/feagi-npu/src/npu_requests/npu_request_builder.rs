use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantizationLevel, FeagiIndexQuantization};

pub struct NPURequestBuilder {
    npu_index_quantization: FeagiIndexQuantizationLevel,
    unsorted_requests: Vec<NPURequests>,
}


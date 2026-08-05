use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationLevel;

pub struct CorticalMappingRequestBuilder {
    index_level: FeagiIndexQuantizationLevel
}

impl CorticalMappingRequestBuilder {
    
    pub(crate) fn new(index_level: FeagiIndexQuantizationLevel) -> Self {
        CorticalMappingRequestBuilder { index_level }
    }
    
    pub fn uniform(self) -> UniformCreatorRequestBuilder {
        UniformCreatorRequestBuilder {
            index_level: self.index_level,
        }
    }
}

pub struct UniformCreatorRequestBuilder {
    index_level: FeagiIndexQuantizationLevel
}

impl UniformCreatorRequestBuilder {
    
}
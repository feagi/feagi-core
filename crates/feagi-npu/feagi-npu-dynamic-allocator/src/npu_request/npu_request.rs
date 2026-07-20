use crate::npu_request::parameters::burst_engine::NPURequestParametersBurstEngine;
use crate::npu_request::parameters::cortical_area::NPURequestParametersCorticalArea;
use crate::npu_request::parameters::cortical_mapping::NPURequestParametersCorticalMapping;

/// easily allows any o these sub keys to become an NPU Request directly
pub trait IntoNPURequest {
    fn into_npu_request(self) -> NPURequest;
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NPURequest
{
    BurstEngine(NPURequestParametersBurstEngine),
    CorticalArea(NPURequestParametersCorticalArea),
    Mapping(NPURequestParametersCorticalMapping),
    GenomeDebug(),
    // TODO agent registration?
}

impl From<NPURequestParametersBurstEngine> for NPURequest {
    fn from(value: NPURequestParametersBurstEngine) -> Self {
        NPURequest::BurstEngine(value)
    }
}

impl From<NPURequestParametersCorticalArea> for NPURequest {
    fn from(value: NPURequestParametersCorticalArea) -> Self {
        NPURequest::CorticalArea(value)
    }
}

impl From<NPURequestParametersCorticalMapping> for NPURequest {
    fn from(value: NPURequestParametersCorticalMapping) -> Self {
        NPURequest::Mapping(value)
    }
}
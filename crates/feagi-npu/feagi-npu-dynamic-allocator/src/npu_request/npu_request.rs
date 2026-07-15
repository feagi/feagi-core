use crate::npu_request::parameters::burst_engine::NPURequestParametersBurstEngine;
use crate::npu_request::parameters::cortical_area::NPURequestParametersCorticalArea;
use crate::npu_request::parameters::cortical_mapping::NPURequestParametersCorticalMapping;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NPURequest
{
    BurstEngine(NPURequestParametersBurstEngine),
    CorticalArea(NPURequestParametersCorticalArea),
    Mapping(NPURequestParametersCorticalMapping),
    // TODO agent registration?
}


// These static functions are kept separate as adding them to the trait makes them no longer dyn compatible

use feagi_basis::FeagiBasisError;
use crate::legacy_code::sensio_motor::data_pipeline::pipeline_stage::PipelineStage;
use crate::legacy_code::sensio_motor::data_pipeline::PipelineStageProperties;

pub(crate) fn stage_properties_to_stages(
    pipeline_stage_properties: &[PipelineStageProperties],
) -> Result<Vec<Box<dyn PipelineStage>>, FeagiBasisError> {
    let mut output: Vec<Box<dyn PipelineStage>> =
        Vec::with_capacity(pipeline_stage_properties.len());
    for properties in pipeline_stage_properties.iter() {
        output.push(properties.create_stage());
    }
    Ok(output)
}

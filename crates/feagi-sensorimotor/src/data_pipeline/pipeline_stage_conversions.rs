use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;
use feagi_data_structures::FeagiDataError;

// These static functions are kept separate as adding them to the trait makes them no longer dyn compatible

pub(crate) fn stage_properties_to_stages(
    pipeline_stage_properties: &[PipelineStageProperties],
) -> Result<Vec<Box<dyn PipelineStage>>, FeagiDataError> {
    let mut output: Vec<Box<dyn PipelineStage>> =
        Vec::with_capacity(pipeline_stage_properties.len());
    for properties in pipeline_stage_properties.iter() {
        output.push(properties.create_stage());
    }
    Ok(output)
}

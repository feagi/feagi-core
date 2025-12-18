
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;
use feagi_data_structures::FeagiDataError;

// These static functions are kept separate as adding them to the trait makes them no longer dyn compatible

pub(crate) fn stage_properties_to_stages(pipeline_stage_properties: &Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<Vec<Box<dyn PipelineStage>>, FeagiDataError> {
    let mut output: Vec<Box<dyn PipelineStage>> = Vec::with_capacity(pipeline_stage_properties.len());
    for pipeline_stage_properties in pipeline_stage_properties.iter() {
        output.push(pipeline_stage_properties.create_stage());
    }
    Ok(output)
}

pub(crate) fn stages_to_stage_properties(pipeline_stages: &Vec<Box<dyn PipelineStage>>) -> Result<Vec<Box<dyn PipelineStageProperties + Sync + Send>>, FeagiDataError> {
    let mut output: Vec<Box<dyn PipelineStageProperties + Sync + Send>> = Vec::with_capacity(pipeline_stages.len());
    for pipeline_stage in pipeline_stages.iter() {
        output.push(pipeline_stage.create_properties())
    }
    Ok(output)

}
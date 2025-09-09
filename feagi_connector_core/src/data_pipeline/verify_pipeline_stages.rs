use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::WrappedIOType;
use crate::data_pipeline::PipelineStageIndex;
use crate::data_pipeline::pipeline_stage::PipelineStage;

pub(crate) fn verify_pipeline_stages(pipeline_stages: &Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
    let number_of_stages = pipeline_stages.len();

    if number_of_stages == 0 {
        return Err(FeagiDataError::BadParameters("Pipeline Stage Runner cannot have 0 Stages!".into()).into())
    }

    // Ensure data can pass between processing
    for stage_index in 0..number_of_stages - 1  {
        let first = &pipeline_stages[stage_index];
        let second = &pipeline_stages[stage_index + 1];
        if first.get_output_data_type() != second.get_input_data_type() {
            return Err(FeagiDataError::BadParameters(format!("Given stage runner at index {} has output type {}, which does not match the input type of stage runner at index {} or type {}!",
                                                              stage_index, first.get_output_data_type(), stage_index + 1,  second.get_input_data_type()).into()).into());
        }
    };

    Ok(())
}

pub(crate) fn verify_replacing_stage(current_stages: &Vec<Box<dyn PipelineStage + Sync + Send>>,
                                     new_stage: &Box<dyn PipelineStage + Sync + Send>,
                                     input_type: &WrappedIOType, output_type: &WrappedIOType,
                                     new_stage_index: PipelineStageIndex) -> Result<(), FeagiDataError> {

    if *new_stage_index >= current_stages.len() as u32 {
        return Err(FeagiDataError::BadParameters(format!("New stage index {} is out of range! Max allowed is {}!", *new_stage_index, current_stages.len()- 1)).into());
    }

    let comparing_input = if *new_stage_index == 0 {
        input_type
    } else {
        &current_stages[*new_stage_index as usize - 1].get_output_data_type()
    };

    let comparing_output = if *new_stage_index == (current_stages.len()- 1) as u32 {
        output_type
    } else {
        &current_stages[*new_stage_index as usize + 1].get_output_data_type()
    };

    if comparing_input != &new_stage.get_input_data_type() {
        return Err(FeagiDataError::BadParameters(format!("Precursor to stage at index {} outputs data type {} but given stage accepts {}!", *new_stage_index, comparing_input, new_stage.get_input_data_type()).into()).into());
    }
    if comparing_output != &new_stage.get_output_data_type() {
        return Err(FeagiDataError::BadParameters(format!("Precursor to stage at index {} outputs data type {} but given stage accepts {}!", *new_stage_index, comparing_output, new_stage.get_output_data_type()).into()).into());
    }
    Ok(())

}


use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::{stage_properties_to_stages, PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub(crate) struct PipelineStageRunner {
    input_type: WrappedIOType,
    output_type: WrappedIOType,
    last_instant_data_processed: Instant,
    pipeline_stages: Vec<Box<dyn PipelineStage>>,
    cached_input: WrappedIOData
}

impl PipelineStageRunner {
    pub fn new(pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>, cached_input_value: WrappedIOData, expected_output_type: WrappedIOType) -> Result<Self, FeagiDataError> {
        let expected_input_type: WrappedIOType = (&cached_input_value).into();
        verify_pipeline_stage_properties(&pipeline_stage_properties, expected_input_type, expected_output_type)?;
        let pipeline_stages = stage_properties_to_stages(&pipeline_stage_properties)?;
        
        Ok(PipelineStageRunner {
            input_type: expected_input_type,
            last_instant_data_processed: Instant::now(),
            output_type: expected_output_type,
            pipeline_stages,
            cached_input: cached_input_value
        })
    }

    //region Data

    /// Returns the input data type expected by this processor chain.
    ///
    /// This is determined by the input type of the first processor in the chain.
    /// Used for validation before processing new input data.
    pub fn get_input_data_type(&self) -> WrappedIOType {
        self.input_type
    }

    /// Returns the output data type produced by this processor chain.
    ///
    /// This is determined by the output type of the last processor in the chain.
    /// Useful for understanding what type of data the pipeline will produce.
    pub fn get_output_data_type(&self) -> WrappedIOType {
        self.output_type
    }

    pub fn try_update_value(&mut self, new_value: WrappedIOData, time_of_update: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        if WrappedIOType::from(&new_value) != self.input_type {
            return Err(FeagiDataError::BadParameters(format!("Expected Input data type of {} but received {}!", self.input_type.to_string(), &new_value.to_string())).into());
        }

        self.cached_input = new_value;

        if self.pipeline_stages.is_empty() {
            return Ok(&self.cached_input);
        }

        //TODO There has to be a better way to do this, but I keep running into limitations with mutating self.cache_processors

        // Process the first processor with the input value
        self.pipeline_stages[0].process_new_input(&self.cached_input, time_of_update)?;

        // Process subsequent processing using split_at_mut to avoid borrowing conflicts
        for i in 1..self.pipeline_stages.len() {
            let (left, right) = self.pipeline_stages.split_at_mut(i);
            let previous_output = left[i - 1].get_most_recent_output();
            right[0].process_new_input(previous_output, time_of_update)?;
        }

        self.last_instant_data_processed = time_of_update;
        Ok(self.pipeline_stages.last().unwrap().get_most_recent_output()) // Return the output from the last processor
    }

    /// Returns the most recent output from the final processor in the chain.
    ///
    /// This provides access to the current state of the processing pipeline without
    /// triggering new processing. Useful for reading the current processed value.
    ///
    /// # Returns
    /// Reference to the output data from the last processor in the chain.
    pub fn get_most_recent_output(&self) -> &WrappedIOData {
        self.pipeline_stages.last().unwrap().get_most_recent_output()
    }

    pub fn get_last_processed_instant(&self) -> Instant {
        self.last_instant_data_processed
    }

    //endregion

    //region Pipeline Stages

    pub fn try_get_single_stage_properties(&self, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        self.verify_pipeline_stage_index(stage_index)?;
        Ok(self.pipeline_stages[*stage_index as usize].create_properties())
    }

    pub fn get_all_stage_properties(&self) -> Vec<Box<dyn PipelineStageProperties + Sync + Send>>  {
        let mut output: Vec<Box<dyn PipelineStageProperties + Sync + Send>> = Vec::with_capacity(self.pipeline_stages.len());
        for stage in &self.pipeline_stages {
            output.push(stage.create_properties())
        }
        output
    }

    pub fn try_update_single_stage_properties(&mut self, updating_stage_index: PipelineStagePropertyIndex, updated_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        self.verify_pipeline_stage_index(updating_stage_index)?;
        self.pipeline_stages[*updating_stage_index as usize].load_properties(updated_properties)?;
        Ok(())
    }

    pub fn try_update_all_stage_properties(&mut self, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        if new_pipeline_stage_properties.len() != self.pipeline_stages.len() {
            return Err(FeagiDataError::BadParameters(format!("Unable to update {} contained stages with {} properties!", self.pipeline_stages.len(), new_pipeline_stage_properties.len())).into());
        }
        self.pipeline_stages.iter_mut()
            .zip(new_pipeline_stage_properties)
            .try_for_each(|(current_stage, new_properties)| {
                current_stage.load_properties(new_properties)
            })?;
        Ok(())
    }

    pub fn try_replace_single_stage(&mut self, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        self.verify_pipeline_stage_index(replacing_at_index)?;
        verify_replacing_stage_properties(&self.pipeline_stages, &new_pipeline_stage_properties, &self.input_type, &self.output_type, replacing_at_index)?;
        self.pipeline_stages[*replacing_at_index as usize] = new_pipeline_stage_properties.create_stage();
        Ok(())
    }

    pub fn try_replace_all_stages(&mut self, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        verify_pipeline_stage_properties(&new_pipeline_stage_properties, self.input_type, self.output_type)?;
        self.pipeline_stages = stage_properties_to_stages(&new_pipeline_stage_properties)?;
        Ok(())
    }


    /*
    // TODO we may not need these

    pub fn clone_stages(&self) -> Vec<Box<dyn PipelineStage + Sync + Send>> {
        let mut output: Vec<Box<dyn PipelineStage + Sync + Send>> = Vec::with_capacity(self.pipeline_stages.len());
        for pipeline_stage in self.pipeline_stages.iter() {
            output.push(pipeline_stage.clone_box())
        }

        output
    }

    pub fn clone_stage(&self, index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {
        if *index >= self.pipeline_stages.len() as u32 {
            return Err(FeagiDataError::BadParameters(format!("Pipeline Index {} out of bounds!", *index)).into());
        }
        Ok(self.pipeline_stages[*index as usize].clone_box())
    }

     */

    //endregion

    //region Internal

    fn verify_pipeline_stage_index(&self, stage_index: PipelineStagePropertyIndex) -> Result<(), FeagiDataError> {
        if self.pipeline_stages.is_empty() {
            return Err(FeagiDataError::BadParameters("No Stages exist to be overwritten!".into()).into());
        }

        if *stage_index >= self.pipeline_stages.len() as u32 {
            return Err(FeagiDataError::BadParameters(format!("New stage index {} is out of range! Max allowed is {}!", *stage_index, self.pipeline_stages.len()- 1)).into());
        }
        Ok(())
    }

    //endregion

}


fn verify_pipeline_stage_properties(pipeline_stage_properties: &Vec<Box<dyn PipelineStageProperties + Sync + Send>>, expected_input: WrappedIOType, expected_output: WrappedIOType) -> Result<(), FeagiDataError> {
    let number_of_stages = pipeline_stage_properties.len();

    if number_of_stages == 0 {
        if expected_input != expected_output {
            return Err(FeagiDataError::BadParameters("If no pipeline stages are given, the expected input data properties must match the expected output data properties!".into()));
        }
        return Ok(())
    }

    // Ensure data can pass between processing
    for stage_index in 0..number_of_stages - 1  {
        let first = &pipeline_stage_properties[stage_index];
        let second = &pipeline_stage_properties[stage_index + 1];
        if first.get_output_data_type() != second.get_input_data_type() { // TODO there may be some cases where one side doesnt care about things like resolution and stuff. Use those checks instead of this!
            return Err(FeagiDataError::BadParameters(format!("Given stage runner at index {} has output type {}, which does not match the input type of stage runner at index {} or type {}!",
                                                             stage_index, first.get_output_data_type(), stage_index + 1,  second.get_input_data_type()).into()).into());
        }
    };
    Ok(())
}

fn verify_replacing_stage_properties(current_stages: &Vec<Box<dyn PipelineStage>>,
                                     new_stage_properties: &Box<dyn PipelineStageProperties + Sync + Send>,
                                     pipeline_input_type: &WrappedIOType, pipeline_output_type: &WrappedIOType,
                                     new_stage_index: PipelineStagePropertyIndex) -> Result<(), FeagiDataError> {

    // WARNING: assumes new_stage_index is valid!

    let comparing_input = if *new_stage_index == 0 {
        pipeline_input_type
    } else {
        &current_stages[*new_stage_index as usize - 1].get_output_data_type()
    };

    let comparing_output = if *new_stage_index == (current_stages.len()- 1) as u32 {
        pipeline_output_type
    } else {
        &current_stages[*new_stage_index as usize + 1].get_output_data_type()
    };

    if comparing_input != &new_stage_properties.get_input_data_type() {
        return Err(FeagiDataError::BadParameters(format!("Precursor to stage at index {} outputs data type {} but given stage accepts {}!", *new_stage_index, comparing_input, new_stage_properties.get_input_data_type()).into()).into());
    }
    if comparing_output != &new_stage_properties.get_output_data_type() {
        return Err(FeagiDataError::BadParameters(format!("Precursor to stage at index {} outputs data type {} but given stage accepts {}!", *new_stage_index, comparing_output, new_stage_properties.get_output_data_type()).into()).into());
    }
    Ok(())
}


use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::PipelineStagePropertyIndex;
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::PipelineStageProperties;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

use super::pipeline_stage_runner_common::{PipelineDirection, PipelineStageRunner};

#[derive(Debug)]
pub(crate) struct SensoryPipelineStageRunner {
    expected_processed_sensor_type: WrappedIOType, // The type expected to be output by the stage runner
    last_instant_data_processed: Instant,
    pipeline_stages: Vec<Box<dyn PipelineStage>>,
    preprocessed_cached_value: WrappedIOData
}

impl PipelineStageRunner for SensoryPipelineStageRunner {
    fn get_direction(&self) -> PipelineDirection {
        PipelineDirection::Sensory
    }

    fn get_fixed_type(&self) -> &WrappedIOType {
        &self.expected_processed_sensor_type
    }

    fn get_stages(&self) -> &Vec<Box<dyn PipelineStage>> {
        &self.pipeline_stages
    }

    fn get_stages_mut_internal(&mut self) -> &mut Vec<Box<dyn PipelineStage>> {
        &mut self.pipeline_stages
    }

    fn get_preprocessed_cached_value(&self) -> &WrappedIOData {
        &self.preprocessed_cached_value
    }

    fn get_last_processed_instant(&self) -> Instant {
        self.last_instant_data_processed
    }
}

impl SensoryPipelineStageRunner {

    /// Creates a new pipeline stage runner with the specified configuration.
    pub fn new(initial_sensory_cached_value: WrappedIOData) -> Result<Self, FeagiDataError> {
        let type_to_be_outputted: WrappedIOType = (&initial_sensory_cached_value).into();

        Ok(SensoryPipelineStageRunner {
            expected_processed_sensor_type: type_to_be_outputted,
            last_instant_data_processed: Instant::now(),
            pipeline_stages: Vec::new(),
            preprocessed_cached_value: initial_sensory_cached_value
        })
    }

    //region Data

    /// Returns the type of data expected to be inputted
    ///
    /// This is determined by the input type of the first processor in the chain.
    /// Used for validation before processing new input data.
    pub fn get_expected_type_to_input_and_process(&self) -> WrappedIOType {
        if self.does_contain_stages() {
            return self.pipeline_stages.first().unwrap().get_input_data_type()
        }
        self.expected_processed_sensor_type
    }

    pub fn get_final_processed_type(&self) -> WrappedIOType {
        self.expected_processed_sensor_type
    }

    /// Returns OK if the given data is compatible with the current processing stages.
    /// Otherwise returns an error
    pub fn verify_input_sensor_data(&self, incoming_data: &WrappedIOData) -> Result<(), FeagiDataError> {
        let incoming_type: WrappedIOType = incoming_data.into();
        if incoming_type != self.get_expected_type_to_input_and_process() {
            return Err(FeagiDataError::BadParameters(format!("Expected input data type to be {} but got {incoming_type}!", self.get_expected_type_to_input_and_process())))
        }
        Ok(())
    }

    pub(crate) fn get_preprocessed_cached_value_mut(&mut self) -> &mut WrappedIOData {
        // WARNING: DOES NOT CHECK TYPE!
        &mut self.preprocessed_cached_value
    }


    /// Returns the most recent output from the last element in the processor chain (if one exists).
    /// Otherwise, returns the last cached input of this struct that had no processing applied.
    /// Guaranteed to be of the same type and properties as defined by self.get_output_data_type().
    ///
    /// # Returns
    /// Reference to the output data from the last processor in the chain or from the internal cache.
    pub fn get_postprocessed_sensor_value(&self) -> &WrappedIOData {
        if self.pipeline_stages.is_empty() {
            return &self.preprocessed_cached_value;
        }
        self.pipeline_stages.last().unwrap().get_most_recent_output()
    }

    /// Processes the currently cached value through the pipeline stages (if available), then returns a reference to the result
    pub fn process_cached_sensor_value(&mut self, time_of_update: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        if self.pipeline_stages.is_empty() {
            return Ok(&self.preprocessed_cached_value);
        }

        // Process the first processor with the input value
        self.pipeline_stages[0].process_new_input(&self.preprocessed_cached_value, time_of_update)?;

        // Process subsequent processing using split_at_mut to avoid borrowing conflicts
        for i in 1..self.pipeline_stages.len() {
            let (left, right) = self.pipeline_stages.split_at_mut(i);
            let previous_output = left[i - 1].get_most_recent_output();
            right[0].process_new_input(previous_output, time_of_update)?;
        }

        self.last_instant_data_processed = time_of_update;
        Ok(self.get_postprocessed_sensor_value()) // Return the output from the last processor
    }

    //endregion

    //region Pipeline Stages (delegating to trait)

    /// Returns true if 1 or more processing stages are within the pipeline stage runner.
    pub fn does_contain_stages(&self) -> bool {
        PipelineStageRunner::does_contain_stages(self)
    }

    /// Retrieves the properties of a single stage in the pipeline.
    pub fn try_get_single_stage_properties(&self, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        PipelineStageRunner::try_get_single_stage_properties(self, stage_index)
    }

    /// Retrieves the properties of all stages in the pipeline.
    pub fn get_all_stage_properties(&self) -> Vec<Box<dyn PipelineStageProperties + Sync + Send>> {
        PipelineStageRunner::get_all_stage_properties(self)
    }

    /// Updates the properties of a single stage in the pipeline.
    pub fn try_update_single_stage_properties(&mut self, updating_stage_index: PipelineStagePropertyIndex, updated_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_update_single_stage_properties(self, updating_stage_index, updated_properties)
    }

    /// Updates the properties of all stages in the pipeline.
    pub fn try_update_all_stage_properties(&mut self, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_update_all_stage_properties(self, new_pipeline_stage_properties)
    }

    /// Replaces a single stage in the pipeline with a new stage.
    pub fn try_replace_single_stage(&mut self, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_replace_single_stage(self, replacing_at_index, new_pipeline_stage_properties)
    }

    /// Replaces all stages in the pipeline with new stages.
    pub fn try_replace_all_stages(&mut self, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_replace_all_stages(self, new_pipeline_stage_properties)
    }

    /// Tries replacing all stages with nothing (remove all stages)
    pub fn try_removing_all_stages(&mut self) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_removing_all_stages(self)
    }

    //endregion
}

use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::PipelineStageProperties;
use crate::data_pipeline::PipelineStagePropertyIndex;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_structures::FeagiDataError;
use std::time::Instant;

use super::pipeline_stage_runner_common::{PipelineDirection, PipelineStageRunner};

#[derive(Debug)]
pub(crate) struct MotorPipelineStageRunner {
    expected_decoded_motor_type: WrappedIOType,
    last_instant_data_processed: Instant,
    pipeline_stages: Vec<Box<dyn PipelineStage>>,
    preprocessed_cached_value: WrappedIOData,
    channel_friendly_name: String,
    channel_index_override: Option<usize>,
}

impl PipelineStageRunner for MotorPipelineStageRunner {
    fn get_direction(&self) -> PipelineDirection {
        PipelineDirection::Motor
    }

    fn get_fixed_type(&self) -> &WrappedIOType {
        &self.expected_decoded_motor_type
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

    fn get_channel_friendly_name(&self) -> &str {
        &self.channel_friendly_name
    }

    fn set_channel_friendly_name(&mut self, channel_friendly_name: String) {
        self.channel_friendly_name = channel_friendly_name;
    }

    fn get_channel_index_override(&self) -> Option<usize> {
        self.channel_index_override
    }

    fn set_channel_index_override(&mut self, channel_index_override: Option<usize>) {
        self.channel_index_override = channel_index_override;
    }
}

impl MotorPipelineStageRunner {
    /// Creates a new pipeline stage runner with the specified configuration.
    pub fn new(initial_motor_cached_value: WrappedIOData) -> Result<Self, FeagiDataError> {
        let expected_decoded_motor_type: WrappedIOType = (&initial_motor_cached_value).into();

        Ok(MotorPipelineStageRunner {
            expected_decoded_motor_type,
            last_instant_data_processed: Instant::now(),
            pipeline_stages: Vec::new(),
            preprocessed_cached_value: initial_motor_cached_value,
            channel_friendly_name: String::new(),
            channel_index_override: None,
        })
    }

    //region Data

    /// Returns the type of data expected to be outputted
    #[allow(dead_code)]
    pub fn get_expected_type_to_output_after_processing(&self) -> WrappedIOType {
        if self.does_contain_stages() {
            return self.pipeline_stages.last().unwrap().get_output_data_type();
        }
        self.expected_decoded_motor_type
    }

    #[allow(dead_code)]
    pub fn get_initial_decoded_type(&self) -> WrappedIOType {
        self.expected_decoded_motor_type
    }

    pub(crate) fn get_preprocessed_cached_value_mut(&mut self) -> &mut WrappedIOData {
        // WARNING: DOES NOT CHECK TYPE!
        &mut self.preprocessed_cached_value
    }

    pub fn get_postprocessed_motor_value(&self) -> &WrappedIOData {
        if self.pipeline_stages.is_empty() {
            return &self.preprocessed_cached_value;
        }
        self.pipeline_stages
            .last()
            .unwrap()
            .get_most_recent_output()
    }

    /// Processes the currently cached value through the pipeline stages (if available), then returns a reference to the result
    pub fn process_cached_decoded_motor_value(
        &mut self,
        time_of_update: Instant,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        if self.pipeline_stages.is_empty() {
            return Ok(&self.preprocessed_cached_value);
        }

        // Process the first processor with the input value
        self.pipeline_stages[0]
            .process_new_input(&self.preprocessed_cached_value, time_of_update)?;

        // Process subsequent processing using split_at_mut to avoid borrowing conflicts
        for i in 1..self.pipeline_stages.len() {
            let (left, right) = self.pipeline_stages.split_at_mut(i);
            let previous_output = left[i - 1].get_most_recent_output();
            right[0].process_new_input(previous_output, time_of_update)?;
        }

        self.last_instant_data_processed = time_of_update;
        Ok(self.get_postprocessed_motor_value()) // Return the output from the last processor
    }

    //endregion

    //region Pipeline Stages (delegating to trait)

    /// Returns true if 1 or more processing stages are within the pipeline stage runner.
    #[allow(dead_code)]
    pub fn does_contain_stages(&self) -> bool {
        PipelineStageRunner::does_contain_stages(self)
    }

    /// Retrieves the properties of a single stage in the pipeline.
    pub fn try_get_single_stage_properties(
        &self,
        stage_index: PipelineStagePropertyIndex,
    ) -> Result<PipelineStageProperties, FeagiDataError> {
        PipelineStageRunner::try_get_single_stage_properties(self, stage_index)
    }

    /// Retrieves the properties of all stages in the pipeline.
    pub fn get_all_stage_properties(&self) -> Vec<PipelineStageProperties> {
        PipelineStageRunner::get_all_stage_properties(self)
    }

    /// Updates the properties of a single stage in the pipeline.
    pub fn try_update_single_stage_properties(
        &mut self,
        updating_stage_index: PipelineStagePropertyIndex,
        updated_properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_update_single_stage_properties(
            self,
            updating_stage_index,
            updated_properties,
        )
    }

    /// Updates the properties of all stages in the pipeline.
    pub fn try_update_all_stage_properties(
        &mut self,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_update_all_stage_properties(self, new_pipeline_stage_properties)
    }

    /// Replaces a single stage in the pipeline with a new stage.
    pub fn try_replace_single_stage(
        &mut self,
        replacing_at_index: PipelineStagePropertyIndex,
        new_pipeline_stage_properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_replace_single_stage(
            self,
            replacing_at_index,
            new_pipeline_stage_properties,
        )
    }

    /// Replaces all stages in the pipeline with new stages.
    pub fn try_replace_all_stages(
        &mut self,
        new_pipeline_stage_properties: Vec<PipelineStageProperties>,
    ) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_replace_all_stages(self, new_pipeline_stage_properties)
    }

    /// Tries replacing all stages with nothing (remove all stages)
    pub fn try_removing_all_stages(&mut self) -> Result<(), FeagiDataError> {
        PipelineStageRunner::try_removing_all_stages(self)
    }

    //endregion
}

use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::{stage_properties_to_stages, PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

/// Manages and executes a pipeline of processing stages for data transformation.
///
/// A `PipelineStageRunner` orchestrates a series of data processing stages, ensuring
/// type compatibility between stages and managing the flow of data from input to output.
/// Each stage in the pipeline transforms data from one type to another, with the output
/// of one stage feeding into the input of the next.
///
/// # Fields
/// - `input_type`: The expected data type for the pipeline's initial input
/// - `output_type`: The expected data type for the pipeline's final output
/// - `last_instant_data_processed`: Timestamp of the most recent data processing
/// - `pipeline_stages`: Ordered sequence of processing stages
/// - `cached_input`: The most recently provided input data
#[derive(Debug)]
pub(crate) struct PipelineStageRunner {
    input_type: WrappedIOType,
    output_type: WrappedIOType,
    last_instant_data_processed: Instant,
    pipeline_stages: Vec<Box<dyn PipelineStage>>,
    cached_input: WrappedIOData
}

impl PipelineStageRunner {
    /// Creates a new pipeline stage runner with the specified configuration.
    ///
    /// Validates that the pipeline stages are compatible with each other and with
    /// the expected input and output types. The pipeline stages are initialized
    /// from the provided properties.
    ///
    /// # Arguments
    /// * `pipeline_stage_properties` - Configuration for each stage in the pipeline
    /// * `cached_input_value` - Initial input value to cache (determines input type)
    /// * `expected_output_type` - The data type that the final stage should produce
    ///
    /// # Returns
    /// * `Ok(PipelineStageRunner)` - Successfully created pipeline runner
    /// * `Err(FeagiDataError)` - If stages are incompatible or validation fails
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

    pub fn verify_input_data(&self, incoming_data: &WrappedIOData) -> Result<(), FeagiDataError> {
        let incoming_type: WrappedIOType = incoming_data.into();
        if incoming_type != self.input_type {
            return Err(FeagiDataError::BadParameters(format!("Expected input data type to be {} but got {incoming_type}!", self.input_type)))
        }
        Ok(())
    }

    /// Returns the output data type produced by this processor chain.
    ///
    /// This is determined by the output type of the last processor in the chain.
    /// Useful for understanding what type of data the pipeline will produce.
    pub fn get_output_data_type(&self) -> WrappedIOType {
        self.output_type
    }


    /// Returns the last cached input of this struct that had no processing applied.
    /// Guaranteed to be of the same type and properties as defined by self.get_output_data_type().
    ///
    /// # Returns
    /// Reference to the cached value (before any processing)
    pub fn get_most_recent_preprocessed_output(&self) -> &WrappedIOData {
        &self.cached_input
    }

    /// Returns the most recent output from the last element in the processor chain (if one exists).
    /// Otherwise, returns the last cached input of this struct that had no processing applied.
    /// Guaranteed to be of the same type and properties as defined by self.get_output_data_type().
    ///
    /// # Returns
    /// Reference to the output data from the last processor in the chain or from the internal cache.
    pub fn get_most_recent_postprocessed_output(&self) -> &WrappedIOData {
        if self.pipeline_stages.is_empty() {
            return &self.cached_input;
        }
        self.pipeline_stages.last().unwrap().get_most_recent_output()
    }

    /// Returns the timestamp of the most recent data processing operation.
    ///
    /// This timestamp is updated each time `try_update_value` successfully processes
    /// new input through the pipeline. Useful for tracking data freshness and timing.
    ///
    /// # Returns
    /// The `Instant` when data was last processed through the pipeline.
    pub fn get_last_processed_instant(&self) -> Instant {
        self.last_instant_data_processed
    }

    pub(crate) fn get_cached_input_mut(&mut self) -> &mut WrappedIOData {
        // WARNING: DOES NOT CHECK TYPE!
        &mut self.cached_input
    }

    pub fn set_cached_input_value(&mut self, value: WrappedIOData) -> Result<(), FeagiDataError> {
        self.verify_input_data(&value)?;
        self.cached_input = value;
        Ok(())
    }

    pub fn process_cached_input_value(&mut self, time_of_update: Instant) -> Result<&WrappedIOData, FeagiDataError> {
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
        Ok(self.get_most_recent_postprocessed_output()) // Return the output from the last processor
    }

    //endregion

    //region Pipeline Stages

    /// Returns true if 1 or more processing stages are within the pipeline stage runner.
    pub fn does_contain_stages(&self) -> bool {
        self.pipeline_stages.len() != 0
    }

    /// Retrieves the properties of a single stage in the pipeline.
    ///
    /// # Arguments
    /// * `stage_index` - The index of the stage to retrieve properties from
    ///
    /// # Returns
    /// * `Ok(Box<dyn PipelineStageProperties>)` - The stage's properties
    /// * `Err(FeagiDataError)` - If the index is invalid or out of bounds
    pub fn try_get_single_stage_properties(&self, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        self.verify_pipeline_stage_index(stage_index)?;
        Ok(self.pipeline_stages[*stage_index as usize].create_properties())
    }

    /// Retrieves the properties of all stages in the pipeline.
    ///
    /// Creates a vector containing property objects for each stage in the pipeline,
    /// in order from first to last stage.
    ///
    /// # Returns
    /// A vector of boxed pipeline stage properties for all stages.
    pub fn get_all_stage_properties(&self) -> Vec<Box<dyn PipelineStageProperties + Sync + Send>>  {
        let mut output: Vec<Box<dyn PipelineStageProperties + Sync + Send>> = Vec::with_capacity(self.pipeline_stages.len());
        for stage in &self.pipeline_stages {
            output.push(stage.create_properties())
        }
        output
    }

    /// Updates the properties of a single stage in the pipeline.
    ///
    /// Modifies the configuration of an existing stage without replacing the stage
    /// itself. The stage must support loading the provided properties.
    ///
    /// # Arguments
    /// * `updating_stage_index` - The index of the stage to update
    /// * `updated_properties` - The new properties to apply to the stage
    ///
    /// # Returns
    /// * `Ok(())` - If the properties were successfully updated
    /// * `Err(FeagiDataError)` - If the index is invalid or properties can't be loaded
    pub fn try_update_single_stage_properties(&mut self, updating_stage_index: PipelineStagePropertyIndex, updated_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        self.verify_pipeline_stage_index(updating_stage_index)?;
        self.pipeline_stages[*updating_stage_index as usize].load_properties(updated_properties)?;
        Ok(())
    }

    /// Updates the properties of all stages in the pipeline.
    ///
    /// Applies new properties to each existing stage in the pipeline. The number of
    /// properties provided must match the number of stages. Does not replace stages,
    /// only updates their configurations.
    ///
    /// # Arguments
    /// * `new_pipeline_stage_properties` - Vector of new properties for each stage
    ///
    /// # Returns
    /// * `Ok(())` - If all properties were successfully updated
    /// * `Err(FeagiDataError)` - If property count doesn't match or loading fails
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

    /// Replaces a single stage in the pipeline with a new stage.
    ///
    /// Unlike `try_update_single_stage_properties`, this completely replaces the stage
    /// with a new one created from the provided properties. The new stage must have
    /// compatible input/output types with adjacent stages.
    ///
    /// # Arguments
    /// * `replacing_at_index` - The index of the stage to replace
    /// * `new_pipeline_stage_properties` - Properties to create the new stage from
    ///
    /// # Returns
    /// * `Ok(())` - If the stage was successfully replaced
    /// * `Err(FeagiDataError)` - If index is invalid or types are incompatible
    pub fn try_replace_single_stage(&mut self, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        self.verify_pipeline_stage_index(replacing_at_index)?;
        verify_replacing_stage_properties(&self.pipeline_stages, &new_pipeline_stage_properties, &self.input_type, &self.output_type, replacing_at_index)?;
        self.pipeline_stages[*replacing_at_index as usize] = new_pipeline_stage_properties.create_stage();
        Ok(())
    }

    /// Replaces all stages in the pipeline with new stages.
    ///
    /// Completely rebuilds the pipeline with new stages created from the provided
    /// properties. The new stages must be compatible with the pipeline's expected
    /// input and output types.
    ///
    /// # Arguments
    /// * `new_pipeline_stage_properties` - Properties for all new stages
    ///
    /// # Returns
    /// * `Ok(())` - If all stages were successfully replaced
    /// * `Err(FeagiDataError)` - If stages are incompatible or creation fails
    pub fn try_replace_all_stages(&mut self, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        verify_pipeline_stage_properties(&new_pipeline_stage_properties, self.input_type, self.output_type)?;
        self.pipeline_stages = stage_properties_to_stages(&new_pipeline_stage_properties)?;
        Ok(())
    }

    /// Tries replacing all stages with nothing (remove all stages)
    ///
    /// Checks to ensure that the input and output properties of the stage runner are the same
    /// in order to do this safely
    /// # Returns
    /// * `Ok(())` - If all stages were successfully removed
    /// * `Err(FeagiDataError)` - If input and output properties do not match
    pub fn try_removing_all_stages(&mut self) -> Result<(), FeagiDataError> {
        if self.pipeline_stages.is_empty() {
            return Ok(());
        }
        verify_pipeline_stage_properties(&Vec::new(), self.input_type, self.output_type)?;
        self.pipeline_stages = stage_properties_to_stages(&Vec::new())?;
        Ok(())
    }

    //endregion

    //region Internal

    /// Validates that a given stage index is within the valid range.
    ///
    /// # Arguments
    /// * `stage_index` - The index to validate
    ///
    /// # Returns
    /// * `Ok(())` - If the index is valid
    /// * `Err(FeagiDataError)` - If no stages exist or index is out of bounds
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

/// Validates that a collection of pipeline stage properties are compatible.
///
/// Ensures that:
/// - If no stages are provided, input and output types must match
/// - All adjacent stages have compatible input/output types
/// - The data can flow properly through the entire pipeline
///
/// # Arguments
/// * `pipeline_stage_properties` - The stages to validate
/// * `expected_input` - The expected input type for the first stage
/// * `expected_output` - The expected output type for the last stage
///
/// # Returns
/// * `Ok(())` - If all stages are compatible
/// * `Err(FeagiDataError)` - If stages are incompatible or types don't match
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

/// Validates that a new stage can replace an existing stage without breaking the pipeline.
///
/// Checks that the new stage's input type matches what the previous stage outputs,
/// and that its output type matches what the next stage expects. For the first stage,
/// validates against the pipeline input type. For the last stage, validates against
/// the pipeline output type.
///
/// # Arguments
/// * `current_stages` - The existing pipeline stages
/// * `new_stage_properties` - Properties for the stage to be inserted
/// * `pipeline_input_type` - The pipeline's overall input type
/// * `pipeline_output_type` - The pipeline's overall output type
/// * `new_stage_index` - Index where the new stage will be placed
///
/// # Returns
/// * `Ok(())` - If the replacement is valid
/// * `Err(FeagiDataError)` - If input/output types are incompatible
///
/// # Note
/// Assumes `new_stage_index` has already been validated as within bounds.
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


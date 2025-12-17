use std::cmp::PartialEq;
use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::{stage_properties_to_stages, PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

/// Represents the direction of data flow in the pipeline, which affects validation logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PipelineDirection {
    // Data flows forward through stages, with a fixed OUTPUT type requirement.
    Sensory,
    //Data flows through stages with a fixed INPUT type requirement.
    Motor,
}

/// Core trait providing common functionality for pipeline stage runners.
///
/// This trait encapsulates the shared behavior between `SensoryPipelineStageRunner`
/// and `MotorPipelineStageRunner`, including stage management, property access,
/// and validation utilities.
pub(crate) trait PipelineStageRunner {
    /// Returns the direction of this pipeline
    fn get_direction(&self) -> PipelineDirection;

    /// Returns the fixed type constraint for this pipeline.
    /// For Sensory: This is the expected output type after all processing
    /// For Motor: This is the expected neuron decoded type before processing
    fn get_fixed_type(&self) -> &WrappedIOType;

    /// Returns a reference to the pipeline stages
    fn get_stages(&self) -> &Vec<Box<dyn PipelineStage>>;

    /// Returns a mutable reference to the pipeline stages vector
    fn get_stages_mut_internal(&mut self) -> &mut Vec<Box<dyn PipelineStage>>;

    /// Returns a reference to the preprocessed cached value
    fn get_preprocessed_cached_value(&self) -> &WrappedIOData;

    /// Returns the last instant when data was processed
    fn get_last_processed_instant(&self) -> Instant;

    //region Defaults Implementations
    
    fn does_contain_stages(&self) -> bool {
        !self.get_stages().is_empty()
    }

    fn get_postprocessed_cached_value(&self) -> &WrappedIOData {
        let stages = self.get_stages();
        if stages.is_empty() {
            return self.get_preprocessed_cached_value()
        }
        stages.last().unwrap().get_most_recent_output()
    }

    /// Retrieves the properties of a single stage in the pipeline.
    fn try_get_single_stage_properties(&self, stage_index: PipelineStagePropertyIndex) -> Result<PipelineStageProperties, FeagiDataError> {
        self.verify_pipeline_stage_index_in_range(stage_index)?;
        Ok(self.get_stages()[*stage_index as usize].create_properties())
    }

    /// Retrieves the properties of all stages in the pipeline.
    fn get_all_stage_properties(&self) -> Vec<PipelineStageProperties> {
        self.get_stages()
            .iter()
            .map(|stage| stage.create_properties())
            .collect()
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
    fn try_update_single_stage_properties(&mut self, updating_stage_index: PipelineStagePropertyIndex, updated_properties: PipelineStageProperties) -> Result<(), FeagiDataError> {
        self.verify_pipeline_stage_index_in_range(updating_stage_index)?;
        self.get_stages_mut_internal()[*updating_stage_index as usize].load_properties(updated_properties)?;
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
    /// * `Err(FeagiDataError)` - if given pipeline is not compatible with current stages
    fn try_update_all_stage_properties(&mut self, new_pipeline_stage_properties: Vec<PipelineStageProperties>) -> Result<(), FeagiDataError> {
        if new_pipeline_stage_properties.len() != self.get_stages().len() {
            return Err(FeagiDataError::BadParameters(format!(
                "Unable to update {} contained stages with {} properties!",
                self.get_stages().len(),
                new_pipeline_stage_properties.len()
            )));
        }
        self.get_stages_mut_internal()
            .iter_mut()
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
    fn try_replace_single_stage(&mut self, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: PipelineStageProperties) -> Result<(), FeagiDataError> {
        self.verify_pipeline_stage_index_in_range(replacing_at_index)?;
        verify_replacing_stage_properties(
            self.get_stages(),
            &new_pipeline_stage_properties,
            &self.get_fixed_type(),
            replacing_at_index,
            self.get_direction(),
        )?;
        self.get_stages_mut_internal()[*replacing_at_index as usize] = new_pipeline_stage_properties.create_stage();
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
    fn try_replace_all_stages(&mut self, new_pipeline_stage_properties: Vec<PipelineStageProperties>) -> Result<(), FeagiDataError> {
        verify_pipeline_stage_properties(
            &new_pipeline_stage_properties,
            self.get_fixed_type(),
            self.get_direction(),
        )?;
        *self.get_stages_mut_internal() = stage_properties_to_stages(&new_pipeline_stage_properties)?;
        Ok(())
    }

    /// Tries replacing all stages with nothing (remove all stages)
    ///
    /// # Returns
    /// * `Ok(())` - If all stages were successfully removed
    fn try_removing_all_stages(&mut self) -> Result<(), FeagiDataError> {
        if self.get_stages().is_empty() {
            return Ok(());
        }
        *self.get_stages_mut_internal() = stage_properties_to_stages(&Vec::new())?;
        Ok(())
    }

    /// Validates that a given stage index is within the valid range.
    ///
    /// # Arguments
    /// * `stage_index` - The index to validate
    ///
    /// # Returns
    /// * `Ok(())` - If the index is valid
    /// * `Err(FeagiDataError)` - If no stages exist or index is out of bounds
    fn verify_pipeline_stage_index_in_range(&self, stage_index: PipelineStagePropertyIndex) -> Result<(), FeagiDataError> {
        if self.get_stages().is_empty() {
            return Err(FeagiDataError::BadParameters("No stages are defined, ergo no indexing is possible!".into()));
        }

        if *stage_index >= self.get_stages().len() as u32 {
            return Err(FeagiDataError::BadParameters(format!(
                "New stage index {} is out of range! Max allowed is {}!",
                *stage_index,
                self.get_stages().len() - 1
            )));
        }
        Ok(())
    }

    //endregion
}

/// Verifies that a sequence of pipeline stage properties is valid for the given direction.
///
/// # Arguments
/// * `pipeline_stage_properties` - The properties to validate
/// * `fixed_type` - The type that must match at the fixed end (input for Motor, output for Sensory)
/// * `direction` - The pipeline direction
///
/// # Returns
/// * `Ok(())` - If the properties form a valid pipeline
/// * `Err(FeagiDataError)` - If the properties are incompatible
pub(crate) fn verify_pipeline_stage_properties(
    pipeline_stage_properties: &Vec<PipelineStageProperties>,
    fixed_type: &WrappedIOType,
    direction: PipelineDirection,
) -> Result<(), FeagiDataError> {
    let number_of_stages = pipeline_stage_properties.len();

    if number_of_stages == 0 {
        return Ok(());
    }

    // Check the fixed end matches
    match direction {
        PipelineDirection::Motor => {
            // For motor: first stage's input must match the fixed type
            if &pipeline_stage_properties.first().unwrap().get_input_data_type() != fixed_type {
                return Err(FeagiDataError::BadParameters("Given stages not compatible!".into()));
            }
        }
        PipelineDirection::Sensory => {
            // For sensory: last stage's output must match the fixed type
            if &pipeline_stage_properties.last().unwrap().get_output_data_type() != fixed_type {
                return Err(FeagiDataError::BadParameters("Given stages not compatible!".into()));
            }
        }
    }

    // Ensure data can pass between processing stages
    for stage_index in 0..number_of_stages - 1 {
        let first = &pipeline_stage_properties[stage_index];
        let second = &pipeline_stage_properties[stage_index + 1];
        if first.get_output_data_type() != second.get_input_data_type() {
            // TODO there may be some cases where one side doesn't care about things like resolution and stuff. Use those checks instead of this!
            return Err(FeagiDataError::BadParameters(format!(
                "Given stage runner at index {} has output type {}, which does not match the input type of stage runner at index {} or type {}!",
                stage_index,
                first.get_output_data_type(),
                stage_index + 1,
                second.get_input_data_type()
            )));
        }
    }
    Ok(())
}

/// Verifies that a replacement stage is compatible with its neighbors in the pipeline.
///
/// # Arguments
/// * `current_stages` - The current stages in the pipeline
/// * `new_stage_properties` - The properties of the stage to insert
/// * `fixed_type` - The type constraint at the fixed end
/// * `new_stage_index` - The index where the stage will be inserted
/// * `direction` - The pipeline direction
///
/// # Returns
/// * `Ok(())` - If the replacement is compatible
/// * `Err(FeagiDataError)` - If types don't match
#[inline]
pub(crate) fn verify_replacing_stage_properties(
    current_stages: &Vec<Box<dyn PipelineStage>>,
    new_stage_properties: &PipelineStageProperties,
    fixed_type: &WrappedIOType,
    new_stage_index: PipelineStagePropertyIndex,
    direction: PipelineDirection,
) -> Result<(), FeagiDataError> {
    // WARNING: assumes new_stage_index is valid!

    let is_first = *new_stage_index == 0;
    let is_last = *new_stage_index == (current_stages.len() - 1) as u32;

    // Determine what input type we're comparing against
    let comparing_input = match direction {
        PipelineDirection::Motor => {
            // For motor: index 0 must match the fixed input type
            if is_first {
                Some(fixed_type)
            } else {
                Some(&current_stages[*new_stage_index as usize - 1].get_output_data_type())
            }
        }
        PipelineDirection::Sensory => {
            // For sensory: index 0 has no input constraint from pipeline
            if is_first {
                None
            } else {
                Some(&current_stages[*new_stage_index as usize - 1].get_output_data_type())
            }
        }
    };

    // Determine what output type we're comparing against
    let comparing_output = match direction {
        PipelineDirection::Motor => {
            // For motor: last index has no output constraint from pipeline
            if is_last {
                None
            } else {
                Some(&current_stages[*new_stage_index as usize + 1].get_input_data_type())
            }
        }
        PipelineDirection::Sensory => {
            // For sensory: last index must match the fixed output type
            if is_last {
                Some(fixed_type)
            } else {
                Some(&current_stages[*new_stage_index as usize + 1].get_input_data_type())
            }
        }
    };

    // Validate input compatibility
    if let Some(expected_input) = comparing_input {
        if expected_input != &new_stage_properties.get_input_data_type() {
            return Err(FeagiDataError::BadParameters(format!(
                "Precursor to stage at index {} outputs data type {} but given stage accepts {}!",
                *new_stage_index,
                expected_input,
                new_stage_properties.get_input_data_type()
            )));
        }
    }

    // Validate output compatibility
    if let Some(expected_output) = comparing_output {
        if expected_output != &new_stage_properties.get_output_data_type() {
            return Err(FeagiDataError::BadParameters(format!(
                "Postcursor to stage at index {} receives data type {} but given stage outputs {}!",
                *new_stage_index,
                expected_output,
                new_stage_properties.get_output_data_type()
            )));
        }
    }

    Ok(())
}

/*
——— No Performant Parallelism? ———
⠀⣞⢽⢪⢣⢣⢣⢫⡺⡵⣝⡮⣗⢷⢽⢽⢽⣮⡷⡽⣜⣜⢮⢺⣜⢷⢽⢝⡽⣝
⠸⡸⠜⠕⠕⠁⢁⢇⢏⢽⢺⣪⡳⡝⣎⣏⢯⢞⡿⣟⣷⣳⢯⡷⣽⢽⢯⣳⣫⠇
⠀⠀⢀⢀⢄⢬⢪⡪⡎⣆⡈⠚⠜⠕⠇⠗⠝⢕⢯⢫⣞⣯⣿⣻⡽⣏⢗⣗⠏⠀
⠀⠪⡪⡪⣪⢪⢺⢸⢢⢓⢆⢤⢀⠀⠀⠀⠀⠈⢊⢞⡾⣿⡯⣏⢮⠷⠁⠀⠀
⠀⠀⠀⠈⠊⠆⡃⠕⢕⢇⢇⢇⢇⢇⢏⢎⢎⢆⢄⠀⢑⣽⣿⢝⠲⠉⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡿⠂⠠⠀⡇⢇⠕⢈⣀⠀⠁⠡⠣⡣⡫⣂⣿⠯⢪⠰⠂⠀⠀⠀⠀
⠀⠀⠀⠀⡦⡙⡂⢀⢤⢣⠣⡈⣾⡃⠠⠄⠀⡄⢱⣌⣶⢏⢊⠂⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢝⡲⣜⡮⡏⢎⢌⢂⠙⠢⠐⢀⢘⢵⣽⣿⡿⠁⠁⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠨⣺⡺⡕⡕⡱⡑⡆⡕⡅⡕⡜⡼⢽⡻⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣼⣳⣫⣾⣵⣗⡵⡱⡡⢣⢑⢕⢜⢕⡝⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⣴⣿⣾⣿⣿⣿⡿⡽⡑⢌⠪⡢⡣⣣⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⡟⡾⣿⢿⢿⢵⣽⣾⣼⣘⢸⢸⣞⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠁⠇⠡⠩⡫⢿⣝⡻⡮⣒⢽⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
—————————————————————————————————
 */

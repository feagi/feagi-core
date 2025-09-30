use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::{stage_properties_to_stages, PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::pipeline_stage::PipelineStage;


#[derive(Debug)]
pub(crate) struct PipelineStageRunner {
    input_type: WrappedIOType,
    output_type: WrappedIOType,
    pipeline_stages: Vec<Box<dyn PipelineStage + Sync + Send>>,
}

impl PipelineStageRunner {
    /// Creates a new ProcessorRunner with a validated chain of processing.
    ///
    /// This constructor performs comprehensive validation to ensure the processor chain
    /// is valid and can execute successfully:
    /// - Checks that at least one processor is provided
    /// - Validates type compatibility between adjacent processing
    /// - Determines the overall input and output types for the pipeline
    ///
    /// # Arguments
    /// * `cache_processors` - Vector of processing to chain together (must be non-empty)
    ///
    /// # Returns
    /// * `Ok(ProcessorRunner)` - A validated processor runner ready for execution
    /// * `Err(FeagiDataProcessingError)` - If validation fails:
    ///   - Empty processor list
    ///   - Type incompatibility between adjacent processing
    ///
    /// # Type Compatibility Rules
    /// For processing to be compatible in a chain, each processor's output type
    /// must exactly match the next processor's input type:
    /// ```text
    /// Processor A: Input(F32) -> Output(F32Normalized0To1)
    /// Processor B: Input(F32Normalized0To1) -> Output(Bool)  ✓ Compatible
    /// 
    /// Processor A: Input(F32) -> Output(F32Normalized0To1)
    /// Processor B: Input(F32) -> Output(Bool)              ✗ Incompatible
    /// ```
    pub fn new(pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<Self, FeagiDataError> {
        verify_pipeline_stage_properties(&pipeline_stage_properties)?;
        let pipeline_stages = stage_properties_to_stages(&pipeline_stage_properties)?;
        
        Ok(PipelineStageRunner {
            input_type: pipeline_stages.first().unwrap().get_input_data_type(),
            output_type: pipeline_stages.last().unwrap().get_output_data_type(),
            pipeline_stages,
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

    /// Processes new input data through the entire processor chain.
    ///
    /// Takes input data, validates it matches the expected input type, then runs it
    /// sequentially through all processing in the chain. Each processor's output
    /// becomes the input for the next processor.
    ///
    /// # Arguments
    /// * `new_value` - Input data to process (must match the chain's input type)
    /// * `time_of_update` - Timestamp for when this update occurred
    ///
    /// # Returns
    /// * `Ok(&IOTypeData)` - Reference to the final processed output from the last processor
    /// * `Err(FeagiDataProcessingError)` - If processing fails:
    ///   - Input type doesn't match expected type
    ///   - Any processor in the chain fails to process its input
    ///
    /// # Processing Flow
    /// 1. Validate input type matches the chain's expected input type
    /// 2. Process input through first processor
    /// 3. For each subsequent processor, use previous processor's output as input
    /// 4. Return final output from the last processor
    ///
    /// # Performance Notes
    /// Uses `split_at_mut` to avoid borrowing conflicts when accessing processor outputs
    /// while mutating subsequent processing in the chain.
    pub fn update_value(&mut self, new_value: &WrappedIOData, time_of_update: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        if WrappedIOType::from(new_value) != self.input_type {
            return Err(FeagiDataError::BadParameters(format!("Expected Input data type of {} but received {}!", self.input_type.to_string(), new_value.to_string())).into());
        }

        //TODO There has to be a better way to do this, but I keep running into limitations with mutating self.cache_processors

        // Process the first processor with the input value
        self.pipeline_stages[0].process_new_input(new_value, time_of_update)?;

        // Process subsequent processing using split_at_mut to avoid borrowing conflicts
        for i in 1..self.pipeline_stages.len() {
            let (left, right) = self.pipeline_stages.split_at_mut(i);
            let previous_output = left[i - 1].get_most_recent_output();
            right[0].process_new_input(previous_output, time_of_update)?;
        }

        // Return the output from the last processor
        Ok(self.pipeline_stages.last().unwrap().get_most_recent_output())
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

    //endregion

    //region Pipeline Stages

    pub fn get_all_stage_properties(&self) -> Vec<Box<dyn PipelineStageProperties + Sync + Send>>  {
        let mut output: Vec<Box<dyn PipelineStageProperties + Sync + Send>> = Vec::with_capacity(self.pipeline_stages.len());
        for stage in &self.pipeline_stages {
            output.push(stage.create_properties())
        }
        output
    }

    pub fn get_single_stage_property(&self, stage_index: PipelineStagePropertyIndex) -> Result<Box<dyn PipelineStageProperties + Sync + Send>, FeagiDataError> {
        self.verify_pipeline_stage_index(stage_index)?;
        Ok(self.pipeline_stages[*stage_index as usize].create_properties())
    }

    pub fn try_update_single_stage_properties(&mut self, updating_stage_index: PipelineStagePropertyIndex, updated_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        self.verify_pipeline_stage_index(updating_stage_index)?;
        self.pipeline_stages[*updating_stage_index as usize].load_properties(updated_properties)?;
        Ok(())
    }

    // NOTE: No vector form of updating stage properties

    pub fn try_replace_all_stages(&mut self, new_pipeline_stage_properties: Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
        verify_pipeline_stage_properties(&new_pipeline_stage_properties)?;
        self.pipeline_stages = stage_properties_to_stages(&new_pipeline_stage_properties)?;
        Ok(())
    }

    pub fn try_replace_single_stage(&mut self, replacing_at_index: PipelineStagePropertyIndex, new_pipeline_stage_properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        self.verify_pipeline_stage_index(replacing_at_index)?;
        verify_replacing_stage_properties(&self.pipeline_stages, &new_pipeline_stage_properties, &self.input_type, &self.output_type, replacing_at_index)?;
        self.pipeline_stages[*replacing_at_index as usize] = new_pipeline_stage_properties.create_stage();
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
        if *stage_index >= self.pipeline_stages.len() as u32 {
            return Err(FeagiDataError::BadParameters(format!("New stage index {} is out of range! Max allowed is {}!", *stage_index, self.pipeline_stages.len()- 1)).into());
        }
        Ok(())
    }

    //endregion

}


fn verify_pipeline_stage_properties(pipeline_stage_properties: &Vec<Box<dyn PipelineStageProperties + Sync + Send>>) -> Result<(), FeagiDataError> {
    let number_of_stages = pipeline_stage_properties.len();

    if number_of_stages == 0 {
        return Err(FeagiDataError::BadParameters("Pipeline Stage Runner cannot have 0 Stages!".into()).into())
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

fn verify_replacing_stage_properties(current_stages: &Vec<Box<dyn PipelineStage + Sync + Send>>,
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


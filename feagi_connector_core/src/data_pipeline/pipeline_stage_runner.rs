use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::PipelineStageIndex;
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::verify_pipeline_stages::{verify_pipeline_stages, verify_replacing_stage};


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
    pub fn new(pipeline_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<Self, FeagiDataError> {

        verify_pipeline_stages(&pipeline_stages)?;
        
        Ok(PipelineStageRunner {
            input_type: pipeline_stages.first().unwrap().get_input_data_type(),
            output_type: pipeline_stages.last().unwrap().get_output_data_type(),
            pipeline_stages,
        })
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
    
    pub fn attempt_replace_stages(&mut self, new_pipeline_stages: Vec<Box<dyn PipelineStage + Sync + Send>>) -> Result<(), FeagiDataError> {
        verify_pipeline_stages(&new_pipeline_stages)?;
        self.pipeline_stages = new_pipeline_stages;
        Ok(())
    }

    pub fn attempt_replace_stage(&mut self, new_pipeline_stage: Box<dyn PipelineStage + Sync + Send>, replacing_at_index: PipelineStageIndex) -> Result<(), FeagiDataError> {
        verify_replacing_stage(&self.pipeline_stages, &new_pipeline_stage, &self.input_type, &self.output_type, replacing_at_index)?;
        self.pipeline_stages[*replacing_at_index as usize] = new_pipeline_stage;
        Ok(())
    }

    pub fn clone_stages(&self) -> Vec<Box<dyn PipelineStage + Sync + Send>> {
        let mut output: Vec<Box<dyn PipelineStage + Sync + Send>> = Vec::with_capacity(self.pipeline_stages.len());
        for pipeline_stage in self.pipeline_stages.iter() {
            output.push(pipeline_stage.clone_box())
        }
        output
    }

    pub fn clone_stage(&self, index: PipelineStageIndex) -> Result<Box<dyn PipelineStage + Sync + Send>, FeagiDataError> {
        if *index >= self.pipeline_stages.len() as u32 {
            return Err(FeagiDataError::BadParameters(format!("Pipeline Index {} out of bounds!", *index)).into());
        }
        Ok(self.pipeline_stages[*index as usize].clone_box())
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

}
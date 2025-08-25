use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::stream_cache_processor_trait::StreamCacheStage;
use crate::data_pipeline::verify_stream_cache_processor_chain::verify_sensor_chain;

/// Orchestrates execution of a chain of stream cache processing.
///
/// `ProcessorRunner` validates that a sequence of processing can be chained together
/// (ensuring output types match subsequent input types) and provides a unified interface
/// for processing data through the entire pipeline.
///
/// # Key Features
///
/// - **Type Validation**: Ensures processing are compatible before execution
/// - **Pipeline Execution**: Runs data through all processing in sequence
/// - **Error Handling**: Provides clear error messages for incompatible processing
/// - **Performance**: Uses efficient borrowing patterns to avoid unnecessary clones
#[derive(Debug)]
pub(crate) struct ProcessorRunner {
    input_type: WrappedIOType,
    output_type: WrappedIOType,
    cache_processors: Vec<Box<dyn StreamCacheStage + Sync + Send>>,
}

impl ProcessorRunner {
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
    pub fn new(cache_processors: Vec<Box<dyn StreamCacheStage + Sync + Send>>) -> Result<Self, FeagiDataError> {

        verify_sensor_chain(&cache_processors)?;
        
        Ok(ProcessorRunner {
            input_type: cache_processors.first().unwrap().get_input_data_type(),
            output_type: cache_processors.last().unwrap().get_output_data_type(),
            cache_processors,
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
        self.cache_processors[0].process_new_input(new_value, time_of_update)?;
        
        // Process subsequent processing using split_at_mut to avoid borrowing conflicts
        for i in 1..self.cache_processors.len() {
            let (left, right) = self.cache_processors.split_at_mut(i);
            let previous_output = left[i - 1].get_most_recent_output(); 
            right[0].process_new_input(previous_output, time_of_update)?;
        }
        
        // Return the output from the last processor
        Ok(self.cache_processors.last().unwrap().get_most_recent_output())
    }
    
    /// Returns the most recent output from the final processor in the chain.
    ///
    /// This provides access to the current state of the processing pipeline without
    /// triggering new processing. Useful for reading the current processed value.
    ///
    /// # Returns
    /// Reference to the output data from the last processor in the chain.
    pub fn get_most_recent_output(&self) -> &WrappedIOData {
        self.cache_processors.last().unwrap().get_most_recent_output()
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
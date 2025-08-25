//! Core trait definition for stream cache processing.
//!
//! This module defines the `StreamCacheStage` trait, which provides a unified interface
//! for all data pipeline processing in the FEAGI system. Processors implementing this
//! trait can be chained together to create complex data processing pipelines.

use std::fmt;
use std::fmt::Debug;
use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub trait StreamCacheStage: fmt::Display + Debug + Sync + Send {
    /// Returns the data type this processor expects as input.
    ///
    /// This is used by `ProcessorRunner` to validate that processing can be chained
    /// together correctly (output type of one matches input type of the next).
    fn get_input_data_type(&self) -> WrappedIOType;

    /// Returns the data type this processor produces as output.
    ///
    /// This is used by `ProcessorRunner` to validate processor chain compatibility
    /// and determine the final output type of processing pipeline.
    fn get_output_data_type(&self) -> WrappedIOType;
    
    /// Returns a reference to the most recently processed output value from internal cached memory.
    fn get_most_recent_output(&self) -> &WrappedIOData;

    /// Process new input data and return a reference to the transformed output.
    ///
    /// This is the core processing method where the actual data transformation occurs.
    /// Implementations should:
    /// - Transform the input value according to their specific logic
    /// - Update internal state with the new output
    /// - Return a reference to the newly computed output
    ///
    /// # Arguments
    /// * `value` - The input data to process (type should match `get_input_data_type()`)
    /// * `time_of_input` - Timestamp when the input was received (for time-aware processing)
    ///
    /// # Returns
    /// * `Ok(&WrappedIOData)` - Reference to the processed output data
    /// * `Err(FeagiDataError)` - If processing fails due to invalid input or internal errors
    ///
    /// # Note
    /// Type checking is not performed here - it's the responsibility of `ProcessorRunner`
    /// to ensure input types are compatible before calling this method.
    fn process_new_input(&mut self, value: &WrappedIOData, time_of_input: Instant) -> Result<&WrappedIOData, FeagiDataError>;
    
}

// TODO JSON descriptors and parameter updates
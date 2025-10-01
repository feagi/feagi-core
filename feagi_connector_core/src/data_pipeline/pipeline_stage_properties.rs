use std::any::Any;
use std::fmt;
use std::fmt::Debug;
use feagi_data_structures::wrapped_io_data::WrappedIOType;
use crate::data_pipeline::PipelineStage;

/// Used by PipelineStage implemented structs to copy out and in only the configuration parameters without needing to copy the data
pub trait PipelineStageProperties: fmt::Display + Debug + Sync + Send + Any {

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

    /// Clones this struct in a box
    fn clone_box(&self) -> Box<dyn PipelineStageProperties>;

    /// Provide access to `Any` trait for downcasting
    fn as_any(&self) -> &dyn Any;

    fn create_stage(&self) -> Box<dyn PipelineStage>;
}
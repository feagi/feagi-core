use std::any::Any;
use std::fmt;
use std::fmt::Debug;

/// Used by PipelineStage implemented structs to copy out and in only the configuration parameters without needing to copy the data
pub trait PipelineStageProperties: fmt::Display + Debug + Sync + Send + Any {

    // Clones this struct in a box
    fn clone_box(&self) -> Box<dyn PipelineStageProperties>;

    /// Provide access to `Any` trait for downcasting
    fn as_any(&self) -> &dyn Any;
}
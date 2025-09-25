use std::any::Any;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;

/// Identity Stages have no parameters, so this structure is essentially blank
#[derive(Debug)]
pub struct IdentityStageProperties;

impl PipelineStageProperties for IdentityStageProperties {
    fn clone_box(&self) -> Box<dyn PipelineStageProperties> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl IdentityStageProperties {
    pub fn new() -> IdentityStageProperties {
        IdentityStageProperties
    }
}

impl std::fmt::Display for IdentityStageProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "IdentityStage()")
    }
}
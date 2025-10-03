use std::any::Any;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;
use crate::data_pipeline::PipelineStage;
use crate::data_pipeline::stages::IdentityStage;
use crate::wrapped_io_data::WrappedIOType;

/// Identity Stages have no parameters, so this structure is essentially blank
#[derive(Debug, Clone)]
pub struct IdentityStageProperties {
    identity_type: WrappedIOType
}

impl PipelineStageProperties for IdentityStageProperties {
    fn get_input_data_type(&self) -> WrappedIOType {
        self.identity_type
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        self.identity_type
    }

    fn clone_box(&self) -> Box<dyn PipelineStageProperties> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_stage(&self) -> Box<dyn PipelineStage> {
        IdentityStage::new_box(self.identity_type).unwrap()
    }
}

impl IdentityStageProperties {
    pub fn new(identity_type: WrappedIOType) -> IdentityStageProperties { // TODO this should be a result, as it can fail
        IdentityStageProperties {
            identity_type
        }
    }
    
    pub fn new_box(identity_type: WrappedIOType) -> Result<Box<dyn PipelineStageProperties + Send + Sync + 'static>, FeagiDataError> {
        Ok(Box::new(IdentityStageProperties::new(identity_type)))
    }
}

impl std::fmt::Display for IdentityStageProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "IdentityStage({})", self.identity_type)
    }
}
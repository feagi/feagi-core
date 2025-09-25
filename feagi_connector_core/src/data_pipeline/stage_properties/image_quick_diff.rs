use std::any::Any;
use std::ops::RangeInclusive;
use feagi_data_structures::data::Percentage;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;

/// Parameter Properties for [ImageQuickDiffStage]
#[derive(Debug)]
pub struct ImageQuickDiffStageProperties {
    pub per_pixel_allowed_range: RangeInclusive<u8>, 
    pub acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>
}

impl PipelineStageProperties for ImageQuickDiffStageProperties {
    fn clone_box(&self) -> Box<dyn PipelineStageProperties> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ImageQuickDiffStageProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "IdentityStage()")
    }
}
use serde::{Deserialize, Serialize};
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelIndex, CorticalUnitIndex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedBackRegistration {
    SegmentedVisionWithGaze { gaze_unit_index: CorticalUnitIndex, gaze_channel_index: CorticalChannelIndex, segmentation_unit_index: CorticalUnitIndex, segmentation_channel_index: CorticalChannelIndex}
}


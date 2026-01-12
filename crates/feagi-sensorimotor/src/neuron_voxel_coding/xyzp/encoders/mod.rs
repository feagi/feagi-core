// TODO there is a bug on all encoders in that they will send all channel data even if not updated since scratch is cleared only if channel is active!
mod boolean;
mod cartesian_plane;
mod misc_data;
mod percentage_encoder;
mod segmented_image_frame;

#[allow(unused_imports)]
pub(crate) use boolean::BooleanNeuronVoxelXYZPEncoder;
#[allow(unused_imports)]
pub(crate) use cartesian_plane::CartesianPlaneNeuronVoxelXYZPEncoder;
#[allow(unused_imports)]
pub(crate) use misc_data::MiscDataNeuronVoxelXYZPEncoder;
#[allow(unused_imports)]
pub(crate) use segmented_image_frame::SegmentedImageFrameNeuronVoxelXYZPEncoder;

// Percentage encoder (uses PercentageNeuronPositioning and PercentageChannelDimensionality from other modules)
#[allow(unused_imports)]
pub(crate) use percentage_encoder::PercentageNeuronVoxelXYZPEncoder;

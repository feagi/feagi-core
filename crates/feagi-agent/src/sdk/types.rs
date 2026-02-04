//! Re-exports of controller-facing types from feagi_structures and feagi_sensorimotor.

pub use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalSubUnitIndex, CorticalUnitIndex,
    NeuronDepth,
};
pub use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
};
pub use feagi_structures::genomic::cortical_area::CorticalID;
pub use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
pub use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

#[cfg(feature = "sdk-io")]
pub use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, ImageXYResolution,
    SegmentedImageFrameProperties,
};
#[cfg(feature = "sdk-io")]
pub use feagi_sensorimotor::data_types::descriptors::MiscDataDimensions;
pub use feagi_sensorimotor::data_types::{GazeProperties, ImageFrame, MiscData};
#[cfg(feature = "sdk-io")]
pub use feagi_sensorimotor::caching::SensorDeviceCache;
#[cfg(feature = "sdk-io")]
pub use feagi_sensorimotor::feedbacks::{FeedBackRegistration, FeedbackRegistrationTargets};
#[cfg(feature = "sdk-io")]
pub use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
#[cfg(feature = "sdk-text")]
pub use feagi_sensorimotor::data_types::encode_token_id_to_misc_data;

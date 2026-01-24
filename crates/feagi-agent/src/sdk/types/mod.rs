//! SDK type re-exports for controller-facing APIs.
//!
//! This module centralizes commonly used FEAGI data types to keep controller
//! code stable while the underlying crates evolve.

pub use feagi_sensorimotor::caching::{MotorDeviceCache, SensorDeviceCache};
pub use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, ImageXYResolution, MiscDataDimensions,
    SegmentedImageFrameProperties,
};
pub use feagi_sensorimotor::data_types::{
    encode_token_id_to_misc_data, GazeProperties, ImageFrame, MiscData,
};
pub use feagi_sensorimotor::feedbacks::{FeedBackRegistration, FeedbackRegistrationTargets};
pub use feagi_sensorimotor::wrapped_io_data::{WrappedIOData, WrappedIOType};

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

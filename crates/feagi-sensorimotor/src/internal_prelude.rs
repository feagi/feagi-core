//! Common types used throughout this crate. Files that lost glob imports from
//! deleted crates can pull this in as `use crate::internal_prelude::*;`.
#![allow(unused_imports)]

pub(crate) use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
pub(crate) use crate::data_types::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, CorticalChannelIndex, ImageFrameProperties,
    MiscDataDimensions, NeuronDepth, SegmentedImageFrameProperties,
};
pub(crate) use crate::data_types::Percentage3D;
pub(crate) use crate::feagi_signal::{FeagiSignal, FeagiSignalIndex};
pub(crate) use crate::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
    NeuronVoxelXYZPSparseVectors,
};
pub(crate) use feagi_data::feagi_data_error::FeagiDataError;
pub(crate) use feagi_genomic_context::cortical_area::CorticalID;
pub(crate) use feagi_genomic_context::cortical_unit::motor_cortical_unit::MotorCorticalUnit;
pub(crate) use feagi_genomic_context::cortical_unit::sensor_cortical_unit::SensoryCorticalUnit;
pub(crate) use feagi_genomic_context::cortical_unit::CorticalUnitIndex;

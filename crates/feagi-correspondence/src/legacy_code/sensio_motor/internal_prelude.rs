//! Common types used throughout this crate. Files that lost glob imports from
//! deleted crates can pull this in as `use crate::internal_prelude::*;`.
#![allow(unused_imports)]

pub(crate) use crate::legacy_code::sensio_motor::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
pub(crate) use crate::legacy_code::sensio_motor::data_types::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, CorticalChannelIndex, ImageFrameProperties,
    MiscDataDimensions, NeuronDepth, SegmentedImageFrameProperties,
};
pub(crate) use crate::legacy_code::sensio_motor::data_types::Percentage3D;
pub(crate) use crate::legacy_code::sensio_motor::feagi_signal::{FeagiSignal, FeagiSignalIndex};
pub(crate) use crate::legacy_code::sensio_motor::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
    NeuronVoxelXYZPSparseVectors,
};
pub(crate) use feagi_basis::FeagiBasisError;
pub(crate) use feagi_basis::feagi_genome::identifiers::cortical_id::CorticalID;

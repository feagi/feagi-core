use crate::collections::quant_indexed::quant_indexed::QuantContiguousStore;
use crate::collections::quant_indexed::spatial::SpatialQuantIndexedOwning;
use crate::collections::spatial_helpers::axis_order::AxisOrderIncrementing;
use crate::prelude::{QuantizedDecimalUnwrappedTrait, QuantizedUnsignedIntegerUnwrappedTrait, SpatialCoordinate, SpatialDimensions};
use crate::{create_wrapped_quantized_decimal, create_wrapped_quantized_unsigned_integer};
use serde::de::DeserializeOwned;

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of the neuron(s) in a voxel. Most of the time, each
    /// voxel contains a single neuron, but in cases where there are more, they are averaged to
    /// make this
    pub CorticalAreaVoxelPotential
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a voxel in a collection using a single uint value that represents
    /// the overall index incrementing from X, Y and Z
    pub CorticalAreaVoxelLinearIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of voxels within a dimensional cortical area
    pub CorticalAreaVoxelCount
);

create_wrapped_quantized_unsigned_integer!(
    /// A generic count of neurons
    pub VoxelCount
);

// No linear collection needed, right?

//region Dimensional Collections

create_wrapped_quantized_unsigned_integer!(
    /// Index of a voxel along one of the XYZ directions within a cortical area
    pub CorticalAreaVoxelCoordinateAxisPosition
);

/// Defines the position of a voxel in a voxel cortical area
pub type CorticalAreaVoxelCoordinates<QI: QuantizedUnsignedIntegerUnwrappedTrait> = SpatialCoordinate<CorticalAreaVoxelCoordinateAxisPosition<QI>, 3>;

/// Defines the voxel dimensions of a voxel cortical area
pub type CorticalAreaVoxelDimensions<QI: QuantizedUnsignedIntegerUnwrappedTrait> = SpatialDimensions<CorticalAreaVoxelCoordinateAxisPosition<QI>, 3>;

/// A collection of Cortical Area Voxels that owns all its inner fields, using default incrementing
/// indexing
pub type CorticalAreaVoxels<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait,
    S: QuantContiguousStore + DeserializeOwned + 'static,
> = SpatialQuantIndexedOwning<
    CorticalAreaVoxelLinearIndex<QI>,
    CorticalAreaVoxelCoordinates<QI>,
    CorticalAreaVoxelDimensions<QI>,
    AxisOrderIncrementing<3>,
    S,
    CorticalAreaVoxelPotential<QP>,
    3,
>;

//endregion

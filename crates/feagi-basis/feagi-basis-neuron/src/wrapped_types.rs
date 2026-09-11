//! Indexing / Dimensions for refering to neuron related structs in a spatial context
//!
//! Voxel -> 3D coordinate, with a Density Index for differentiating multiple neurons per voxel
//! Dimensional -> 4D coordinate, with density simply being the 4th value
//! We have different structs since voxels consolidate (lose) data from the source dimensional data
//! which NPU operates in

use feagi_basis_quantization::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait;
use feagi_basis_quantization::{
    create_wrapped_quantized_decimal, create_wrapped_quantized_unsigned_integer, create_wrapped_unsigned_integer_spatial_coordinate,
    create_wrapped_unsigned_integer_spatial_dimensions,
};
//region Values

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of a single neuron
    pub CorticalAreaNeuronPotential
);

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of the neuron(s) in a voxel. Most of the time, each
    /// voxel contains a single neuron, but in cases where there are more, they are averaged to
    /// make this
    pub CorticalAreaVoxelPotential
);

//endregion

//region Linear Indexing

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron relative to its parent cortical area
    pub CorticalAreaNeuronLocalIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a voxel in a collection using a single uint value that represents
    /// the overall index incrementing from X, Y and Z
    pub CorticalAreaVoxelLinearIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a voxel in a cortical interface channel using a single uint value
    /// that represents the overall index incrementing from X, Y and Z
    pub CorticalChannelVoxelLinearIndex
);

//endregion

//region Count

create_wrapped_quantized_unsigned_integer!(
    /// The number of neurons (not voxels) within a dimensional cortical area
    pub CorticalAreaNeuronCount
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of voxels within a dimensional cortical area
    pub CorticalAreaVoxelCount
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of voxels within a channel of a cortical interface area
    pub CorticalChannelVoxelCount
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of neurons within a voxel (normally 1)
    pub NeuronVoxelDensity
);

create_wrapped_quantized_unsigned_integer!(
    /// Defines a number of neurons (generically, for any context)
    pub NeuronCount
);

impl<Q: QuantizedUnsignedIntegerUnwrappedTrait> Into<NeuronCount<Q>> for CorticalAreaNeuronCount<Q> {
    fn into(self) -> NeuronCount<Q> {
        NeuronCount::new(self.0)
    }
}

create_wrapped_quantized_unsigned_integer!(
    /// Defines a number of voxels (generically, for any context)
    pub VoxelCount
);

impl<Q: QuantizedUnsignedIntegerUnwrappedTrait> Into<VoxelCount<Q>> for CorticalAreaVoxelCount<Q> {
    fn into(self) -> VoxelCount<Q> {
        VoxelCount::new(self.0)
    }
}

impl<Q: QuantizedUnsignedIntegerUnwrappedTrait> Into<VoxelCount<Q>> for CorticalChannelVoxelCount<Q> {
    fn into(self) -> VoxelCount<Q> {
        VoxelCount::new(self.0)
    }
}

//endregion

//region Spatial

//region Spatial Index

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron along one of the XYZD directions within a cortical area
    pub CorticalAreaCoordinateAxisIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a voxel along one of the XYZ directions within a cortical area
    pub CorticalAreaVoxelCoordinateAxisIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a voxel along one of the XYZ directions within a channel of a cortical interface area
    pub CorticalChannelVoxelCoordinateAxisIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron within a voxel. Most voxels only have 1 neuron, but some have more
    pub NeuronDensityIndex
);

//endregion

//region Coordinate

create_wrapped_unsigned_integer_spatial_coordinate!(
        /// Represents a 4D coordinate of a neuron within a dimensional cortical_area area, with the
        /// 4th dimension being the density index
        pub CorticalAreaNeuronCoordinate,
        4,
        (0, x, CorticalAreaCoordinateAxisIndex), (1, y, CorticalAreaCoordinateAxisIndex), (2, z, CorticalAreaCoordinateAxisIndex), (3, d, NeuronDensityIndex)
);

create_wrapped_unsigned_integer_spatial_coordinate!(
    /// Represents a 3D coordinate of a voxel within a dimensional cortical area
    pub CorticalAreaVoxelCoordinate,
    3,
    (0, x, CorticalAreaVoxelCoordinateAxisIndex), (1, y, CorticalAreaVoxelCoordinateAxisIndex), (2, z, CorticalAreaVoxelCoordinateAxisIndex)
);

create_wrapped_unsigned_integer_spatial_coordinate!(
    /// Represents a 3D coordinate of a voxel within a channel of a cortical interface
    pub CorticalChannelVoxelCoordinate,
    3,
    (0, x, CorticalChannelVoxelCoordinateAxisIndex), (1, y, CorticalChannelVoxelCoordinateAxisIndex), (2, z, CorticalChannelVoxelCoordinateAxisIndex)
);

//endregion

//region Dimensions

create_wrapped_unsigned_integer_spatial_dimensions!(
    /// Represents the dimensions of the neurons in a dimensional neuron area
    pub CorticalAreaNeuronDimensions,
    CorticalAreaNeuronCoordinate,
    CorticalAreaNeuronLocalIndex,
    CorticalAreaNeuronCount,
    4,
    (0, x, CorticalAreaCoordinateAxisIndex), (1, y, CorticalAreaCoordinateAxisIndex), (2, z, CorticalAreaCoordinateAxisIndex), (3, d, NeuronVoxelDensity)
);

create_wrapped_unsigned_integer_spatial_dimensions!(
    /// Represents the dimensions of the voxels of a cortical area
    pub CorticalAreaVoxelDimensions,
    CorticalAreaVoxelCoordinate,
    CorticalAreaVoxelLinearIndex,
    CorticalAreaVoxelCount,
    3,
    (0, x, CorticalAreaCoordinateAxisIndex), (1, y, CorticalAreaCoordinateAxisIndex), (2, z, CorticalAreaCoordinateAxisIndex),
);

create_wrapped_unsigned_integer_spatial_dimensions!(
    /// Represents the dimensions of the voxels within a channel of a cortical area
    pub CorticalChannelVoxelDimensions,
    CorticalChannelVoxelCoordinate,
    CorticalChannelVoxelLinearIndex,
    CorticalChannelVoxelCount,
    3,
    (0, x, CorticalChannelVoxelCoordinateAxisIndex), (1, y, CorticalChannelVoxelCoordinateAxisIndex), (2, z, CorticalChannelVoxelCoordinateAxisIndex),
);

//endregion

//endregion

// NOTE: BitBatch indexing refers to multiple neurons, so they make little sense for things like spatial indexing

//region BitBatch

create_wrapped_quantized_unsigned_integer!(
    /// Index of the uint storing the bitpacked information of neuron activations
    pub NeuronActivationBitBatchIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Defines the number of BitBatched values encoding neuron activations. Likely has padding
    pub NeuronActivationBitBatchCount
);

//endregion

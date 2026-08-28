//! Indexing / Dimensions for refering to neuron related structs in a spatial context
//! 
//! Voxel -> 3D coordinate, with a Density Index for differentiating multiple neurons per voxel
//! Dimensional -> 4D coordinate, with density simply being the 4th value
//! We have different structs since voxels consolidate (lose) data from the source dimensional data 
//! which NPU operates in

use crate::{create_wrapped_quantized_decimal, create_wrapped_quantized_unsigned_integer, create_wrapped_unsigned_integer_spatial_coordinate, create_wrapped_unsigned_integer_spatial_dimensions};


create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of the neuron(s) in a voxel. Most of the time, each
    /// voxel contains a single neuron, but in cases where there are more, they are averaged to
    /// make this
    pub CorticalNeuronPotential
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron relative to its parent cortical area
    pub CorticalNeuronLocalIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Defines a number of neurons
    pub NeuronCount
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of the uint storing the bitpacked information of neuron activations
    pub NeuronActivationBitBatchIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Defines the number of BitBatched values encoding neuron activations. Likely has padding
    pub NeuronActivationBitBatchCount
);

//region Spatial

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron OR voxel along one of the XYZ directions
    pub CorticalCoordinateAxisIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron within a voxel. Most voxels only have 1 neuron, but some have more
    pub NeuronDensityIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of neurons within a voxel (normally 1)
    pub PerVoxelNeuronCount
);

//region Dimensional


create_wrapped_quantized_unsigned_integer!(
    /// The number of neurons (not voxels) within a dimensional cortical area
    pub CorticalAreaNeuronCount
);

create_wrapped_unsigned_integer_spatial_coordinate!(
        /// Represents a 4D coordinate of a neuron within a dimensional cortical_area area, with the
        /// 4th dimension being the density index
        pub CorticalNeuronCoordinate,
        4,
        (0, x, CorticalCoordinateAxisIndex), (1, y, CorticalCoordinateAxisIndex), (2, z, CorticalCoordinateAxisIndex), (3, d, NeuronDensityIndex)
);

create_wrapped_unsigned_integer_spatial_dimensions!(
    /// Represents the dimensions of the neurons in a dimensional neuron area
    pub CorticalNeuronDimensions,
    CorticalNeuronCoordinate,
    CorticalNeuronLocalIndex,
    CorticalAreaNeuronCount,
    4,
    (0, x, CorticalCoordinateAxisIndex), (1, y, CorticalCoordinateAxisIndex), (2, z, CorticalCoordinateAxisIndex), (3, d, NeuronDensityIndex)
);


// No aliases for dimensionality, since this is mainly for NPU anyways

//endregion


//region Voxel

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of the neuron(s) in a voxel. Most of the time, each
    /// voxel contains a single neuron, but in cases where there are more, they are averaged to
    /// make this
    pub CorticalVoxelPotential
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a voxel in a collection using a single uint value that represents
    /// the overall index incrementing from X, Y and Z
    pub CorticalVoxelLinearIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of voxels within a dimensional cortical area
    pub CorticalAreaVoxelCount
);

create_wrapped_unsigned_integer_spatial_coordinate!(
        /// Represents a 3D coordinate of a voxel within a dimensional cortical area
        pub CorticalVoxelCoordinate,
        3,
        (0, x, CorticalCoordinateAxisIndex), (1, y, CorticalCoordinateAxisIndex), (2, z, CorticalCoordinateAxisIndex)
);

create_wrapped_unsigned_integer_spatial_dimensions!(
    /// Represents the dimensions of the voxels of a cortical area
    pub CorticalVoxelDimensions,
    CorticalVoxelCoordinate,
    CorticalVoxelLinearIndex,
    CorticalAreaVoxelCount,
    3,
    (0, x, CorticalCoordinateAxisIndex), (1, y, CorticalCoordinateAxisIndex), (2, z, CorticalCoordinateAxisIndex),
);

// TODO temp

pub type CorticalVoxelLinearIndexGenomic = CorticalVoxelLinearIndex<u32>;
pub type CorticalVoxelCoordinateGenomic = CorticalVoxelCoordinate<u32>;
pub type CorticalVoxelDimensionsGenomic = CorticalVoxelDimensions<u32>;

//endregion

//endregion











use crate::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use crate::{create_wrapped_quantized_decimal, create_wrapped_quantized_unsigned_integer, create_wrapped_unsigned_integer_spatial_coordinate, create_wrapped_unsigned_integer_spatial_dimensions};
use serde::{Deserialize, Serialize};

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of the neuron(s) in a voxel. Most of the time, each
    /// voxel contains a single neuron, but in cases where there are more, they are averaged to
    /// make this
    pub NeuronVoxelPotential
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron within a voxel. Most voxels only have 1 neuron, but some have more
    pub NeuronVoxelDensityIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of neurons within a voxel (normally 1)
    pub PerVoxelNeuronCount
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron within a voxel. Most voxels only have 1 neuron, but some have more
    pub NeuronVoxelCoordinateAxis
);


create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a voxel in a collection using a single uint value that represents
    /// the overall index incrementing from X, Y and Z
    pub NeuronVoxelLinearIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of voxels within a dimensional cortical area
    pub CorticalAreaVoxelCount
);

create_wrapped_unsigned_integer_spatial_coordinate!(
        /// Represents a 4D coordinate of a neuron within a dimensional cortical_area area, with the
        /// 4th dimension being the density index
        pub NeuronVoxelCoordinate,
        3,
        (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis)
);

create_wrapped_unsigned_integer_spatial_dimensions!(
    /// Represents the dimensions and density of a dimensional cortical_area area
    pub NeuronVoxelDimensions,
    NeuronVoxelCoordinate,
    NeuronVoxelLinearIndex,
    CorticalAreaVoxelCount,
    3,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis),
);


pub type NeuronVoxelLinearIndexGenomic = NeuronVoxelLinearIndex<<FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>;
pub type NeuronVoxelCoordinateGenomic = NeuronVoxelCoordinate<<FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>;
pub type NeuronVoxelDimensionsGenomic = NeuronVoxelDimensions<<FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>;

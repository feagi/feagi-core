use crate::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use crate::values::spatial::unsigned_integer::{QuantizedIndexCoord3D, QuantizedIndexDimension3D};
use crate::{
    create_wrapped_quantized_decimal, create_wrapped_quantized_index, create_wrapped_quantized_index_coordinate,
    create_wrapped_quantized_index_dimension,
};
use serde::{Deserialize, Serialize};

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of the neuron(s) in a voxel. Most of the time, each
    /// voxel contains a single neuron, but in cases where there are more, they are averaged to
    /// make this
    pub NeuronVoxelPotential);

create_wrapped_quantized_index!(
    /// Represents the index of a voxel in a collection using a single uint value that represents
    /// the overall index incrementing from X, Y and Z
    pub NeuronVoxelLinearIndex
);

create_wrapped_quantized_index!(
    /// Represents the uint value of a single axis of a coordinate or dimension
    pub NeuronVoxelCoordinateAxis
);

create_wrapped_quantized_index_coordinate!(
    /// Represents a 3D coordinate of a specific neuron voxel
    pub NeuronVoxelCoordinate,
    QuantizedIndexCoord3D,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis)
);

create_wrapped_quantized_index_dimension!(
    /// Represents the dimensions of rectangular prism of neuron voxels
    pub NeuronVoxelDimensions,
    QuantizedIndexDimension3D,
    NeuronVoxelCoordinate,
    NeuronVoxelLinearIndex,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis)
);

pub type NeuronVoxelLinearIndexGenomic = NeuronVoxelLinearIndex<<FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>;
pub type NeuronVoxelCoordinateGenomic = NeuronVoxelCoordinate<<FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>;
pub type NeuronVoxelDimensionsGenomic = NeuronVoxelDimensions<<FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>;

use feagi_data::{create_wrapped_quantized_decimal, create_wrapped_quantized_index, create_wrapped_quantized_index_coordinate, create_wrapped_quantized_index_dimension};
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinateAxis;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;
use feagi_data::values::spatial::quantizable_index::{QuantizedIndexCoord4D, QuantizedIndexDimension4D};

create_wrapped_quantized_index!(
    /// The current burst index of a given engine
    pub BurstIndex
);

impl<Q: QuantizedIndexCountTrait> BurstIndex<Q> {
    pub fn new_from_middle() -> Self {
        Self(Q::QUANT_MAX / (Q::QUANT_ONE + Q::QUANT_ONE))
    }
}


create_wrapped_quantized_decimal!(
    /// The membrane potential of a single neuron
    pub NeuronMembranePotential
);

create_wrapped_quantized_index!(
    /// Index of a neuron relative to the burst engine
    pub NeuronEngineIndex
);

create_wrapped_quantized_index!(
    /// Index of a neuron relative to all neurons of that neuron model and quantization within the
    /// burst engine
    pub NeuronQuantizedModelIndex
);

create_wrapped_quantized_index!(
    /// Index of a neuron relative to its parent cortical area
    pub NeuronCorticalIndex
);


create_wrapped_quantized_index!(
    /// Index of a cortical area relative to the burst engine
    pub CorticalEngineIndex
);

create_wrapped_quantized_index!(
    /// Index of a cortical area relative to all cortical areas of that neuron model and
    /// quantization within the burst engine
    pub CorticalQuantizedModelIndex
);



create_wrapped_quantized_index!(
    /// Index of a neuron within a voxel. Most voxels only have 1 neuron, but some have more
    pub NeuronVoxelDensityIndex
);

create_wrapped_quantized_index_coordinate!(
    /// Represents a 4D coordinate of a neuron within a dimensional cortical area, with the
    /// 4th dimension being the density index
    pub DimensionCorticalAreaCoordinate,
    QuantizedIndexCoord4D,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis), (3, d, NeuronVoxelDensityIndex)
);

create_wrapped_quantized_index_dimension!(
    /// Represents the dimensions and density of a dimensional cortical area
    pub DimensionalCorticalAreaDimensions,
    QuantizedIndexDimension4D,
    DimensionCorticalAreaCoordinate,
    NeuronCorticalIndex,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis), (3, d, NeuronVoxelDensityIndex)
);
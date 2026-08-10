use crate::neuron_voxels::wrapped_values::{NeuronVoxelCoordinate, NeuronVoxelCoordinateAxis, NeuronVoxelDimensions};
use crate::values::quantizable::QuantizedIndexCountTrait;
use crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError;
use crate::values::spatial::quantizable_index::{QuantizedIndexCoord4D, QuantizedIndexDimension4D};
use crate::{
    create_wrapped_quantized_decimal, create_wrapped_quantized_index, create_wrapped_quantized_index_coordinate,
    create_wrapped_quantized_index_dimension,
};

create_wrapped_quantized_decimal!(
    /// The membrane potential of a single neuron (NOT VOXEL)
    pub NeuronMembranePotential
);

create_wrapped_quantized_index!(
    /// Index of a neuron within a voxel. Most voxels only have 1 neuron, but some have more
    pub NeuronVoxelDensityIndex
);

create_wrapped_quantized_index!(
    /// Index of a neuron relative to its parent cortical area
    pub NeuronCorticalLocalIndex
);

create_wrapped_quantized_index_coordinate!(
    /// Represents a 4D coordinate of a neuron within a dimensional cortical_area area, with the
    /// 4th dimension being the density index
    pub DimensionalCorticalArea4DCoordinate,
    QuantizedIndexCoord4D,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis), (3, d, NeuronVoxelDensityIndex)
);

impl<Q: QuantizedIndexCountTrait> DimensionalCorticalArea4DCoordinate<Q> {
    pub fn new_from_voxel_and_density(voxel_coord: NeuronVoxelCoordinate<Q>, density: NeuronVoxelDensityIndex<Q>) -> Self {
        DimensionalCorticalArea4DCoordinate::new(*voxel_coord.get_x(), *voxel_coord.get_y(), *voxel_coord.get_z(), density)
    }
}

create_wrapped_quantized_index_dimension!(
    /// Represents the dimensions and density of a dimensional cortical_area area
    pub DimensionalCorticalArea4DDimensions,
    QuantizedIndexDimension4D,
    DimensionalCorticalArea4DCoordinate,
    NeuronCorticalLocalIndex,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis), (3, d, NeuronVoxelDensityIndex)
);

impl<Q: QuantizedIndexCountTrait> DimensionalCorticalArea4DDimensions<Q> {
    pub fn try_new_from_voxel_and_density(
        voxel_dim: NeuronVoxelDimensions<Q>,
        density: NeuronVoxelDensityIndex<Q>,
    ) -> Result<Self, FeagiDataValuesSpatialError> {
        DimensionalCorticalArea4DDimensions::try_new(*voxel_dim.get_x(), *voxel_dim.get_y(), *voxel_dim.get_z(), density)
    }
}

use crate::{create_wrapped_quantized_unsigned_integer, create_wrapped_unsigned_integer_spatial_coordinate, create_wrapped_unsigned_integer_spatial_dimensions};
use crate::{create_wrapped_quantized_decimal};
use crate::neurons::neuron_voxels::wrapped_values::{NeuronVoxelCoordinate, NeuronVoxelCoordinateAxis, NeuronVoxelDensityIndex};
use crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait;





/*


create_wrapped_unsigned_integer_spatial_coordinate!(
        /// Represents a 4D coordinate of a neuron within a dimensional cortical_area area, with the
        /// 4th dimension being the density index
        pub DimensionalCorticalArea4DCoordinate,
        4,
        (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis), (3, d, NeuronVoxelDensityIndex)
);


impl<Q: QuantizedUnsignedIntegerUnwrappedTrait> DimensionalCorticalArea4DCoordinate<Q> {
    pub fn new_from_voxel_and_density(voxel_coord: NeuronVoxelCoordinate<Q>, density: NeuronVoxelDensityIndex<Q>) -> Self {
        DimensionalCorticalArea4DCoordinate::new(*voxel_coord.get_x(), *voxel_coord.get_y(), *voxel_coord.get_z(), density)
    }
}

create_wrapped_unsigned_integer_spatial_dimensions!(
    /// Represents the dimensions and density of a dimensional cortical_area area
    pub DimensionalCorticalArea4DDimensions,
    DimensionalCorticalArea4DCoordinate,
    NeuronCorticalLocalIndex,
    CorticalAreaNeuronCount,
    4,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis), (3, d, NeuronVoxelDensityIndex)
);


 */
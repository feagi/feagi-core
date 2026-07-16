use crate::feagi_data_error::FeagiDataError;
use crate::neuron_voxels::neuron_voxel_error::{FeagiVoxelError, FeagiVoxelsInvalidDimensions};
use crate::feagi_quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::values::quantizable::QuantizedIndexCountTrait;
use crate::values::spatial::quantizable_index::{QuantizedIndexCoord3D, QuantizedIndexDimension3D};
use crate::{create_spatial_bitpacked_vector, create_wrapped_quantized_decimal, create_wrapped_quantized_index, create_wrapped_quantized_index_coordinate, create_wrapped_quantized_index_dimension};

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

impl<Q: QuantizedIndexCountTrait> NeuronVoxelCoordinate<Q> {
    pub fn new_from_usize(x: usize, y: usize, z: usize) -> Result<Self, FeagiDataError> {
        if x == 0 || y == 0 || z == 0 {
            return Err(FeagiVoxelError::InvalidDimensions(FeagiVoxelsInvalidDimensions::new(
                "Neuron Voxel Dimensions cannot have a side length of 0!",
            ))
                .into());
        }

        let x = NeuronVoxelCoordinateAxis::new(Q::quant_from_usize(x));
        let y = NeuronVoxelCoordinateAxis::new(Q::quant_from_usize(y));
        let z = NeuronVoxelCoordinateAxis::new(Q::quant_from_usize(z));
        Ok(NeuronVoxelCoordinate::new(x, y, z))
    }
}

create_wrapped_quantized_index_dimension!(
    /// Represents the dimensions of rectangular prism of neuron voxels
    pub NeuronVoxelDimensions,
    QuantizedIndexDimension3D,
    NeuronVoxelCoordinate,
    NeuronVoxelLinearIndex,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis)
);

impl<Q: QuantizedIndexCountTrait> NeuronVoxelDimensions<Q> {
    pub fn new_from_usize(x: usize, y: usize, z: usize) -> Result<Self, FeagiDataError> {
        if x == 0 || y == 0 || z == 0 {
            return Err(FeagiVoxelError::InvalidDimensions(FeagiVoxelsInvalidDimensions::new(
                "Neuron Voxel Dimensions cannot have a side length of 0!",
            ))
            .into());
        }

        let x = NeuronVoxelCoordinateAxis::new(Q::quant_from_usize(x));
        let y = NeuronVoxelCoordinateAxis::new(Q::quant_from_usize(y));
        let z = NeuronVoxelCoordinateAxis::new(Q::quant_from_usize(z));
        Ok(NeuronVoxelDimensions::new(x, y, z))
    }

    // TODO this should be part of the macro!
    pub(crate) fn temp_to_ref(self) -> QuantizedIndexDimension3D<Q> {
        self.0
    }
}

use core::fmt::{Debug, Display};
use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;

pub enum SingleCorticalNeuronVoxelCollectionType {
    DenseArray,
    DenseVector,
    IndexVector,
    CoordVector,
}


//region Neuron Voxel Coordinate

crate::define_unsigned_coordinate_3d_type_family!(NeuronVoxelCoordinate);

//endregion

//region Neuron Voxel Dimensions

crate::define_dimension_3d_type_family!(NeuronVoxelDimensions, NeuronVoxelCoordinate);

impl<CoordQuant: QuantizableUInt> NeuronVoxelDimensions<CoordQuant> {
    pub fn get_max_allowed_index_exclusive(&self) -> NeuronVoxelIndexQuant {
        self.x * self.y * self.z
    }
}

//endregion

//region Neuron Voxel Potential

crate::define_quantizable_value_type_family!(NeuronVoxelPotential);

impl<Potential: QuantizableValue> NeuronVoxelPotential<Potential> {

}

//endregion
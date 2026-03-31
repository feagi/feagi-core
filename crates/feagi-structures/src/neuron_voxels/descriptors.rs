use crate::base_quantizable::QuantizableUIntType;
use crate::base_quantizable::QuantizableValueType;
use crate::neurons::descriptors::NumberNeuronsPerVoxel;

/// Denotes how neurons are being stored
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

impl<CoordQuant: QuantizableUIntType> NeuronVoxelDimensions<CoordQuant> {
    pub fn get_max_allowed_index_exclusive(&self) -> CoordQuant { // TODO we should be using an index quantization here!!!
        self.x * self.y * self.z
    }

    pub fn get_number_neurons(&self, density: NumberNeuronsPerVoxel) -> usize {
        self.get_max_allowed_index_exclusive() as usize * (density as usize)
    }
}

//endregion

//region Neuron Voxel Potential

crate::define_quantizable_value_type_family!(NeuronVoxelPotential);

impl<Potential: QuantizableValueType> NeuronVoxelPotential<Potential> {

}

//endregion
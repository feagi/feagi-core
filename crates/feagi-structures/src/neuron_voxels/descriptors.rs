use crate::base_quantizable::{NonzeroCount, QuantizableUIntType};
use crate::base_quantizable::QuantizableValueType;
use crate::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};

/// Denotes how neurons are being stored
pub enum SingleCorticalNeuronVoxelCollectionType {
    DenseArray,
    DenseVector,
    IndexVector,
    CoordVector,
}


//region Neuron Voxel Index

crate::define_quantizable_uint_type_family!(NeuronVoxelIndex);

impl<IndexQuant: QuantizableUIntType> NeuronVoxelIndex<IndexQuant> {

}
//endregion

//region Neuron Voxel Count

crate::define_quantizable_uint_type_family!(NeuronVoxelCount);

impl<IndexQuant: QuantizableUIntType> NeuronVoxelCount<IndexQuant> {

}
//endregion

//region Neuron Voxel Coordinate

crate::define_unsigned_coordinate_3d_type_family!(NeuronVoxelCoordinate);

//endregion

//region Neuron Voxel Dimensions

crate::define_dimension_3d_type_family!(NeuronVoxelDimensions, NeuronVoxelCoordinate);

impl<CoordQuant: QuantizableUIntType> NeuronVoxelDimensions<CoordQuant> {
    pub const fn default_1_1_1_cube() -> Self {
        NeuronVoxelDimensions {
            x: NonzeroCount::ONE,
            y: NonzeroCount::ONE,
            z: NonzeroCount::ONE,
        }
    }

    pub fn get_max_allowed_index_exclusive(&self) -> usize {
        (self.x * self.y * self.z).to_usize()
    }

    pub fn get_number_voxels<IndexQuant: QuantizableUIntType>(&self) -> NeuronVoxelCount<IndexQuant> {
        NeuronVoxelCount::from_usize(self.get_max_allowed_index_exclusive())
    }

    pub fn get_number_neurons<IndexQuant: QuantizableUIntType>(&self, density: &NeuronCount<NumberNeuronsPerVoxel>) -> NeuronCount<IndexQuant> {
        NeuronCount::from_usize(self.get_max_allowed_index_exclusive().to_usize() * (density.to_usize()))
    }

    /// Linear voxel index with **x varying fastest**: `index = x + y·dx + z·dx·dy`.
    #[inline(always)]
    pub fn linear_index_to_coordinate<IndexQuant: QuantizableUIntType>(
        &self,
        index: IndexQuant,
    ) -> NeuronVoxelCoordinate<CoordQuant> {
        let i = index.to_usize();
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let plane = dx * dy;
        let z = i / plane;
        let rem = i % plane;
        let y = rem / dx;
        let x = rem % dx;
        NeuronVoxelCoordinate::new(
            CoordQuant::from_usize(x),
            CoordQuant::from_usize(y),
            CoordQuant::from_usize(z),
        )
    }

    /// Inverse of [`Self::linear_index_to_coordinate`].
    #[inline(always)]
    pub fn coordinate_to_linear_index<IndexQuant: QuantizableUIntType>(
        &self,
        coordinate: NeuronVoxelCoordinate<CoordQuant>,
    ) -> IndexQuant {
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let x = coordinate.x.to_usize();
        let y = coordinate.y.to_usize();
        let z = coordinate.z.to_usize();
        let i = x + y * dx + z * dx * dy;
        IndexQuant::from_usize(i)
    }
}

//endregion

//region Neuron Voxel Potential

crate::define_quantizable_value_type_family!(NeuronVoxelPotential);

impl<Potential: QuantizableValueType> NeuronVoxelPotential<Potential> {

}

//endregion
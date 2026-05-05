use crate::base_feagi_types::quantizable_types::{QuantizableNonzeroUIntType, QuantizableUIntType};
use crate::base_feagi_types::quantizable_types::QuantizableValueType;
use crate::neuron_collections::data_values::{NeuronCount, NeuronDensityPerVoxel};


pub trait NeuronCollectionQuantizationLevelType {
    type VoxelIndexCountCoordQuant: QuantizableUIntType;
    type VoxelPotentialQuant: QuantizableValueType;
}

/// Denotes how neuron_collections are being stored
pub enum SingleCorticalNeuronVoxelCollectionType {
    DenseArray,
    DenseVector,
    IndexVector,
    CoordVector,
}


//region Neuron Voxel Index

crate::define_quantizable_uint_type_family!(NeuronVoxelIndex);

impl<VoxelIndexCountCoordQuant: QuantizableUIntType> NeuronVoxelIndex<VoxelIndexCountCoordQuant> {

}
//endregion

//region Neuron Voxel Count

crate::define_quantizable_uint_type_family!(NeuronVoxelCount);

impl<VoxelIndexCountCoordQuant: QuantizableUIntType> NeuronVoxelCount<VoxelIndexCountCoordQuant> {

}
//endregion

//region Neuron Voxel Coordinate

crate::define_unsigned_coordinate_3d_type_family!(NeuronVoxelCoordinate);

//endregion

//region Neuron Voxel Dimensions

crate::define_dimension_3d_type_family!(NeuronVoxelDimensions, NeuronVoxelCoordinate);

impl<VoxelIndexCountCoordQuant: QuantizableUIntType> NeuronVoxelDimensions<VoxelIndexCountCoordQuant> {

    pub fn get_number_voxels(&self) -> NeuronVoxelCount<VoxelIndexCountCoordQuant> {
        NeuronVoxelCount::from_usize(self.number_elements())
    }

    pub fn get_number_neurons(&self, density: &NeuronDensityPerVoxel) -> NeuronCount<VoxelIndexCountCoordQuant> {
        NeuronVoxelDimensions::get_number_neurons(self, density)
    }

    /// Linear voxel index with **x varying fastest**: `index = x + y·dx + z·dx·dy`.
    #[inline(always)]
    pub fn linear_index_to_coordinate(
        &self,
        index: VoxelIndexCountCoordQuant,
    ) -> NeuronVoxelCoordinate<VoxelIndexCountCoordQuant> {
        let i = index.to_usize();
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let plane = dx * dy;
        let z = i / plane;
        let rem = i % plane;
        let y = rem / dx;
        let x = rem % dx;
        NeuronVoxelCoordinate::new(
            VoxelIndexCountCoordQuant::from_usize(x),
            VoxelIndexCountCoordQuant::from_usize(y),
            VoxelIndexCountCoordQuant::from_usize(z),
        )
    }

    /// Inverse of [`Self::linear_index_to_coordinate`].
    #[inline(always)]
    pub fn coordinate_to_linear_index(
        &self,
        coordinate: NeuronVoxelCoordinate<VoxelIndexCountCoordQuant>,
    ) -> VoxelIndexCountCoordQuant {
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let x = coordinate.x.to_usize();
        let y = coordinate.y.to_usize();
        let z = coordinate.z.to_usize();
        let i = x + y * dx + z * dx * dy;
        VoxelIndexCountCoordQuant::from_usize(i)
    }

    // TODO iterators
}

//endregion

//region Neuron Voxel Potential

crate::define_quantizable_value_type_family!(NeuronVoxelPotential);

impl<VoxelPotentialQuant: QuantizableValueType> NeuronVoxelPotential<VoxelPotentialQuant> {

}

//endregion
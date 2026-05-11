/// Common Neuron related structs used in various collection types and in other areas

/// Describes how a neuron collection is storing its data
pub enum NeuronCollectionType {
    DenseArray,
    DenseVector,
    IndexVector,
}

/// Describes what method a collection is using to store potential data. Mainly matters when neuron
/// density != 1
pub enum NeuronPotentialType {
    /// Potential is stored per individual neuron, particularly relevant for NPU
    IndividualNeuron,
    /// Potential is stored for a given voxel of neuron(s)
    Voxel
}

/// Describes what method a voxel's potential is calculated if it has multiple neurons
pub enum NeuronVoxelMultiPotentialCalculationMethod {
    Sum,
    Average,
    Max
}

/// Neuron Potential of neuron_collections (not voxels!)
//region Individual Neuron Membrane Potential

crate::define_quantizable_value_type_family!(IndividualNeuronMembranePotential);

//endregion

//region Neuron Index and Count

crate::define_quantizable_uint_type_family!(IndividualNeuronIndexCount);

//endregion

//region Neuron Density Per Voxel

pub use all_neuron_densities::NeuronDensityPerVoxel;
use crate::base_feagi_types::quantizable_types::{FeagiBaseQuantizationType, FeagiBaseSingleElementQuantizationType, QuantizableNonzeroUIntType, QuantizableUIntType, QuantizableValueType};

// We do this since we only want to expose the u8 level
mod all_neuron_densities {

    /// The number of neuron_collections that a single voxel represents. In most contexts this will be 1,
    /// but sometimes may be more, though never high, hence being locked to a u8. Cannot be 0
    pub type NeuronDensityPerVoxel = NeuronDensityPerVoxelAll<u8>;

    /// This defines multiple quantizations, which we dont care for
    crate::define_nonzero_count_family!(NeuronDensityPerVoxelAll);

}

//endregion

//region Neuron Voxel Index and Count

crate::define_quantizable_uint_type_family!(NeuronVoxelIndexCount);

//endregion

//region Neuron Voxel Coordinate

crate::define_unsigned_coordinate_3d_type_family!(NeuronVoxelCoordinate);

//endregion

//region Neuron Voxel Dimensions

crate::define_dimension_3d_type_family!(NeuronVoxelDimensions, NeuronVoxelCoordinate);

impl<VoxelIndexCountCoordQuant: QuantizableUIntType> NeuronVoxelDimensions<VoxelIndexCountCoordQuant> {

    pub fn get_number_voxels(&self) -> NeuronVoxelIndexCount<VoxelIndexCountCoordQuant> {
        NeuronVoxelIndexCount::from_usize(self.number_elements())
    }

    pub fn get_number_neurons(&self, density: &NeuronDensityPerVoxel) -> IndividualNeuronIndexCount<VoxelIndexCountCoordQuant> {
        IndividualNeuronIndexCount::from_usize(self.number_elements() * density.to_usize())
    }

    /// Linear voxel index with **x varying fastest**: `index = x + y·dx + z·dx·dy`.
    #[inline(always)]
    pub fn linear_index_to_standard_voxel_coordinate(
        &self,
        index: NeuronVoxelIndexCount<VoxelIndexCountCoordQuant>,
    ) -> NeuronVoxelCoordinate<VoxelIndexCountCoordQuant> {
        let i = QuantizableUIntType::to_usize(index);
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
    pub fn voxel_standard_coordinate_to_linear_index(
        &self,
        coordinate: NeuronVoxelCoordinate<VoxelIndexCountCoordQuant>,
    ) -> NeuronVoxelIndexCount<VoxelIndexCountCoordQuant> {
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let x = coordinate.x.to_usize();
        let y = coordinate.y.to_usize();
        let z = coordinate.z.to_usize();
        let i = x + y * dx + z * dx * dy;
        NeuronVoxelIndexCount::from_usize(i)
    }

    // TODO iterators
}

//endregion

//region Neuron Voxel Potential

crate::define_quantizable_value_type_family!(NeuronVoxelPotential);

impl<PotentialQuant: QuantizableValueType> NeuronVoxelPotential<PotentialQuant> {

    pub fn voxel_potential_from_sum_neurons(&mut self,
                                            neurons: &[IndividualNeuronMembranePotential<PotentialQuant>])
                                            -> NeuronVoxelPotential<PotentialQuant>
    {
        neurons.iter().fold(NeuronVoxelPotential::ZERO, |acc, neuron| {
            acc.saturating_add(NeuronVoxelPotential(neuron.0))
        })
    }


    pub fn voxel_potential_from_sum_neurons_in_place(&mut self,
                                                     neurons: &[IndividualNeuronMembranePotential<PotentialQuant>])
    {
        *self = NeuronVoxelPotential::ZERO;
        neurons.iter().for_each(|neuron| {
            self.saturating_add(NeuronVoxelPotential(neuron.0));
        })

    }


}

//endregion
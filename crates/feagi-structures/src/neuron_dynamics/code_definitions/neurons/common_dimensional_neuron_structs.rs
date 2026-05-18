use core::ops::Range;
use crate::base_feagi_types::quantizable_types::{QuantizableNonzeroUIntType, QuantizableUIntType, QuantizableValueType};
use crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::LinearNeuronIndexCount;

//region Neuron Density Per Voxel

// We do this since we only want to expose the u8 level
pub use all_neuron_densities::NeuronDensityPerVoxel;
mod all_neuron_densities {

    /// The number of neuron_collections that a single voxel represents. In most contexts this will be 1,
    /// but sometimes may be more, though never high, hence being locked to a u8. Cannot be 0
    pub type NeuronDensityPerVoxel = NeuronDensityPerVoxelAll<u8>;

    /// This defines multiple quantizations, which we dont care for
    crate::define_nonzero_count_family!(NeuronDensityPerVoxelAll);

}

//endregion

//region Neuron Voxel Index and Count

crate::define_quantizable_uint_type_family!(VoxelIndexCount);

impl<VoxelIndexCountCoordQuant: QuantizableUIntType> VoxelIndexCount<VoxelIndexCountCoordQuant> {
    pub fn calculate_linear_index_range(&self,
                                        density: LinearNeuronIndexCount<NeuronDensityPerVoxel>)
                                        -> Range<LinearNeuronIndexCount<VoxelIndexCountCoordQuant>>
    {
        let density = density.through_usize_to_quant::<VoxelIndexCountCoordQuant>();
        let start = self.0 / density;
        LinearNeuronIndexCount(start)..LinearNeuronIndexCount(start + density)
    }
}

//endregion

//region Neuron Voxel Coordinate

crate::define_unsigned_coordinate_3d_type_family!(VoxelCoordinate);

//endregion

//region Neuron Voxel Dimensions

crate::define_dimension_3d_type_family!(VoxelDimensions, VoxelCoordinate);

impl<VoxelIndexCountCoordQuant: QuantizableUIntType> VoxelDimensions<VoxelIndexCountCoordQuant> {

    pub fn get_number_voxels(&self) -> VoxelIndexCount<VoxelIndexCountCoordQuant> {
        VoxelIndexCount::from_usize(self.number_elements())
    }

    pub fn get_number_neurons(&self, density: &NeuronDensityPerVoxel) -> LinearNeuronIndexCount<VoxelIndexCountCoordQuant> {
        LinearNeuronIndexCount::from_usize(self.number_elements() * density.to_usize())
    }

    // TODO remove to_usize conversions
    /// Linear voxel index with **x varying fastest**: `index = x + y·dx + z·dx·dy`.
    #[inline(always)]
    pub fn linear_index_to_standard_voxel_coordinate(
        &self,
        index: VoxelIndexCount<VoxelIndexCountCoordQuant>,
    ) -> VoxelCoordinate<VoxelIndexCountCoordQuant> {
        let i = QuantizableUIntType::to_usize(index);
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let plane = dx * dy;
        let z = i / plane;
        let rem = i % plane;
        let y = rem / dx;
        let x = rem % dx;
        VoxelCoordinate::new(
            VoxelIndexCountCoordQuant::from_usize(x),
            VoxelIndexCountCoordQuant::from_usize(y),
            VoxelIndexCountCoordQuant::from_usize(z),
        )
    }

    /// Inverse of [`Self::linear_index_to_coordinate`].
    #[inline(always)]
    pub fn voxel_standard_coordinate_to_linear_index(
        &self,
        coordinate: VoxelCoordinate<VoxelIndexCountCoordQuant>,
    ) -> VoxelIndexCount<VoxelIndexCountCoordQuant> {
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let x = coordinate.x.to_usize();
        let y = coordinate.y.to_usize();
        let z = coordinate.z.to_usize();
        let i = x + y * dx + z * dx * dy;
        VoxelIndexCount::from_usize(i)
    }

    // TODO iterators
}

//endregion




/*
//region Neuron Voxel Potential

crate::define_quantizable_value_type_family!(VoxelPotential);

impl<PotentialQuant: QuantizableValueType> VoxelPotential<PotentialQuant> {

    pub fn voxel_potential_from_sum_neurons(&mut self,
                                            neurons: &[NeuronMembranePotential<PotentialQuant>])
                                            -> VoxelPotential<PotentialQuant>
    {
        neurons.iter().fold(VoxelPotential::ZERO, |acc, neuron| {
            acc.saturating_add(VoxelPotential(neuron.0))
        })
    }


    pub fn voxel_potential_from_sum_neurons_in_place(&mut self,
                                                     neurons: &[NeuronMembranePotential<PotentialQuant>])
    {
        *self = VoxelPotential::ZERO;
        neurons.iter().for_each(|neuron| {
            self.saturating_add(VoxelPotential(neuron.0));
        })

    }


}

//endregion

 */
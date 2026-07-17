use crate::feagi_quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::feagi_quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_voxels::neuron_voxel_error::FeagiVoxelError;
use crate::neuron_voxels::wrapped_values::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelLinearIndex, NeuronVoxelPotential};

pub trait NeuronVoxelCollection<FIQ: FeagiIndexQuantization,  CPQ: CorticalPotentialQuantization>
{
    /// Get the dimensions of the voxel structure
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensions<FIQ::NeuronIndexCountQuant>;


    fn max_linear_index(&self) -> NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant> {
        self.get_voxel_dimensions().number_contained_elements()
    }

    fn try_get_potential_by_voxel_index(&self, voxel_index: NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotential<CPQ::MembranePotentialQuant>>;

    fn try_get_potential_by_voxel_index_mut(&mut self, voxel_index: NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotential<CPQ::MembranePotentialQuant>>;



    /// Tries setting a neuron voxel potential at a voxel index. Returns any value being bumped out if it is.
    fn try_set_potential_at_voxel_index(
        &mut self,
        voxel_index: NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant>,
        potential: NeuronVoxelPotential<CPQ::MembranePotentialQuant>)
        -> Result<Option<NeuronVoxelPotential<CPQ::MembranePotentialQuant>>, FeagiVoxelError>;

    fn try_insert_potential_at_voxel_coordinate(
        &mut self,
        voxel_coordinate: NeuronVoxelCoordinate<FIQ::NeuronIndexCountQuant>,
        potential: NeuronVoxelPotential<CPQ::MembranePotentialQuant>)
        -> Result<Option<NeuronVoxelPotential<CPQ::MembranePotentialQuant>>, FeagiVoxelError>
    {
        self.try_set_potential_at_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate),
            potential
        )
    }

    fn iter_with_voxel_index(&self)
                             -> impl Iterator<Item=(NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant>, &NeuronVoxelPotential<CPQ::MembranePotentialQuant>)>;

    fn iter_mut_with_voxel_index(&mut self)
                                 -> impl Iterator<Item=(NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotential<CPQ::MembranePotentialQuant>)>;

    fn iter_with_index_and_coordinate(&self)
                                      -> impl Iterator<Item=(NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinate<FIQ::NeuronIndexCountQuant>, &NeuronVoxelPotential<CPQ::MembranePotentialQuant>)>;

    fn iter_mut_with_index_and_coordinate(&mut self)
                                          -> impl Iterator<Item=(NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinate<FIQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotential<CPQ::MembranePotentialQuant>)>;


    fn try_get_potential_by_voxel_coordinate(&self,  voxel_coordinate: NeuronVoxelCoordinate<FIQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotential<CPQ::MembranePotentialQuant>> {
        self.try_get_potential_by_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate)
        )
    }

    fn try_get_potential_by_voxel_coordinate_mut(&mut self,  voxel_coordinate: NeuronVoxelCoordinate<FIQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotential<CPQ::MembranePotentialQuant>> {
        self.try_get_potential_by_voxel_index_mut(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate)
        )
    }

}

pub trait NeuronVoxelCollectionDense<FIQ: FeagiIndexQuantization,  CPQ: CorticalPotentialQuantization>:
NeuronVoxelCollection<FIQ, CPQ>
{
    fn get_neuron_voxel_potentials_slice(&self) -> &[NeuronVoxelPotential<CPQ::MembranePotentialQuant>];

    fn get_neuron_voxel_potentials_slice_mut(&mut self) -> &mut [NeuronVoxelPotential<CPQ::MembranePotentialQuant>];
}


pub trait NeuronVoxelCollectionSparse<FIQ: FeagiIndexQuantization,  CPQ: CorticalPotentialQuantization>:
NeuronVoxelCollection<FIQ, CPQ>
{
    /// Tries to remove a neuron voxel potential at a voxel given index. Returns any value that was removed if there was one there
    fn remove_potential_at_voxel_index(
        &mut self,
        index: NeuronVoxelLinearIndex<FIQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotential<CPQ::MembranePotentialQuant>>;

    fn remove_potential_at_voxel_coordinate(
        &mut self,
        voxel_coordinate: NeuronVoxelCoordinate<FIQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotential<CPQ::MembranePotentialQuant>>
    {

        self.remove_potential_at_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate),
        )
    }
}


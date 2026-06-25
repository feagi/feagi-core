use crate::neuron_voxels::voxel_collection_generic_descriptors::*;
use crate::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;

pub trait NeuronVoxelCollection<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> 
{
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<FGQ::NeuronIndexCountQuant>;

    fn max_linear_index(&self) -> NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant> {
        self.get_voxel_dimensions().max_linear_index()
    }
}

pub trait NeuronVoxelCollectionDense<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
NeuronVoxelCollection<FGQ, CPQ>
{
    // No certain access to data
}


pub trait NeuronVoxelCollectionSparse<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
NeuronVoxelCollection<FGQ, CPQ>
{
    // No certain access to data
}




pub trait CPUNeuronVoxelCollection<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
NeuronVoxelCollection<FGQ, CPQ>
{
    fn try_get_potential_by_voxel_index(&self, voxel_index: NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>;

    fn try_get_potential_by_voxel_index_mut(&mut self, voxel_index: NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>;
    
    
    fn iter_with_voxel_index(&self)
                             -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>;

    fn iter_mut_with_voxel_index(&mut self)
                                 -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>;

    fn iter_with_index_and_coordinate(&self)
                                      -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<FGQ::NeuronIndexCountQuant>, &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>;

    fn iter_mut_with_index_and_coordinate(&mut self)
                                          -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<FGQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>;


    fn try_get_potential_by_voxel_coordinate(&self,  voxel_coordinate: NeuronVoxelCoordinateGeneric<FGQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.try_get_potential_by_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate)
        )
    }

    fn try_get_potential_by_voxel_coordinate_mut(&mut self,  voxel_coordinate: NeuronVoxelCoordinateGeneric<FGQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.try_get_potential_by_voxel_index_mut(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate)
        )
    }

}

pub trait CPUNeuronVoxelCollectionDense<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
CPUNeuronVoxelCollection<FGQ, CPQ>
+ NeuronVoxelCollectionDense<FGQ, CPQ>
{
    fn get_neuron_voxel_potentials_slice(&self) -> &[NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>];

    fn get_neuron_voxel_potentials_slice_mut(&mut self) -> &mut [NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>];
}


pub trait CPUNeuronVoxelCollectionSparse<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
CPUNeuronVoxelCollection<FGQ, CPQ>
+ NeuronVoxelCollectionSparse<FGQ, CPQ>
{
    /// Tries inserting a neuron voxel potential at a voxel index. Returns any value being bumped out if it is.
    fn insert_potential_at_voxel_index(
        &mut self, 
        voxel_index: NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, 
        potential: NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>) 
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>;

    /// Tries to remove a neuron voxel potential at a voxel given index. Returns any value that was removed if there was one there
    fn remove_potential_at_voxel_index(
        &mut self, 
        index: NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>;

    fn insert_potential_at_voxel_coordinate(
        &mut self,
        voxel_coordinate: NeuronVoxelCoordinateGeneric<FGQ::NeuronIndexCountQuant>,
        potential: NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> 
    {
        
        self.insert_potential_at_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate),
            potential
        )
    }

    fn remove_potential_at_voxel_coordinate(
        &mut self,
        voxel_coordinate: NeuronVoxelCoordinateGeneric<FGQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> 
    {

        self.remove_potential_at_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate),
        )
    }
}
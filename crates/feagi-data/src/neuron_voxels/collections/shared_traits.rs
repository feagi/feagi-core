use crate::neuron_voxels::voxel_collection_generic_descriptors::*;
use crate::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;

pub trait NeuronVoxelCollection<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>
{
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<FIQ::NeuronIndexCountQuant>;

    fn max_linear_index(&self) -> NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant> {
        self.get_voxel_dimensions().max_linear_index()
    }
}

pub trait NeuronVoxelCollectionDense<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
NeuronVoxelCollection<FIQ, CPQ>
{
    // No certain access to data
}


pub trait NeuronVoxelCollectionSparse<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
NeuronVoxelCollection<FIQ, CPQ>
{
    // No certain access to data
}




pub trait CPUNeuronVoxelCollection<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
NeuronVoxelCollection<FIQ, CPQ>
{
    fn try_get_potential_by_voxel_index(&self, voxel_index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>;

    fn try_get_potential_by_voxel_index_mut(&mut self, voxel_index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>;
    
    
    fn iter_with_voxel_index(&self)
                             -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>, &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>;

    fn iter_mut_with_voxel_index(&mut self)
                                 -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>;

    fn iter_with_index_and_coordinate(&self)
                                      -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>, &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>;

    fn iter_mut_with_index_and_coordinate(&mut self)
                                          -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>;


    fn try_get_potential_by_voxel_coordinate(&self,  voxel_coordinate: NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.try_get_potential_by_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate)
        )
    }

    fn try_get_potential_by_voxel_coordinate_mut(&mut self,  voxel_coordinate: NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.try_get_potential_by_voxel_index_mut(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate)
        )
    }

}

pub trait CPUNeuronVoxelCollectionDense<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
CPUNeuronVoxelCollection<FIQ, CPQ>
+ NeuronVoxelCollectionDense<FIQ, CPQ>
{
    fn get_neuron_voxel_potentials_slice(&self) -> &[NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>];

    fn get_neuron_voxel_potentials_slice_mut(&mut self) -> &mut [NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>];
}


pub trait CPUNeuronVoxelCollectionSparse<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>:
CPUNeuronVoxelCollection<FIQ, CPQ>
+ NeuronVoxelCollectionSparse<FIQ, CPQ>
{
    /// Tries inserting a neuron voxel potential at a voxel index. Returns any value being bumped out if it is.
    fn insert_potential_at_voxel_index(
        &mut self, 
        voxel_index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>,
        potential: NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>) 
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>;

    /// Tries to remove a neuron voxel potential at a voxel given index. Returns any value that was removed if there was one there
    fn remove_potential_at_voxel_index(
        &mut self, 
        index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>;

    fn insert_potential_at_voxel_coordinate(
        &mut self,
        voxel_coordinate: NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>,
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
        voxel_coordinate: NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> 
    {

        self.remove_potential_at_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate),
        )
    }
}
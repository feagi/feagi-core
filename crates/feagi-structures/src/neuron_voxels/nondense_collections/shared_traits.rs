use feagi_data::quantizable_spatial::index::SpatialIndexCoordinate3D;
use feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, CorticalAreasIndexQuantization};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;

pub trait NeuronVoxelCollection<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> 
{
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<CAIQ::NeuronIndexCountQuant>;

    fn max_linear_index(&self) -> NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant> {
        self.get_voxel_dimensions().max_linear_index()
    }
}

pub trait NeuronVoxelCollectionDense<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase>:
NeuronVoxelCollection<CAIQ, CANQ>
{
    // No certain access to data
}


pub trait NeuronVoxelCollectionSparse<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase>:
NeuronVoxelCollection<CAIQ, CANQ>
{
    // No certain access to data
}




pub trait CPUNeuronVoxelCollection<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase>:
NeuronVoxelCollection<CAIQ, CANQ>
{
    fn try_get_potential_by_voxel_index(&self,  voxel_index: NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>>;

    fn try_get_potential_by_voxel_index_mut(&mut self,  voxel_index: NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>>;
    
    
    fn iter_with_voxel_index<'a>(&'a self)
        -> impl Iterator<Item = (NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, &'a NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)> where NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a;

    fn iter_mut_with_voxel_index<'a>(&'a mut self)
        -> impl Iterator<Item = (NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, &'a mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)> where NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a;

    fn iter_with_index_and_coordinate<'a>(&'a self)
        -> impl Iterator<Item = (NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<CAIQ::NeuronIndexCountQuant>, &'a NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)> where NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a;

    fn iter_mut_with_index_and_coordinate<'a>(&'a mut self) 
        -> impl Iterator<Item = (NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<CAIQ::NeuronIndexCountQuant>, &'a mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)> where NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a;


    fn try_get_potential_by_voxel_coordinate(&self,  voxel_coordinate: NeuronVoxelCoordinateGeneric<CAIQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>> {
        self.try_get_potential_by_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate)
        )
    }

    fn try_get_potential_by_voxel_coordinate_mut(&mut self,  voxel_coordinate: NeuronVoxelCoordinateGeneric<CAIQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>> {
        self.try_get_potential_by_voxel_index_mut(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate)
        )
    }

}

pub trait CPUNeuronVoxelCollectionDense<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase>:
CPUNeuronVoxelCollection<CAIQ, CANQ>
+ NeuronVoxelCollectionDense<CAIQ, CANQ>
{
    fn get_neuron_voxel_potentials_slice(&self) -> &[NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>];

    fn get_neuron_voxel_potentials_slice_mut(&mut self) -> &mut [NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>];
}


pub trait CPUNeuronVoxelCollectionSparse<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase>:
CPUNeuronVoxelCollection<CAIQ, CANQ>
+ NeuronVoxelCollectionSparse<CAIQ, CANQ>
{
    /// Tries inserting a neuron voxel potential at a voxel index. Returns any value being bumped out if it is.
    fn insert_potential_at_voxel_index(
        &mut self, 
        voxel_index: NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, 
        potential: NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>) 
        -> Option<NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>>;

    /// Tries to remove a neuron voxel potential at a voxel given index. Returns any value that was removed if there was one there
    fn remove_potential_at_voxel_index(
        &mut self, 
        index: NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>>;

    fn insert_potential_at_voxel_coordinate(
        &mut self,
        voxel_coordinate: NeuronVoxelCoordinateGeneric<CAIQ::NeuronIndexCountQuant>,
        potential: NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>> 
    {
        
        self.insert_potential_at_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate),
            potential
        )
    }

    fn remove_potential_at_voxel_coordinate(
        &mut self,
        voxel_coordinate: NeuronVoxelCoordinateGeneric<CAIQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>> 
    {

        self.remove_potential_at_voxel_index(
            self.get_voxel_dimensions().coordinate_to_linear_index(voxel_coordinate),
        )
    }
}
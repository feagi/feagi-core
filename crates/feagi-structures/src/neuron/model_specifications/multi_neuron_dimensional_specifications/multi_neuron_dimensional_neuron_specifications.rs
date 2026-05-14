use crate::define_ref_access_trait_methods;
use crate::neuron::FeagiNeuronError;
use crate::neuron::model_specifications::base_specifications::{BaseNeuronCollectionSharedTrait, LinearNeuronIndexCount};
use crate::neuron::model_specifications::base_dimensional_specifications::dimensional_neuron_common_structs::{NeuronDensityPerVoxel, VoxelCoordinate, VoxelDimensions, VoxelIndexCount};
use crate::quantization_level::CorticalAreaNeuronQuantization;


pub trait MultiNeuronDimensionalNeuronModelCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseNeuronCollectionSharedTrait<CANQ> {
    fn get_number_neurons_per_voxel(&self) -> LinearNeuronIndexCount<NeuronDensityPerVoxel>;

    fn try_get_voxel_data_ref(&self, index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<, FeagiNeuronError>;

    fn try_get_voxel_data_ref_mut(&mut self, index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<, FeagiNeuronError>;
    
    fn get_number_contained_voxels(&self) -> VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// What is the upper bound (exclusive) neuron voxel index allowed?
    fn get_voxel_max_index(&self) -> VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.get_cortical_area_voxel_dimensions().get_number_voxels()
    }
}


// TODO? There are weird risks here with trying to grab a set of independent neurons with some that are missing
/*
pub trait MultiNeuronDimensionalNeuronModelCollectionSparseTrait<CANQ: CorticalAreaNeuronQuantization>:
MultiNeuronDimensionalNeuronModelCollectionSharedTrait<CANQ>
{

}
 */


pub trait MultiNeuronDimensionalNeuronModelCollectionDenseTrait<CANQ: CorticalAreaNeuronQuantization>:
MultiNeuronDimensionalNeuronModelCollectionSharedTrait<CANQ>
{
    fn get_enumerated_voxel_iterator_single_density(&self)
                                                    -> Result<impl Iterator<Item=VoxelSmartIterator<CANQ>>, FeagiNeuronError> {
        todo!()
    }
}


pub trait MultiNeuronDimensionalNeuronModelCollectionResizableTrait<CANQ: CorticalAreaNeuronQuantization>:
MultiNeuronDimensionalNeuronModelCollectionSharedTrait<CANQ>
{
    fn resize_neuron_data_vectors_for_new_dimensions(&mut self,
                                                     new_dimensions: VoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
                                                     neurons_per_voxel: LinearNeuronIndexCount<NeuronDensityPerVoxel>);
}


pub struct MultiNeuronVoxelDataRefContainer<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: MultiNeuronDimensionalNeuronModelCollectionSharedTrait<CANQ>> {
    data_neurons_of_voxel: &'a [BNMC::SingleNeuronReference]
}





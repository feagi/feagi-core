use crate::neuron::model_specifications::base_specifications::{BaseNeuronCollectionSharedTrait};
use crate::neuron::model_specifications::base_dimensional_specifications::dimensional_neuron_common_structs::{VoxelDimensions, VoxelIndexCount};
use crate::quantization_level::CorticalAreaNeuronQuantization;


pub trait BaseDimensionalNeuronCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseNeuronCollectionSharedTrait<CANQ> {
    fn get_cortical_area_voxel_dimensions(&self) -> &VoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>;

    
    
    /// What is the upper bound (exclusive) neuron voxel index allowed?
    fn get_voxel_max_index(&self) -> VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.get_cortical_area_voxel_dimensions().get_number_voxels()
    }
}



pub trait BaseDimensionalNeuronCollectionSparseTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronCollectionSharedTrait<CANQ>
{

}


pub trait BaseDimensionalNeuronCollectionDenseTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronCollectionSharedTrait<CANQ>
{

}

// TODO conversion between single and multi?
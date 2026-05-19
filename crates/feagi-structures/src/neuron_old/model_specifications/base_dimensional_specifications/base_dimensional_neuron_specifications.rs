use std::ops::Range;
use crate::neuron_old::FeagiNeuronError;
use crate::neuron_old::model_specifications::base_specifications::{BaseNeuronCollectionSharedTrait, LinearNeuronIndexCount};
use crate::neuron_old::model_specifications::base_dimensional_specifications::dimensional_neuron_common_structs::{NeuronDensityPerVoxel, VoxelDimensions, VoxelIndexCount};
use crate::neuron_old::model_specifications::base_dimensional_specifications::multi_neuron_containers::{VoxelMultiNeuronMutRefContainer, VoxelMultiNeuronRefContainer};
use crate::quantization_level::CorticalAreaNeuronQuantization;


pub trait BaseDimensionalNeuronCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseNeuronCollectionSharedTrait<CANQ> {
    fn get_cortical_area_voxel_dimensions(&self) -> &VoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>;

    fn get_number_neurons_per_voxel(&self) -> LinearNeuronIndexCount<NeuronDensityPerVoxel>;

    fn is_single_neuron_per_voxel(&self) -> bool;
    
    fn try_get_voxel_data_ref(&self, voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<VoxelMultiNeuronRefContainer<CANQ, Self::SingleNeuronReference>, FeagiNeuronError>;

    fn try_get_voxel_data_ref_mut(&mut self, voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<VoxelMultiNeuronMutRefContainer<CANQ, Self::SingleNeuronReference>, FeagiNeuronError>;

    /// What is the upper bound (exclusive) neuron voxel index allowed?
    fn get_voxel_max_index(&self) -> VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.get_cortical_area_voxel_dimensions().get_number_voxels()
    }

    fn get_linear_range_from_voxel_index(&self, voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<Range<LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>>, FeagiNeuronError> {
        // TODO debug check if in voxel max range!

        // TODO debug check if range contains all existing elements!
        Ok(voxel_index.calculate_linear_index_range(self.get_number_neurons_per_voxel()))
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
use crate::neuron::FeagiNeuronError;
use crate::neuron::model_specifications::base_dimensional_specifications::base_dimensional_neuron_specifications::BaseDimensionalNeuronCollectionSharedTrait;
use crate::neuron::model_specifications::base_dimensional_specifications::dimensional_neuron_common_structs::{VoxelDimensions, VoxelIndexCount};
use crate::neuron::model_specifications::base_specifications::LinearNeuronIndexCount;
use crate::quantization_level::CorticalAreaNeuronQuantization;


pub trait SingleNeuronDimensionalNeuronCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronCollectionSharedTrait<CANQ> {
    fn try_get_voxel_single_neuron_data_ref(&self, index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&Self::SingleNeuronReference, FeagiNeuronError> {
        self.try_get_neuron_data_ref(LinearNeuronIndexCount(index.0))
    }

    fn try_get_voxel_single_neuron_data_ref_mut(&mut self, index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut Self::SingleNeuronReference, FeagiNeuronError> {
        self.try_get_neuron_data_ref_mut(LinearNeuronIndexCount(index.0))
    }
}



pub trait SingleNeuronDimensionalNeuronCollectionSparseTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronCollectionSharedTrait<CANQ>
{
    // TODO?
}


pub trait SingleNeuronDimensionalNeuronCollectionDenseTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronCollectionSharedTrait<CANQ>
{

}

/// A Dimensional Neuron Container that can allocate additional memory to resize
pub trait SingleNeuronDimensionalNeuronCollectionResizableTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronCollectionSharedTrait<CANQ>
{
    fn resize_neuron_data_vectors_for_new_dimensions(&mut self,
                                                     new_dimensions: VoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>);
}
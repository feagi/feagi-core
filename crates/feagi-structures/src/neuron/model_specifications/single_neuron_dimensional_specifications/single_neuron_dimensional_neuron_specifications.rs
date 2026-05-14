use crate::quantization_level::CorticalAreaNeuronQuantization;


pub trait SingleNeuronDimensionalNeuronCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronModelCollectionSharedTrait<CANQ> {
    
    fn try_get_voxel_data_ref(&self, index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&Self::SingleNeuronReference, FeagiNeuronError>;

    fn try_get_voxel_data_ref_mut(&mut self, index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut Self::SingleNeuronReference, FeagiNeuronError>;
}



pub trait SingleNeuronDimensionalNeuronCollectionSparseTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronModelCollectionSharedTrait<CANQ>
{
    // TODO?
}


pub trait SingleNeuronDimensionalNeuronCollectionDenseTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronModelCollectionSharedTrait<CANQ>
{

}

/// A Dimensional Neuron Container that can allocate additional memory to resize
pub trait SingleNeuronDimensionalNeuronCollectionResizableTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronModelCollectionSharedTrait<CANQ>
{
    fn resize_neuron_data_vectors_for_new_dimensions(&mut self,
                                                     new_dimensions: VoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>);
}
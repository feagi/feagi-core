use crate::neuron_old::model_specifications::base_dimensional_specifications::base_dimensional_neuron_specifications::BaseDimensionalNeuronCollectionSharedTrait;
use crate::neuron_old::model_specifications::base_specifications::LinearNeuronIndexCount;
use crate::quantization_level::CorticalAreaNeuronQuantization;


pub struct VoxelMultiNeuronRefContainer<'a, CANQ: CorticalAreaNeuronQuantization, BDNC: BaseDimensionalNeuronCollectionSharedTrait<CANQ>> {
    data_neurons_of_voxel: &'a [BDNC::SingleNeuronReference]
}

pub struct VoxelMultiNeuronMutRefContainer<'a, CANQ: CorticalAreaNeuronQuantization, BDNC: BaseDimensionalNeuronCollectionSharedTrait<CANQ>> {
    data_neurons_of_voxel: &'a mut [BDNC::SingleNeuronReference]
}

// TODO multi voxel container, where we still keep a flat data_neurons_of_voxel but allow iterating by VoxelMultiNeuronRefContainer

pub struct MultiVoxelMultiNeuronRefContainer<'a, CANQ: CorticalAreaNeuronQuantization, BDNC: BaseDimensionalNeuronCollectionSharedTrait<CANQ>> {
    data_neurons_of_voxel: &'a [BDNC::SingleNeuronReference],
    density: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>
}

pub struct MultiVoxelMultiNeuronMutRefContainer<'a, CANQ: CorticalAreaNeuronQuantization, BDNC: BaseDimensionalNeuronCollectionSharedTrait<CANQ>> {
    data_neurons_of_voxel: &'a mut [BDNC::SingleNeuronReference],
    density: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>
}
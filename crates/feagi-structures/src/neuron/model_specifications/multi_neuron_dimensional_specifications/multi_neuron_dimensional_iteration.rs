use crate::neuron::model_specifications::base_dimensional_specifications::dimensional_neuron_common_structs::VoxelIndexCount;
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub struct EnumeratedMultiNeuronVoxelIteratedItem<'a, CANQ: CorticalAreaNeuronQuantization, NCT: MultiNeuronVoxelNeuronCollectionSharedTrait<CANQ>> {
    voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a NCT
}
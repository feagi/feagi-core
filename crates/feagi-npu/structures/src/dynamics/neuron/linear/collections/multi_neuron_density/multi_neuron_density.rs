use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::LinearNeuronIndexCount;
use crate::dynamics::neuron::linear::collections::NeuronModelCollectionBaseLinearTrait;

pub trait NeuronModelCollectionMultiNeuronLinearTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelCollectionBaseLinearTrait<CANQ>
{
    fn get_number_neurons_per_unit(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;


}
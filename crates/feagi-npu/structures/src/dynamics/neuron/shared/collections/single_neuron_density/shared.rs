use feagi_structures::CorticalAreaNeuronQuantization;
use crate::dynamics::neuron::linear::collections::NeuronModelCollectionBaseLinearTrait;
use crate::dynamics::neuron::shared::neurons::NeuronModelParametersTrait;

// NOTE: This trait shouldnt have conversion to multi neuron, that should only be multineruon itself

pub trait NeuronModelCollectionSingleNeuronLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    // I dont know if anything even exists to be added here? Exists only as a requirement to support resize
}

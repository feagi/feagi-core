use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount};
use crate::dynamics::neuron::linear::collections::NeuronModelCollectionBaseLinearTrait;
use crate::dynamics::neuron::linear::neurons::NeuronModelParametersTrait;

// NOTE: This trait shouldnt have conversion to multi neuron, that should only be multineruon itself

pub trait NeuronModelCollectionSingleNeuronLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    // I dont know if anything even exists to be added here?
}

/// Optional Trait that allows linear resizing
pub trait NeuronModelCollectionSingleNeuronLinearResizableTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionSingleNeuronLinearTrait<CANQ, NMP>
{
    fn resize_single_linear_neuron_collection(&mut self,
                                              new_total_neuron_count: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>, 
                                              clear_neurons_first: bool) -> Result<(), FeagiNeuronError>;
}
use crate::neuron::model_specifications::base_specifications::{BaseNeuronCollectionSharedTrait};
use crate::quantization_level::CorticalAreaNeuronQuantization;



pub trait BasePropagatingNeuronsCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseNeuronCollectionSharedTrait<CANQ>
{
    type CorticalConfigurationType;
    // NOTE: Do NOT store CorticalConfigurationType in this struct as it risks multithreading locks
}





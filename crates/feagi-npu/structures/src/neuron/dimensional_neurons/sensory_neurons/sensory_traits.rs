use crate::neuron::base_dimension_traits::{DimensionalAllocStorageTrait, DimensionalStaticStorageTrait};
use crate::neuron::base_traits::{BaseNeuronAllocStorageTrait, BaseNeuronStaticStorageTrait};
use crate::neuron::dimensional_neurons::dimensional_traits::{DimensionalNeuronAllocStorageTrait, DimensionalNeuronStaticStorageTrait};
use crate::quantizables::NPUQuantization;

pub trait SensoryNeuronBaseTrait<Q: NPUQuantization>:
{
    
}

pub trait SensoryNeuronStaticStorageTrait<Q: NPUQuantization>:
SensoryNeuronBaseTrait<Q> +
DimensionalNeuronStaticStorageTrait<Q>
{

}


#[cfg(feature = "alloc")]
pub trait SensoryNeuronAllocStorageTrait<Q: NPUQuantization>:
SensoryNeuronBaseTrait<Q> +
DimensionalNeuronAllocStorageTrait<Q>
{

}
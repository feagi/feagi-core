use crate::neuron::base_dimension_traits::{DimensionalAllocStorageTrait, DimensionalStaticStorageTrait};
use crate::neuron::base_traits::{BaseNeuronAllocStorageTrait, BaseNeuronStaticStorageTrait};
use crate::neuron::dimensional_neurons::dimensional_traits::{DimensionalNeuronAllocStorageTrait, DimensionalNeuronStaticStorageTrait};
use crate::quantizables::NPUDataQuantization;

pub trait SensoryNeuronBaseTrait<Q: NPUDataQuantization>:
{
    
}

pub trait SensoryNeuronStaticStorageTrait<Q: NPUDataQuantization>:
SensoryNeuronBaseTrait<Q> +
DimensionalNeuronStaticStorageTrait<Q>
{

}


#[cfg(feature = "alloc")]
pub trait SensoryNeuronAllocStorageTrait<Q: NPUDataQuantization>:
SensoryNeuronBaseTrait<Q> +
DimensionalNeuronAllocStorageTrait<Q>
{

}
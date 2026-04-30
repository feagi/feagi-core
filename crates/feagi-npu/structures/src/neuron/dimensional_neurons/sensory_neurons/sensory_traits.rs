use crate::neuron::base_dimension_traits::{DimensionalAllocStorageTrait, DimensionalStaticStorageTrait};
use crate::neuron::base_storage_traits::{BaseNeuronResizableStorageTrait, BaseNeuronFixedStorageTrait};
use crate::neuron::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronResizableStorageTrait, DimensionalNeuronFixedStorageTrait};
use crate::quantizables::NPUGlobalQuantization;

pub trait SensoryNeuronBaseTrait<Q: NPUGlobalQuantization>:
{
    
}

pub trait SensoryNeuronStaticStorageTrait<Q: NPUGlobalQuantization>:
SensoryNeuronBaseTrait<Q> +
DimensionalNeuronFixedStorageTrait<Q>
{

}


#[cfg(feature = "alloc")]
pub trait SensoryNeuronAllocStorageTrait<Q: NPUGlobalQuantization>:
SensoryNeuronBaseTrait<Q> +
DimensionalNeuronResizableStorageTrait<Q>
{

}
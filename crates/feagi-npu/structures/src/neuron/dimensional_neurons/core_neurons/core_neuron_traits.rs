// Core neurons are a bit unique as they are included in all genomes and are rather static

use crate::neuron::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronResizableStorageTrait, DimensionalNeuronFixedStorageTrait};
use crate::quantizables::NPUGlobalQuantization;

pub trait CoreNeuronBaseTrait<Q: NPUGlobalQuantization>:
{

}

pub trait CoreNeuronStaticStorageTrait<Q: NPUGlobalQuantization>:
CoreNeuronBaseTrait<Q> + 
DimensionalNeuronFixedStorageTrait<Q>
{

}

#[cfg(feature = "alloc")]
pub trait CoreNeuronAllocStorageTrait<Q: NPUGlobalQuantization>:
CoreNeuronBaseTrait<Q>
// NOTE: We specifically do not import the other alloc traits as we do not support adding/removing
// cortical areas of this type! 
{

}
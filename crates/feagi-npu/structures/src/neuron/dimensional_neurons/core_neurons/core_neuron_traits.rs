// Core neurons are a bit unique as they are included in all genomes and are rather static

use crate::neuron::dimensional_neurons::dimensional_traits::{DimensionalNeuronAllocStorageTrait, DimensionalNeuronStaticStorageTrait};
use crate::quantizables::NPUDataQuantization;

pub trait CoreNeuronBaseTrait<Q: NPUDataQuantization>:
{

}

pub trait CoreNeuronStaticStorageTrait<Q: NPUDataQuantization>:
CoreNeuronBaseTrait<Q> + 
DimensionalNeuronStaticStorageTrait<Q>
{

}

#[cfg(feature = "alloc")]
pub trait CoreNeuronAllocStorageTrait<Q: NPUDataQuantization>:
CoreNeuronBaseTrait<Q>
// NOTE: We specifically do not import the other alloc traits as we do not support adding/removing
// cortical areas of this type! 
{

}
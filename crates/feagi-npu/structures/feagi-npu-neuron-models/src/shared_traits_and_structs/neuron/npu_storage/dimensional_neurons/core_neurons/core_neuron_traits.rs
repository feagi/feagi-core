// Core neurons are a bit unique as they are included in all genomes and are rather static

use crate::neuron::npu_storage::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronFixedStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};

pub trait CoreNeuronCommonStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronCommonStorageTrait<Q, DNQ>
{

}


pub trait CoreNeuronFixedStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronFixedStorageTrait<Q, DNQ> +
CoreNeuronCommonStorageTrait<Q, DNQ>
{

}


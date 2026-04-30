use crate::neuron::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronFixedStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};



pub trait InterNeuronCommonStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronCommonStorageTrait<Q, DNQ>
{
    
}


pub trait InterNeuronFixedStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronFixedStorageTrait<Q, DNQ> +
InterNeuronCommonStorageTrait<Q, DNQ>
{
    
}

#[cfg(feature = "alloc")]
pub trait InterNeuronResizableStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronResizableStorageTrait<Q, DNQ> +
InterNeuronCommonStorageTrait<Q, DNQ>
{
    
}
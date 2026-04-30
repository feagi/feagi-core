use crate::neuron::dimensional_neurons::inter_neurons::inter_neuron_traits::{InterNeuronCommonStorageTrait, InterNeuronFixedStorageTrait, InterNeuronResizableStorageTrait};
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};

pub trait FeagiStandardNeuronCommonStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
InterNeuronCommonStorageTrait<Q, DNQ>
{

}


pub trait FeagiStandardNeuronFixedStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
InterNeuronFixedStorageTrait<Q, DNQ> +
FeagiStandardNeuronCommonStorageTrait<Q, DNQ>
{

}

#[cfg(feature = "alloc")]
pub trait FeagiStandardNeuronResizableStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
InterNeuronResizableStorageTrait<Q, DNQ> +
FeagiStandardNeuronCommonStorageTrait<Q, DNQ>
{

}
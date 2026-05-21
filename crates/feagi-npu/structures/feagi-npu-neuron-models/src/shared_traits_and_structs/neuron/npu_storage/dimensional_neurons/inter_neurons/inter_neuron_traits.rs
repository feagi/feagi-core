use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_area_generator_traits::DimensionalCorticalAreaGeneratorTrait;
use crate::neuron::neuron_models::dimensional_models::dimensional_neuron_data_traits::{DimensionalNeuronModelDataSharedTrait};
use crate::neuron::npu_storage::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronFixedStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};



pub trait InterNeuronCommonStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronCommonStorageTrait<Q, DNQ>
{
    type NeuronModelType: DimensionalNeuronModelDataSharedTrait<Q, DNQ>;
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
    fn add_interneuron_cortical_area(&mut self, cortical_area_generator: &impl DimensionalCorticalAreaGeneratorTrait<Q, DNQ>) 
        -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUNeuronError>;
}
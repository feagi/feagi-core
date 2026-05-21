use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::feagi_standard::FeagiStandardCorticalAreaGenerator;
use crate::neuron::npu_storage::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronFixedStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};



pub trait MotorNeuronCommonStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronCommonStorageTrait<Q, DNQ>
{
    
}


pub trait MotorNeuronFixedStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronFixedStorageTrait<Q, DNQ> +
MotorNeuronCommonStorageTrait<Q, DNQ>
{

}

#[cfg(feature = "alloc")]
pub trait MotorNeuronResizableStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronResizableStorageTrait<Q, DNQ> +
MotorNeuronCommonStorageTrait<Q, DNQ>
{
    fn add_motor_cortical_area(&mut self, cortical_area_generator: &impl FeagiStandardCorticalAreaGenerator<Q, DNQ>)
                                     -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUNeuronError>;
}
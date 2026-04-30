
// TODO different firing / refractory mode support eventually

use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::quantizables::{NPUGlobalQuantization, BurstDelta, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronMembranePotential, NeuronExcitability, BurstGlobalIndex, NPUDimensionalNeuronQuantization};
use crate::neuron::base_storage_traits::{BaseNeuronCommonStorageTrait, BaseNeuronFixedStorageTrait, BaseNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::neuron_models::DimensionalNeuronModelDataSharedTrait;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::flags::NeuronFlag;

pub trait DimensionalNeuronCommonStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
BaseNeuronCommonStorageTrait<Q, DNQ>
{
    type DimensionalNeuronModelDataType: DimensionalNeuronModelDataSharedTrait<Q, DNQ>;

    fn get_cortical_area_data(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError>;

    fn get_cortical_area_data_mut(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&mut Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError>;
}



pub trait DimensionalNeuronFixedStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
BaseNeuronFixedStorageTrait<Q, DNQ> +
DimensionalNeuronCommonStorageTrait<Q, DNQ>
{
    // TODO ?
}


#[cfg(feature = "alloc")]
pub trait DimensionalNeuronResizableStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
BaseNeuronResizableStorageTrait<Q, DNQ> +
DimensionalNeuronCommonStorageTrait<Q, DNQ>
{
    // TODO ?
}


// TODO different firing / refractory mode support eventually

use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::quantizables::{NPUGlobalQuantization, BurstDelta, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronMembranePotential, NeuronExcitability, BurstGlobalIndex, NPUDimensionalNeuronQuantization};
use crate::neuron::base_storage_traits::{BaseNeuronCommonStorageTrait, BaseNeuronFixedStorageTrait, BaseNeuronResizableStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::flags::NeuronFlag;

pub trait DimensionalNeuronCommonStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
BaseNeuronCommonStorageTrait<Q, DNQ>
{
    fn get_cortical_data(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&DimensionalNeuronCorticalData<DNQ>, FeagiNPUNeuronError>;

    fn get_cortical_data_mut(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&mut DimensionalNeuronCorticalData<DNQ>, FeagiNPUNeuronError>;

    fn get_global_burst_index_of_last_firing(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstGlobalIndex<Q::GlobalBurstIndexQuant>], FeagiNPUNeuronError>;

    fn get_neuron_membrane_potential(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[NPUNeuronMembranePotential<DNQ::ValueQuant>], FeagiNPUNeuronError>;

    fn get_fire_threshold(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[FireThreshold<DNQ::ValueQuant>], FeagiNPUNeuronError>;

    fn get_leak_coefficient(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[LeakCoefficient<DNQ::PercentageQuant>], FeagiNPUNeuronError>;

    fn get_neuron_flags(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[NeuronFlag], FeagiNPUNeuronError>;

    fn get_refractory_countdown(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstDelta<DNQ::BurstDeltaQuant>], FeagiNPUNeuronError>;

    fn get_consecutive_fire_count(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstDelta<DNQ::BurstDeltaQuant>], FeagiNPUNeuronError>;

    /// Returns a struct of references to the slices of all neuron data (include sparse invalids)
    fn get_neuron_values_of_all_dimensional_neuron_cortical_areas_to_process(&mut self) -> DimensionalNeuronDataRefSliceAllCorticalAreas<'_, Q>;

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>)
                                                                                 -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, Q>, FeagiNPUNeuronError>;

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>, executor: &impl NeuronFireThresholdExecutor<DNQ::ValueQuant, DNQ::CoordQuant>)
                                 -> Result<(), FeagiNPUNeuronError>;


    // TODO add more specific functions for getting specific fields for neuron processing
}



pub trait DimensionalNeuronFixedStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
BaseNeuronFixedStorageTrait<Q, DNQ> +
DimensionalNeuronCommonStorageTrait<Q, DNQ>
{
    // TODO
}


#[cfg(feature = "alloc")]
pub trait DimensionalNeuronResizableStorageTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
BaseNeuronResizableStorageTrait<Q, DNQ> +
DimensionalNeuronCommonStorageTrait<Q, DNQ>
{
    // TODO ?
}

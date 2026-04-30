
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

    // TODO this may not work as different neuron types may need different things to be uniform!

    /// Creates a cortical area of given dimensions but using a set of neuron values copied across
    /// all neurons.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_uniform_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::GlobalBurstIndexQuant>,
                                                 neuron_membrane_potential: NPUNeuronMembranePotential<DNQ::ValueQuant>,
                                                 neuron_fire_threshold: FireThreshold<DNQ::ValueQuant>,
                                                 neuron_leak_coefficient: LeakCoefficient<DNQ::PercentageQuant>,
                                                 neuron_refractory_countdown: BurstDelta<DNQ::BurstDeltaQuant>,
                                                 neuron_consecutive_fire_count: BurstDelta<DNQ::BurstDeltaQuant>,
                                                 cortical_excitability: NeuronExcitability<DNQ::PercentageQuant>,
                                                 cortical_refractory_period_limit: BurstDelta<DNQ::BurstDeltaQuant>,
                                                 cortical_fire_threshold_limit: FireThresholdLimit<DNQ::ValueQuant>,
                                                 cortical_consecutive_fire_limit: BurstDelta<DNQ::BurstDeltaQuant>,
                                                 cortical_is_mp_charge_accumulation_enabled: bool,
                                                 cortical_is_mp_driven_psp_enabled: bool)
                                                 -> Result<(CorticalAreaIndex<Q::CorticalIndexQuant>), FeagiNPUNeuronError>;


    // TODO we cannot resize with a default neuron value, we need to take those in
    /*
    /// Effectively deletes a cortical area (by invalidating their neurons), then rebuilds it to the
    /// new given dimensions and density. While cortical properties are preserved, neuron data is
    /// reset to default. Returns a tuple of the old invalid neuron index range, and the new
    /// created neuron index range.
    fn resize_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
                                                 neurons_per_voxel: NeuronVoxelDensity,
                                                 cortical_index: CorticalAreaIndex<Q::CorticalIndexQuant>)
                                                 -> Result<(), FeagiNPUNeuronError>;

     */
}

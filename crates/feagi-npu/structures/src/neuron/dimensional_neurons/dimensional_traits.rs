
// TODO different firing / refractory mode support eventually

use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_dimension_traits::{DimensionalAllocStorageTrait, DimensionalStaticStorageTrait};
use crate::quantizables::{NPUQuantization, BurstDelta, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability, BurstGlobalIndex};
use crate::neuron::base_traits::{BaseNeuronAllocStorageTrait, BaseNeuronStaticStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronDataFromCorticalArea, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};


pub trait DimensionalNeuronStaticStorageTrait<Q: NPUQuantization>:
BaseNeuronStaticStorageTrait<Q>
{
    /// Returns a struct of references to the slices of all neuron data (include sparse invalids)
    fn get_neuron_values_of_all_dimensional_neuron_cortical_areas_to_process(&mut self) -> DimensionalNeuronDataRefSliceAllCorticalAreas<'_, Q>;

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: Q::CorticalIndex)
                                                                                 -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, Q>, FeagiNPUNeuronError>;

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: Q::CorticalIndex, executor: &impl NeuronFireThresholdExecutor<Q::Value, Q::Coord>)
                                 -> Result<(), FeagiNPUNeuronError>;


    // TODO add more specific functions for getting specific fields for neuron processing

}





#[cfg(feature = "alloc")]
pub trait DimensionalNeuronAllocStorageTrait<Q: NPUQuantization>:
DimensionalAllocStorageTrait<Q> +
DimensionalStaticStorageTrait<Q> +
BaseNeuronAllocStorageTrait<Q> +
DimensionalNeuronStaticStorageTrait<Q>
{
    /// Creates a cortical area of given dimensions but using a set of neuron values copied across
    /// all neurons.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_uniform_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::BurstIndex>,
                                                 neuron_membrane_potential: NPUNeuronMembranePotential<Q::Value>,
                                                 neuron_fire_threshold: FireThreshold<Q::Value>,
                                                 neuron_leak_coefficient: LeakCoefficient<Q::Percentage>,
                                                 neuron_refractory_countdown: BurstDelta<Q::BurstDelta>,
                                                 neuron_consecutive_fire_count: BurstDelta<Q::BurstDelta>,
                                                 cortical_excitability: NeuronExcitability<Q::Percentage>,
                                                 cortical_refractory_period_limit: BurstDelta<Q::BurstDelta>,
                                                 cortical_fire_threshold_limit: FireThresholdLimit<Q::Value>,
                                                 cortical_consecutive_fire_limit: BurstDelta<Q::BurstDelta>,
                                                 cortical_is_mp_charge_accumulation_enabled: bool,
                                                 cortical_is_mp_driven_psp_enabled: bool)
                                                 -> Result<(CorticalAreaIndex<Q::CorticalIndex>), FeagiNPUNeuronError>;


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn default_create_cortical_area_with_individualized_neurons(&mut self,
                                                                cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                                neurons_per_voxel: NumberNeuronsPerVoxel,
                                                                neuron_data: DimensionalNeuronDataFromCorticalArea<Q>)
                                                                -> Result<(CorticalAreaIndex<Q::CorticalIndex>), FeagiNPUNeuronError>;
}

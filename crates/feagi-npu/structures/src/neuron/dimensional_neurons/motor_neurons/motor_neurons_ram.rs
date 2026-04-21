use core::ops::Range;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use feagi_structures::useful_structs::{IndexedDataTracker, RangeUintVector};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_dimension_traits::{DimensionalAllocStorageTrait, DimensionalStaticStorageTrait};
use crate::neuron::base_traits::{BaseNeuronAllocStorageTrait, BaseNeuronStaticStorageTrait};
use crate::neuron::defaults::{MotorNeuronsDefaults};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataFromCorticalArea, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::dimensional_neurons::dimensional_traits::{DimensionalNeuronAllocStorageTrait, DimensionalNeuronStaticStorageTrait};
use crate::neuron::dimensional_neurons::shared_funcs_ram::{
    create_cortical_area_with_individualized_neurons,
    default_create_cortical_area_with_uniform_neurons,
    get_cortical_area_ref,
    invalidate_cortical_area_and_return_invalidated_neuron_range,
};
use crate::quantizables::{NPUQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability};
// In this implementation, we can do a lot by keeping neurons of a cortical area grouped together, albeit they may not be guaranteed to be in cortical index order

// TODO motor traits
// TODO just copying inter neurons for now, but we should have some sensiomotor / motor specific implementations
pub struct MotorNeuronAllocRAMStorage<Q: NPUQuantization>
{
    // Per Neuron (including invalids)
    neuron_cortical_area_index: Vec<CorticalAreaIndex<Q::CorticalIndex>>, // faster than potentially reverse looking up a large hashmap
    neuron_global_burst_index_of_last_firing: Vec<BurstGlobalIndex<Q::BurstIndex>>,
    neuron_membrane_potential: Vec<NPUNeuronMembranePotential<Q::Value>>,
    neuron_fire_threshold: Vec<FireThreshold<Q::Value>>,
    neuron_leak_coefficient: Vec<LeakCoefficient<Q::Percentage>>,
    neuron_flags: Vec<NeuronFlag>,
    neuron_refractory_countdown: Vec<BurstDelta<Q::BurstDelta>>,
    neuron_consecutive_fire_count: Vec<BurstDelta<Q::BurstDelta>>, // how many times the neuron fired burst recently

    // Per Cortical Area (including invalids)
    cortical_datas: IndexedDataTracker<DimensionalNeuronCorticalData<Q>, CorticalAreaIndex<Q::CorticalIndex>>,

    // Cached Data
    cache_number_valid_neurons: NeuronCount<Q::NeuronIndex>,
    cache_number_invalid_neurons: NeuronCount<Q::NeuronIndex>,
    cache_invalid_neuron_index_blocks: RangeUintVector<NPUNeuronIndex<Q::NeuronIndex>, NeuronCount<Q::NeuronIndex>>,
}

// NOTE: Only define the constructor here, as we will be going through traits / generics for all data transfer!
impl<Q: NPUQuantization>
MotorNeuronAllocRAMStorage<Q>
{
    pub fn new(number_neurons_to_preallocate_space_for: NeuronCount<Q::NeuronIndex>, number_cortical_areas_to_preallocate_space_for: CorticalAreaIndex<Q::CorticalIndex>) -> Self {
        Self {
            neuron_cortical_area_index: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_global_burst_index_of_last_firing: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_membrane_potential: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_fire_threshold: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_leak_coefficient: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_flags: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_refractory_countdown: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_consecutive_fire_count: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),

            cortical_datas: IndexedDataTracker::with_capacity(number_cortical_areas_to_preallocate_space_for),

            cache_number_valid_neurons: NeuronCount::ZERO,
            cache_number_invalid_neurons: NeuronCount::ZERO,
            cache_invalid_neuron_index_blocks: RangeUintVector::new(),
        }
    }
    


}


impl<Q: NPUQuantization>
DimensionalNeuronAllocStorageTrait<Q>
for MotorNeuronAllocRAMStorage<Q>
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
                                                 -> Result<(CorticalAreaIndex<Q::CorticalIndex>), FeagiNPUNeuronError> {
        let (output_cortical_index, extending) = default_create_cortical_area_with_uniform_neurons::<Q, MotorNeuronsDefaults<Q>>(
            cortical_area_dimensions,
            neurons_per_voxel,
            neuron_global_burst_index_of_last_firing,
            neuron_membrane_potential,
            neuron_fire_threshold,
            neuron_leak_coefficient,
            neuron_refractory_countdown,
            neuron_consecutive_fire_count,
            cortical_excitability,
            cortical_refractory_period_limit,
            cortical_fire_threshold_limit,
            cortical_consecutive_fire_limit,
            cortical_is_mp_charge_accumulation_enabled,
            cortical_is_mp_driven_psp_enabled,
            &mut self.neuron_cortical_area_index,
            &mut self.neuron_global_burst_index_of_last_firing,
            &mut self.neuron_membrane_potential,
            &mut self.neuron_fire_threshold,
            &mut self.neuron_leak_coefficient,
            &mut self.neuron_flags,
            &mut self.neuron_refractory_countdown,
            &mut self.neuron_consecutive_fire_count,
            &mut self.cortical_datas,
            &mut self.cache_invalid_neuron_index_blocks,
        )?;

        Ok((output_cortical_index))
    }


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn default_create_cortical_area_with_individualized_neurons(&mut self,
                                                                cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                                neurons_per_voxel: NumberNeuronsPerVoxel,
                                                                neuron_data: DimensionalNeuronDataFromCorticalArea<Q>)
                                                                -> Result<(CorticalAreaIndex<Q::CorticalIndex>), FeagiNPUNeuronError> {
        let (output_cortical_index, is_extending) = create_cortical_area_with_individualized_neurons::<Q, MotorNeuronsDefaults<Q>>(
            cortical_area_dimensions,
            neurons_per_voxel,
            neuron_data,
            &mut self.neuron_cortical_area_index,
            &mut self.neuron_global_burst_index_of_last_firing,
            &mut self.neuron_membrane_potential,
            &mut self.neuron_fire_threshold,
            &mut self.neuron_leak_coefficient,
            &mut self.neuron_flags,
            &mut self.neuron_refractory_countdown,
            &mut self.neuron_consecutive_fire_count,
            &mut self.cortical_datas,
            &mut self.cache_invalid_neuron_index_blocks,
        )?;

        Ok((output_cortical_index))
    }

}


impl<Q: NPUQuantization>
DimensionalNeuronStaticStorageTrait<Q>
for MotorNeuronAllocRAMStorage<Q>
{

    /// Used to pass around slices easily at low cost for all cortical areas
    fn get_neuron_values_of_all_dimensional_neuron_cortical_areas_to_process(&mut self) -> DimensionalNeuronDataRefSliceAllCorticalAreas<'_, Q> {
        DimensionalNeuronDataRefSliceAllCorticalAreas {
            neuron_cortical_area_index: &self.neuron_cortical_area_index,
            neuron_global_burst_index_of_last_firing: &mut self.neuron_global_burst_index_of_last_firing,
            neuron_membrane_potential: &mut self.neuron_membrane_potential,
            neuron_fire_threshold: &mut self.neuron_fire_threshold,
            neuron_leak_coefficient: &mut self.neuron_leak_coefficient,
            neuron_flags: &mut self.neuron_flags,
            neuron_refractory_countdown: &mut self.neuron_refractory_countdown,
            neuron_consecutive_fire_count: &mut self.neuron_consecutive_fire_count,

            cortical_data: &self.cortical_datas,
        }
    }

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: Q::CorticalIndex) -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, Q>, FeagiNPUNeuronError> {
        let cortical_area_index = CorticalAreaIndex(cortical_area_index);
        let cortical_data = get_cortical_area_ref(&cortical_area_index, &self.cortical_datas)?;
        let neuron_range = cortical_data.neuron_range.clone();
        let usize_range: Range<usize> = NPUNeuronIndex::<Q::NeuronIndex>::to_usize_range(neuron_range.clone());

        Ok(DimensionalNeuronDataRefSliceSingleCorticalArea {
            neuron_cortical_area_index: &self.neuron_cortical_area_index[usize_range.clone()],
            neuron_global_burst_index_of_last_firing: &mut self.neuron_global_burst_index_of_last_firing[usize_range.clone()],
            neuron_membrane_potential: &mut self.neuron_membrane_potential[usize_range.clone()],
            neuron_fire_threshold: &mut self.neuron_fire_threshold[usize_range.clone()],
            neuron_leak_coefficient: &mut self.neuron_leak_coefficient[usize_range.clone()],
            neuron_flags: &mut self.neuron_flags[usize_range.clone()],
            neuron_refractory_countdown: &mut self.neuron_refractory_countdown[usize_range.clone()],
            neuron_consecutive_fire_count: &mut self.neuron_consecutive_fire_count[usize_range],

            cortical_data,
            global_neuron_index_range: neuron_range
        })
    }

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: Q::CorticalIndex, executor: &impl NeuronFireThresholdExecutor<Q::Value, Q::Coord>) -> Result<(), FeagiNPUNeuronError> {
        let cortical_area_index = CorticalAreaIndex(cortical_area_index);
        let (usize_range, dimensions, neurons_per_voxel) = {
            let cortical_data = get_cortical_area_ref(&cortical_area_index, &self.cortical_datas)?;
            (
                NPUNeuronIndex::<Q::NeuronIndex>::to_usize_range(cortical_data.neuron_range.clone()),
                cortical_data.dimensions,
                cortical_data.number_neurons_per_voxel,
            )
        };
        executor.set_new_fire_thresholds(
            &mut self.neuron_fire_threshold[usize_range.clone()],
            &self.neuron_flags[usize_range],
            &dimensions,
            neurons_per_voxel,
        )
    }
}


impl<Q: NPUQuantization>
DimensionalAllocStorageTrait<Q>
for MotorNeuronAllocRAMStorage<Q>
{
    /// Creates a cortical area of given dimensions and neuron density,
    /// and returns its cortical area index and range of neuron indexes it covers
    fn create_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel)
                                                 -> Result<(CorticalAreaIndex<Q::CorticalIndex>), FeagiNPUNeuronError>
    {
        self.create_cortical_area_with_uniform_neurons(
            cortical_area_dimensions,
            neurons_per_voxel,
            MotorNeuronsDefaults::<Q>::DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING,
            MotorNeuronsDefaults::<Q>::DEFAULT_NEURON_MEMBRANE_POTENTIAL,
            MotorNeuronsDefaults::<Q>::DEFAULT_NEURON_FIRE_THRESHOLD,
            MotorNeuronsDefaults::<Q>::DEFAULT_NEURON_LEAK_COEFFICIENT,
            MotorNeuronsDefaults::<Q>::DEFAULT_NEURON_REFRACTORY_COUNTDOWN,
            MotorNeuronsDefaults::<Q>::DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT,

            MotorNeuronsDefaults::<Q>::DEFAULT_CORTICAL_EXCITABILITY,
            MotorNeuronsDefaults::<Q>::DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT,
            MotorNeuronsDefaults::<Q>::DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT,
            MotorNeuronsDefaults::<Q>::DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT,
            MotorNeuronsDefaults::<Q>::DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED,
            MotorNeuronsDefaults::<Q>::DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED,
        )
    }



    /// Effectively deletes a cortical area (by invalidating their neurons), then rebuilds it to the
    /// new given dimensions and density. While cortical properties are preserved, neuron data is
    /// reset to default. Returns a tuple of the old invalid neuron index range, and the new
    /// created neuron index range.
    /// WARNING: BE SURE TO MANAGE SYNAPSE MAPPINGS!
    fn resize_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 cortical_index: CorticalAreaIndex<Q::CorticalIndex>)
                                                 -> Result<(Range<NPUNeuronIndex<Q::NeuronIndex>>), FeagiNPUNeuronError> {
        
        // TODO broken!
        todo!()

        /*
        // no need to verify cortical index since the delete function handles that for us
        let deleted_indexes = self.delete_cortical_area(cortical_index)?;
        // TODO This is currently broken due to different indexing system!
        // TODO best to make an explicit system instead! We should be able to have a shared function here
        let new_indexes = self.create_cortical_area_with_default_neurons(cortical_area_dimensions, neurons_per_voxel)?;
        Ok((deleted_indexes, new_indexes.1))
        
         */
    }
}


impl<Q: NPUQuantization>
DimensionalStaticStorageTrait<Q>
for MotorNeuronAllocRAMStorage<Q>
{

}


impl<Q: NPUQuantization>
BaseNeuronAllocStorageTrait<Q>
for MotorNeuronAllocRAMStorage<Q>
{

    /// Frees unused neuron vector capacity and invalid neurons (assuming they were sorted to the back first!)
    /// albeit allowing a buffer of free space. Returns the number of neurons that were freed.
    /// Returns 0 if no neurons were freed (nothing to free or spare capacity is at or less than
    /// what was requested). Note that invalid neurons not sorted to the back will not be freed.
    fn free_unused_neuron_capacity(&mut self, _spare_capacity_to_maintain: NeuronCount<Q::NeuronIndex>) -> NeuronCount<Q::NeuronIndex> {
        todo!()
    }

    /// Deletes a cortical area by invalidating all of its neurons. Returns the neuron indexes
    /// of the disabled neurons
    /// WARNING: BE SURE TO REMOVE ASSOCIATED SYNAPSE MAPPINGS!
    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndex>)
                            -> Result<Range<NPUNeuronIndex<Q::NeuronIndex>>, FeagiNPUNeuronError> {
        invalidate_cortical_area_and_return_invalidated_neuron_range(
            &cortical_index,
            &mut self.cortical_datas,
            &mut self.neuron_flags,
            &mut self.cache_number_valid_neurons,
            &mut self.cache_number_invalid_neurons,
            &mut self.cache_invalid_neuron_index_blocks
        )
    }




}


impl<Q: NPUQuantization>
BaseNeuronStaticStorageTrait<Q>
for MotorNeuronAllocRAMStorage<Q>
{
    const NUMBER_BYTES_PER_NEURON: usize = 0; // TODO

    /// Gets the maximum possible neuron index achievable by current quantization (or in the case
    /// of static implementations, the size of the neuron array).
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<Q::NeuronIndex> {
        NPUNeuronIndex::MAX_VALUE
    }

    /// Returns the count of valid neurons in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// NEURONS STORED!
    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<Q::NeuronIndex> {
        self.cache_number_valid_neurons
    }


    /// Returns the count of invalid neurons in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<Q::NeuronIndex> {
        self.cache_number_invalid_neurons
    }


    fn get_max_possible_cortical_area_index(&self) -> CorticalAreaIndex<Q::CorticalIndex> {
        CorticalAreaIndex::MAX_VALUE
    }
}

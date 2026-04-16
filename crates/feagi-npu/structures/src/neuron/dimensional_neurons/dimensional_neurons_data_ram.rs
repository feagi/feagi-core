use core::ops::Range;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use feagi_structures::useful_structs::{InvalidatableVector, RangeUintVector};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_dimension_traits::{DimensionalAllocStorageTrait, DimensionalStaticStorageTrait};
use crate::neuron::base_traits::{BaseNeuronAllocStorageTrait, BaseNeuronStaticStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataFromCorticalArea, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::dimensional_neurons::dimensional_traits::{DimensionalNeuronAllocStorageTrait, DimensionalNeuronStaticStorageTrait};
use crate::neuron::dimensional_neurons::shared_funcs_ram::{get_cortical_area_ref, invalidate_cortical_area_and_return_invalidated_neuron_range};
use crate::quantizables::{NPUQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability};
// In this implementation, we can do a lot by keeping neurons of a cortical area grouped together, albeit they may not be guaranteed to be in cortical index order


pub struct DimensionalNeuronAllocRAMStorage<Q: NPUQuantization>
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
    cortical_datas: InvalidatableVector<DimensionalNeuronCorticalData<Q>>,

    // Cached Data
    cache_number_valid_neurons: NeuronCount<Q::NeuronIndex>,
    cache_number_invalid_neurons: NeuronCount<Q::NeuronIndex>,
    cache_invalid_neuron_index_blocks: RangeUintVector<NPUNeuronIndex<Q::NeuronIndex>>,
}

// NOTE: Only define the constructor here, as we will be going through traits / generics for all data transfer!
impl<Q: NPUQuantization>
DimensionalNeuronAllocRAMStorage<Q>
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

                cortical_datas: InvalidatableVector::with_capacity(number_cortical_areas_to_preallocate_space_for.to_usize()),

                cache_number_valid_neurons: NeuronCount::ZERO,
                cache_number_invalid_neurons: NeuronCount::ZERO,
                cache_invalid_neuron_index_blocks: RangeUintVector::new(),
            }
    }



}


impl<Q: NPUQuantization>
DimensionalNeuronAllocStorageTrait<Q>
for DimensionalNeuronAllocRAMStorage<Q>
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
                                                 -> Result<(CorticalAreaIndex<Q::CorticalIndex>, Range<NPUNeuronIndex<Q::NeuronIndex>>), FeagiNPUNeuronError> {

        // NOTE: for now neuron flag only checks for validity, so we dont need that parameter.
        let number_of_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
        let neuron_writing_region = self.cache_invalid_neuron_index_blocks.find_space(NPUNeuronIndex::from_usize(number_of_neurons)); // TODO incorrect type! Shouldnt this take in count?

        let mut cortical_flags: DimensionalNeuronCorticalFlag = DimensionalNeuronCorticalFlag::new_valid();
        cortical_flags.set_mp_charge_accumulation_enabled(cortical_is_mp_charge_accumulation_enabled);
        cortical_flags.set_mp_driven_psp_enabled(cortical_is_mp_driven_psp_enabled);

        let mut neuron_flag = NeuronFlag::new_valid();

        let output_cortical_index;
        let output_neuron_region;



        // TODO use par iter on massive arrays!


        if neuron_writing_region.is_none() {
            // No space, Allocate
            let neuron_writing_region = NPUNeuronIndex::from_usize(self.neuron_flags.len()) .. NPUNeuronIndex::from_usize(self.neuron_flags.len() + number_of_neurons);
            let cortical_data = DimensionalNeuronCorticalData {
                flags: cortical_flags,
                neuron_range: neuron_writing_region.clone(),
                number_neurons_invalid_from_degeneration: NeuronCount::ZERO, // no neurons assumed dead yet
                dimensions: cortical_area_dimensions,
                number_neurons_per_voxel: neurons_per_voxel,
                excitability: cortical_excitability,
                refractory_period_limit: cortical_refractory_period_limit,
                fire_threshold_limit: cortical_fire_threshold_limit,
                consecutive_fire_limit: cortical_consecutive_fire_limit,
            };

            let cortical_index =  CorticalAreaIndex::from_usize(self.cortical_datas.push(cortical_data));

            output_cortical_index = cortical_index;
            output_neuron_region = neuron_writing_region;

            self.neuron_cortical_area_index.extend(std::iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing.extend(std::iter::repeat_n(neuron_global_burst_index_of_last_firing, number_of_neurons));
            self.neuron_membrane_potential.extend(std::iter::repeat_n(neuron_membrane_potential, number_of_neurons));
            self.neuron_fire_threshold.extend(std::iter::repeat_n(neuron_fire_threshold, number_of_neurons));
            self.neuron_leak_coefficient.extend(std::iter::repeat_n(neuron_leak_coefficient, number_of_neurons));
            self.neuron_flags.extend(std::iter::repeat_n(neuron_flag, number_of_neurons));
            self.neuron_refractory_countdown.extend(std::iter::repeat_n(neuron_refractory_countdown, number_of_neurons));
            self.neuron_consecutive_fire_count.extend(std::iter::repeat_n(neuron_consecutive_fire_count, number_of_neurons));


        } else {
            // We have space, lets overwrite
            let neuron_writing_region = neuron_writing_region.unwrap();
            let cortical_data = DimensionalNeuronCorticalData {
                flags: cortical_flags,
                neuron_range: neuron_writing_region.clone(),
                number_neurons_invalid_from_degeneration: NeuronCount::ZERO, // no neurons assumed dead yet
                dimensions: cortical_area_dimensions,
                number_neurons_per_voxel: neurons_per_voxel,
                excitability: cortical_excitability,
                refractory_period_limit: cortical_refractory_period_limit,
                fire_threshold_limit: cortical_fire_threshold_limit,
                consecutive_fire_limit: cortical_consecutive_fire_limit,
            };
            let cortical_index =  CorticalAreaIndex::from_usize(self.cortical_datas.push(cortical_data));
            self.neuron_cortical_area_index[&neuron_writing_region].fill(std::iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing[&neuron_writing_region].fill(std::iter::repeat_n(neuron_global_burst_index_of_last_firing, number_of_neurons));
            self.neuron_membrane_potential[&neuron_writing_region].fill(std::iter::repeat_n(neuron_membrane_potential, number_of_neurons));
            self.neuron_fire_threshold[&neuron_writing_region].fill(std::iter::repeat_n(neuron_fire_threshold, number_of_neurons));
            self.neuron_leak_coefficient[&neuron_writing_region].fill(std::iter::repeat_n(neuron_leak_coefficient, number_of_neurons));
            self.neuron_flags[&neuron_writing_region].fill(std::iter::repeat_n(neuron_flag, number_of_neurons));
            self.neuron_refractory_countdown[&neuron_writing_region].fill(std::iter::repeat_n(neuron_refractory_countdown, number_of_neurons));
            self.neuron_consecutive_fire_count[&neuron_writing_region].fill(std::iter::repeat_n(neuron_consecutive_fire_count, number_of_neurons));

            output_cortical_index = cortical_index;
            output_neuron_region = neuron_writing_region;

        }


        return Ok((output_cortical_index, output_neuron_region))
    }


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_individualized_neurons(&mut self,
                                                        cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                        neurons_per_voxel: NumberNeuronsPerVoxel,
                                                        neuron_data: DimensionalNeuronDataFromCorticalArea<Q>)
                                                        -> Result<(CorticalAreaIndex<Q::CorticalIndex>, Range<NPUNeuronIndex<Q::NeuronIndex>>), FeagiNPUNeuronError> {

        // Find where to write neuron data
        let number_of_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
        let neuron_writing_region = self.cache_invalid_neuron_index_blocks.find_space(NPUNeuronIndex::from_usize(number_of_neurons)); // TODO incorrect type! Shouldnt this take in count?

        let output_cortical_index;
        let output_neuron_region;


        if neuron_writing_region.is_none() {
            // No space, Allocate
            let neuron_writing_region = NPUNeuronIndex::from_usize(self.neuron_flags.len()) .. NPUNeuronIndex::from_usize(self.neuron_flags.len() + number_of_neurons);
            let cortical_data = DimensionalNeuronCorticalData::new_default_valid(neuron_writing_region.clone(), cortical_area_dimensions, neurons_per_voxel);
            let cortical_index =  CorticalAreaIndex::from_usize(self.cortical_datas.push(cortical_data));
            self.neuron_cortical_area_index.extend(std::iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing.extend(neuron_data.neuron_global_burst_index_of_last_firing);
            self.neuron_membrane_potential.extend(neuron_data.neuron_membrane_potential);
            self.neuron_fire_threshold.extend(neuron_data.neuron_fire_threshold);
            self.neuron_leak_coefficient.extend(neuron_data.neuron_leak_coefficient);
            self.neuron_flags.extend(neuron_data.neuron_flags);
            self.neuron_refractory_countdown.extend(neuron_data.neuron_refractory_countdown);
            self.neuron_consecutive_fire_count.extend(neuron_data.neuron_consecutive_fire_count);

            output_cortical_index = cortical_index;
            output_neuron_region = neuron_writing_region;

        } else {
            // We have space, lets overwrite
            let neuron_writing_region = neuron_writing_region.unwrap();
            let cortical_data = DimensionalNeuronCorticalData::new_default_valid(neuron_writing_region.clone(), cortical_area_dimensions, neurons_per_voxel);
            let cortical_index =  CorticalAreaIndex::from_usize(self.cortical_datas.push(cortical_data));
            self.neuron_cortical_area_index[neuron_writing_region].fill(std::iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing[neuron_writing_region].copy_from_slice(&neuron_data.neuron_global_burst_index_of_last_firing);
            self.neuron_membrane_potential[neuron_writing_region].copy_from_slice(&neuron_data.neuron_membrane_potential);
            self.neuron_fire_threshold[neuron_writing_region].copy_from_slice(&neuron_data.neuron_fire_threshold);
            self.neuron_leak_coefficient[neuron_writing_region].copy_from_slice(&neuron_data.neuron_leak_coefficient);
            self.neuron_flags[neuron_writing_region].copy_from_slice(&neuron_data.neuron_flags);
            self.neuron_refractory_countdown[neuron_writing_region].copy_from_slice(&neuron_data.neuron_refractory_countdown);
            self.neuron_consecutive_fire_count[neuron_writing_region].copy_from_slice(&neuron_data.neuron_consecutive_fire_count);

            output_cortical_index = cortical_index;
            output_neuron_region = neuron_writing_region;

        }
        return Ok((output_cortical_index, output_neuron_region))
    }

}


impl<Q: NPUQuantization>
DimensionalNeuronStaticStorageTrait<Q>
for DimensionalNeuronAllocRAMStorage<Q>
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
    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: &CorticalAreaIndex<Q::CorticalIndex>) -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, Q>, FeagiNPUNeuronError> {
        let cortical_data = get_cortical_area_ref(&cortical_area_index, &self.cortical_datas)?;
        let neuron_range = cortical_data.neuron_range.clone();

        Ok(DimensionalNeuronDataRefSliceSingleCorticalArea {
            neuron_cortical_area_index: &self.neuron_cortical_area_index[neuron_range],
            neuron_global_burst_index_of_last_firing: &mut self.neuron_global_burst_index_of_last_firing[neuron_range],
            neuron_membrane_potential: &mut self.neuron_membrane_potential[neuron_range],
            neuron_fire_threshold: &mut self.neuron_fire_threshold[neuron_range],
            neuron_leak_coefficient: &mut self.neuron_leak_coefficient[neuron_range],
            neuron_flags: &mut self.neuron_flags[neuron_range],
            neuron_refractory_countdown: &mut self.neuron_refractory_countdown[neuron_range],
            neuron_consecutive_fire_count: &mut self.neuron_consecutive_fire_count[neuron_range],

            cortical_data: cortical_data,
            global_neuron_index_range: neuron_range
        })
    }

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: Q::CorticalIndex, executor: &impl NeuronFireThresholdExecutor<Q::Value, Q::Coord>) -> Result<(), FeagiNPUNeuronError> {
        let cortical_data = self.get_cortical_data_ref(cortical_area_index)?;
        executor.set_new_fire_thresholds(
            &mut self.neuron_fire_threshold[&cortical_data.neuron_range],
            &self.neuron_flags[&cortical_data.neuron_range],
            &cortical_data.dimensions
        )
    }
}


impl<Q: NPUQuantization>
DimensionalAllocStorageTrait<Q>
for DimensionalNeuronAllocRAMStorage<Q>
{
    /// Creates a cortical area of given dimensions and neuron density,
    /// and returns its cortical area index and range of neuron indexes it covers
    fn create_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel)
                                                 -> Result<(CorticalAreaIndex<Q::CorticalIndex>, Range<NPUNeuronIndex<Q::NeuronIndex>>), FeagiNPUNeuronError> 
    {

        let expected_number_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
        self.create_cortical_area_with_uniform_neurons(
            cortical_area_dimensions,
            neurons_per_voxel,
            DimensionalNeuronStaticStorageTrait::DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING,
            DimensionalNeuronStaticStorageTrait::DEFAULT_NEURON_MEMBRANE_POTENTIAL,
            DimensionalNeuronStaticStorageTrait::DEFAULT_NEURON_FIRE_THRESHOLD,
            DimensionalNeuronStaticStorageTrait::DEFAULT_NEURON_LEAK_COEFFICIENT,
            DimensionalNeuronStaticStorageTrait::DEFAULT_NEURON_REFRACTORY_COUNTDOWN,
            DimensionalNeuronStaticStorageTrait::DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT,

            DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_EXCITABILITY,
            DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT,
            DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT,
            DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT,
            DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED,
            DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED,
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
                                                 -> Result<(Range<NPUNeuronIndex<Q::NeuronIndex>>, Range<NPUNeuronIndex<Q::NeuronIndex>>), FeagiNPUNeuronError> {

        // no need to verify cortical index since the delete function handles that for us
        let deleted_indexes = self.delete_cortical_area(cortical_index)?;
        // RISKY: We know in current implementation, a deleted cortical index immediately goes back
        // to the available pool, so we *should* get the same one back, right?
        // TODO best to make an explicit system instead! We should be able to have a shared function here
        let new_indexes = self.create_cortical_area_with_default_neurons(cortical_area_dimensions, neurons_per_voxel)?;
        return Ok((deleted_indexes, new_indexes.1));
    }
}


impl<Q: NPUQuantization>
DimensionalStaticStorageTrait<Q>
for DimensionalNeuronAllocRAMStorage<Q>
{

}


impl<Q: NPUQuantization>
BaseNeuronAllocStorageTrait<Q>
for DimensionalNeuronAllocRAMStorage<Q>
{

    /// Frees unused neuron vector capacity and invalid neurons (assuming they were sorted to the back first!)
    /// albeit allowing a buffer of free space. Returns the number of neurons that were freed.
    /// Returns 0 if no neurons were freed (nothing to free or spare capacity is at or less than
    /// what was requested). Note that invalid neurons not sorted to the back will not be freed.
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<Q::NeuronIndex>) -> NeuronCount<Q::NeuronIndex> {
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
for DimensionalNeuronAllocRAMStorage<Q>
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

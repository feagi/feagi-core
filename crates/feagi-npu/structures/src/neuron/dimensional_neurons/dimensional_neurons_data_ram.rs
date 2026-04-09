use core::ops::Range;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_dimension_traits::{DimensionalAllocStorageTrait, DimensionalStaticStorageTrait};
use crate::neuron::base_traits::{BaseNeuronAllocStorageTrait, BaseNeuronStaticStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::neuron::dimensional_neurons::shared_funcs_and_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataFromCorticalArea, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::dimensional_neurons::traits::{DimensionalNeuronAllocStorageTrait, DimensionalNeuronStaticStorageTrait};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability};
// In this implementation, we can do a lot by keeping neurons of a cortical area grouped together, albeit they may not be guaranteed to be in cortical index order


pub struct DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    // Per Neuron (including invalids)
    neuron_cortical_area_index: Vec<CorticalAreaIndex<CorticalIndexQuant>>, // faster than potentially reverse looking up a large hashmap
    neuron_global_burst_index_of_last_firing: Vec<BurstGlobalIndex<BurstIndexQuant>>,
    neuron_membrane_potential: Vec<NeuronMembranePotential<ValueQuant>>,
    neuron_fire_threshold: Vec<FireThreshold<ValueQuant>>,
    neuron_leak_coefficient: Vec<LeakCoefficient<PercentageQuant>>,
    neuron_flags: Vec<NeuronFlag>,
    neuron_refractory_countdown: Vec<BurstDelta<BurstDeltaQuant>>,
    neuron_consecutive_fire_count: Vec<BurstDelta<BurstDeltaQuant>>, // how many times the neuron fired burst recently

    // Per Cortical Area (including invalids)
    cortical_data: Vec<DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, ValueQuant, PercentageQuant>>,

    // Cached Data
    cache_number_valid_neurons: NeuronCount<NeuronIndexQuant>,
    cache_number_invalid_neurons: NeuronCount<NeuronIndexQuant>,
    cache_index_to_write_new_neurons: NPUNeuronIndex<NeuronIndexQuant>, // Index starting where new neurons will be written to
    cache_skipped_cortical_indexes: Vec<CorticalAreaIndex<CorticalIndexQuant>>, // when a cortical area is removed, put the index here, these will be the first given out
    cache_invalid_neuron_indexes: Vec<Range<NPUNeuronIndex<NeuronIndexQuant>>>,
}

// NOTE: Only define the constructor here, as we will be going through traits / generics for all data transfer!
impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    pub fn new(number_neurons_to_preallocate_space_for: NeuronCount<NeuronIndexQuant>, number_cortical_areas_to_preallocate_space_for: CorticalAreaIndex<CorticalIndexQuant>) -> Self {
            Self {
                neuron_cortical_area_index: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
                neuron_global_burst_index_of_last_firing: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
                neuron_membrane_potential: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
                neuron_fire_threshold: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
                neuron_leak_coefficient: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
                neuron_flags: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
                neuron_refractory_countdown: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
                neuron_consecutive_fire_count: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),

                cortical_data: Vec::with_capacity(number_cortical_areas_to_preallocate_space_for.to_usize()),

                cache_number_valid_neurons: NeuronIndexQuant::ZERO,
                cache_number_invalid_neurons: NeuronIndexQuant::ZERO,
                cache_index_to_write_new_neurons: NeuronIndexQuant::ZERO,
                cache_skipped_cortical_indexes: Vec::new(),
                cache_invalid_neuron_indexes: Vec::new(),
            }
    }


    //region Internal Helper Functions

    /// Marks the neurons of a cortical area as invalid, as well as other cache work in this regard.
    /// Returns the range of neuron indexes invalidated.
    fn invalidate_cortical_area(&mut self, cortical_area_index: CorticalIndexQuant) -> Result<Range<NeuronIndexQuant>, FeagiNPUNeuronError> {
        // These basic checks are fast and we arent iterating over cortical areas THAT fast, right? // TODO shove checks in a debug?

        let cortical_data = self.get_cortical_data_ref_mut(&cortical_area_index)?;

        if !cortical_data.flags.is_valid() {
            return Err(FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "Unable to invalidate given internueron cortical index as it is marked as invalid!",
                given_cortical_index: cortical_area_index.to_usize() as u32
            });
        }

        cortical_data.flags.toggle_valid();
        
        let number_of_neurons: NeuronCount<NeuronIndexQuant> =  NeuronIndexQuant::from_usize({
            // TODO (debug?) check for validity of range

            let neuron_flag_slice: &mut[NeuronFlag] = self.neuron_flags[cortical_data.neuron_range];

            // so, since we actually do not care for any other flag in the neuron data except for
            // the is valid flag being set to false, just mass fill the area with the bitpack containing
            // that setting

            let invalid_flag = NeuronFlag::ALL_ZEROS;

            // TODO look into iterator / par iterator fills
            neuron_flag_slice.fill(invalid_flag);
            neuron_flag_slice.len()
        });

        // Some neurons may have died on their own
        let number_of_neurons_invalidated = number_of_neurons - cortical_data.number_neurons_invalid_from_degeneration;

        // Mark neurons as dead in the cache too
        self.cache_number_valid_neurons -= number_of_neurons_invalidated;
        self.cache_number_invalid_neurons += number_of_neurons_invalidated;
        self.cache_invalid_neuron_indexes.push(cortical_data.neuron_range.clone()); // TODO maybe we should have a smarter insert? in the case of connecting segments, make them one bigger segment instead

        // Mark this cortical index as free
        self.cache_skipped_cortical_indexes.push(cortical_area_index);
        cortical_data.flags.set_valid(false);

        Ok(cortical_data.neuron_range.clone())
    }

    /// Adds cortical data to the next available cortical area slot (either at the end or in the middle if available. Returns the cortical ID used
    fn add_cortical_data_to_next_available_cortical_area_index(&mut self, new_cortical_data: DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>) -> Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUNeuronError> {
        // TODO Extreme edge case error, when we hit quant limit
        let mut cortical_index: CorticalAreaIndex<CorticalIndexQuant>;
        if &self.cache_skipped_cortical_indexes.is_empty() {
            cortical_index = CorticalIndexQuant::from_usize(self.cortical_data.len());
            self.cortical_data.push(new_cortical_data);
        }
        else {
            cortical_index  = self.cache_skipped_cortical_indexes.pop().unwrap();
            // TODO DEBUG: ensure we arent overwriting a valid cortical area!
            self.cortical_data[cortical_index.to_usize()] = new_cortical_data;
        }
        Ok(cortical_index)
    }

    /// Returns an empty result if a cortical area exists AND is valid. Otherwise errors.
    fn verify_cortical_area_index_exist_and_valid(&self, cortical_area_index: &CorticalAreaIndex<CorticalIndexQuant>) -> Result<(), FeagiNPUNeuronError> {
        let reference = self.get_cortical_data_ref(cortical_area_index)?;
        if reference.flags.is_valid() {
            return Ok(())
        }
        Err(FeagiNPUNeuronError::InvalidCorticalIndex{
            context: "Requested Cortical Area Index exists but is not valid!",
            given_cortical_index: cortical_area_index as u32
        })
    }

    /// Get the cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
    fn get_cortical_data_ref(&self, cortical_area_index: &CorticalAreaIndex<CorticalIndexQuant>) -> Result<&DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>, FeagiNPUNeuronError> {
        Ok(self.cortical_data.get(cortical_area_index.to_usize())
            .ok_or_else(|| FeagiNPUNeuronError::InvalidCorticalIndex{
                context: "Requested Cortical Area Index does not exist!",
                given_cortical_index: cortical_area_index as u32
            })?)
    }

    /// Get the mutable cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
    fn get_cortical_data_ref_mut(&mut self, cortical_area_index: &CorticalAreaIndex<CorticalIndexQuant>) -> Result<&mut DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>, FeagiNPUNeuronError> {
        Ok(self.cortical_data.get_mut(cortical_area_index.to_usize())
            .ok_or_else(|| FeagiNPUNeuronError::InvalidCorticalIndex{
                context: "Requested Cortical Area Index does not exist!",
                given_cortical_index: cortical_area_index as u32
            })?)
    }

    //endregion

}


impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
DimensionalNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
for DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    /// Creates a cortical area of given dimensions but using a set of neuron values copied across
    /// all neurons.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_uniform_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 neuron_global_burst_index_of_last_firing: BurstGlobalIndex<BurstIndexQuant>,
                                                 neuron_membrane_potential: NeuronMembranePotential<PotentialQuant>,
                                                 neuron_fire_threshold: FireThreshold<PotentialQuant>,
                                                 neuron_leak_coefficient: LeakCoefficient<PercentageQuant>,
                                                 neuron_refractory_countdown: BurstDelta<BurstDeltaQuant>,
                                                 neuron_consecutive_fire_count: BurstDelta<BurstDeltaQuant>,
                                                 cortical_excitability: NeuronExcitability<PercentageQuant>,
                                                 cortical_refractory_period_limit: BurstDelta<BurstDeltaQuant>,
                                                 cortical_fire_threshold_limit: FireThresholdLimit<PotentialQuant>,
                                                 cortical_consecutive_fire_limit: BurstDelta<BurstDeltaQuant>,
                                                 cortical_is_mp_charge_accumulation_enabled: bool,
                                                 cortical_is_mp_driven_psp_enabled: bool)
                                                 -> Result<(CorticalAreaIndex<CorticalIndexQuant>, Range<NPUNeuronIndex<NeuronIndexQuant>>), FeagiNPUNeuronError> {

        // NOTE: for now neuron flag only checks for validity, so we dont need that parameter.
        let neuron_flag = NeuronFlag::new_valid();

        // TODO debug: check against allocation with invalid neuron flag


        // Find where to write neuron data
        let number_of_neurons = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
        let (neuron_index_range, is_allocation_at_end_needed): (Range<NPUNeuronIndex<NeuronIndexQuant>>, bool) = {
            // TODO instead of allocating right to the end, what if we have a way to quickly check through cache_invalid_neuron_indexes (assuming we also group neighboring ranges) and put ourselves there if we fit?
            //if self.cache_number_invalid_neurons.to_usize() > number_of_neurons {
            //
            //}
            // TODO size checks (not debug only, we need to be careful)
            let start = self.cache_index_to_write_new_neurons.clone();
            self.cache_index_to_write_new_neurons += NPUNeuronIndex::from_usize(number_of_neurons); // increment new index
            (start..(start + NPUNeuronIndex::from_usize(number_of_neurons)), true)
        };

        // Create and write cortical data

        let mut cortical_flags: DimensionalNeuronCorticalFlag = DimensionalNeuronCorticalFlag::new_valid();
        cortical_flags.set_mp_charge_accumulation_enabled(cortical_is_mp_charge_accumulation_enabled);
        cortical_flags.set_mp_driven_psp_enabled(cortical_is_mp_driven_psp_enabled);

        let cortical_data = DimensionalNeuronCorticalData {
            flags: cortical_flags,
            neuron_range: neuron_index_range.clone(),
            number_neurons_invalid_from_degeneration: NeuronIndexQuant::ZERO, // no neurons assumed dead yet
            dimensions: cortical_area_dimensions,
            number_neurons_per_voxel: neurons_per_voxel,
            excitability: cortical_excitability,
            refractory_period_limit: cortical_refractory_period_limit,
            fire_threshold_limit: cortical_fire_threshold_limit,
            consecutive_fire_limit: cortical_consecutive_fire_limit,
        };

        let cortical_index: CorticalIndexQuant = self.add_cortical_data_to_next_available_cortical_area_index(cortical_data)?;

        // TODO use par iter on massive arrays!
        
        // Actually allocate if needed, otherwise write to existing memory
        if is_allocation_at_end_needed {
            self.neuron_cortical_area_index.extend(std::iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing.extend(std::iter::repeat_n(neuron_global_burst_index_of_last_firing, number_of_neurons));
            self.neuron_membrane_potential.extend(std::iter::repeat_n(neuron_membrane_potential, number_of_neurons));
            self.neuron_fire_threshold.extend(std::iter::repeat_n(neuron_fire_threshold, number_of_neurons));
            self.neuron_leak_coefficient.extend(std::iter::repeat_n(neuron_leak_coefficient, number_of_neurons));
            self.neuron_flags.extend(std::iter::repeat_n(neuron_flag, number_of_neurons));
            self.neuron_refractory_countdown.extend(std::iter::repeat_n(neuron_refractory_countdown, number_of_neurons));
            self.neuron_consecutive_fire_count.extend(std::iter::repeat_n(neuron_consecutive_fire_count, number_of_neurons));
        }
        else {
            self.neuron_cortical_area_index[&neuron_index_range].fill(std::iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing[&neuron_index_range].fill(std::iter::repeat_n(neuron_global_burst_index_of_last_firing, number_of_neurons));
            self.neuron_membrane_potential[&neuron_index_range].fill(std::iter::repeat_n(neuron_membrane_potential, number_of_neurons));
            self.neuron_fire_threshold[&neuron_index_range].fill(std::iter::repeat_n(neuron_fire_threshold, number_of_neurons));
            self.neuron_leak_coefficient[&neuron_index_range].fill(std::iter::repeat_n(neuron_leak_coefficient, number_of_neurons));
            self.neuron_flags[&neuron_index_range].fill(std::iter::repeat_n(neuron_flag, number_of_neurons));
            self.neuron_refractory_countdown[&neuron_index_range].fill(std::iter::repeat_n(neuron_refractory_countdown, number_of_neurons));
            self.neuron_consecutive_fire_count[&neuron_index_range].fill(std::iter::repeat_n(neuron_consecutive_fire_count, number_of_neurons));
        }

        Ok((cortical_index, neuron_index_range))
    }


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_individualized_neurons(&mut self,
                                                        cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                        neurons_per_voxel: NumberNeuronsPerVoxel,
                                                        neuron_data: DimensionalNeuronDataFromCorticalArea<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>)
                                                        -> Result<(CorticalAreaIndex<CorticalIndexQuant>, Range<NPUNeuronIndex<NeuronIndexQuant>>), FeagiNPUNeuronError> {

        // Find where to write neuron data
        let number_of_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
        let (neuron_index_range, is_allocation_at_end_needed): (Range<NeuronIndexQuant>, bool) = {
            // TODO instead of allocating right to the end, what if we have a way to quickly check through cache_invalid_neuron_indexes (assuming we also group neighboring ranges) and put ourselves there if we fit?
            //if self.cache_number_invalid_neurons.to_usize() > number_of_neurons {
            //
            //}
            // TODO size checks (not debug only, we need to be careful)
            let start = self.cache_index_to_write_new_neurons.clone();
            self.cache_index_to_write_new_neurons += NeuronIndexQuant::from_usize(number_of_neurons); // increment new index
            (start..(start + NeuronIndexQuant::from_usize(number_of_neurons)), true)
        };

        let cortical_area_data= DimensionalNeuronCorticalData::new_default_valid(
            neuron_index_range,
            cortical_area_dimensions,
            neurons_per_voxel,
        );

        let cortical_index: CorticalIndexQuant = self.add_cortical_data_to_next_available_cortical_area_index(cortical_area_data)?;

        // TODO use par iter on massive arrays!

        // Actually allocate if needed, otherwise write to existing memory
        if is_allocation_at_end_needed {
            self.neuron_cortical_area_index.extend(std::iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing.extend(neuron_data.neuron_global_burst_index_of_last_firing);
            self.neuron_membrane_potential.extend(neuron_data.neuron_membrane_potential);
            self.neuron_fire_threshold.extend(neuron_data.neuron_fire_threshold);
            self.neuron_leak_coefficient.extend(neuron_data.neuron_leak_coefficient);
            self.neuron_flags.extend(neuron_data.neuron_flags);
            self.neuron_refractory_countdown.extend(neuron_data.neuron_refractory_countdown);
            self.neuron_consecutive_fire_count.extend(neuron_data.neuron_consecutive_fire_count);
        }
        else {
            self.neuron_cortical_area_index[neuron_index_range].fill(std::iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing[neuron_index_range].copy_from_slice(&neuron_data.neuron_global_burst_index_of_last_firing);
            self.neuron_membrane_potential[neuron_index_range].copy_from_slice(&neuron_data.neuron_membrane_potential);
            self.neuron_fire_threshold[neuron_index_range].copy_from_slice(&neuron_data.neuron_fire_threshold);
            self.neuron_leak_coefficient[neuron_index_range].copy_from_slice(&neuron_data.neuron_leak_coefficient);
            self.neuron_flags[neuron_index_range].copy_from_slice(&neuron_data.neuron_flags);
            self.neuron_refractory_countdown[neuron_index_range].copy_from_slice(&neuron_data.neuron_refractory_countdown);
            self.neuron_consecutive_fire_count[neuron_index_range].copy_from_slice(&neuron_data.neuron_consecutive_fire_count);
        }
        return Ok((cortical_index, neuron_index_range))
    }

}


impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
DimensionalNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
for DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    /// Used to pass around slices easily at low cost for all cortical areas
    fn get_neuron_values_of_all_dimensional_neuron_cortical_areas_to_process(&mut self) -> DimensionalNeuronDataRefSliceAllCorticalAreas<'_, NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> {
        DimensionalNeuronDataRefSliceAllCorticalAreas {
            neuron_cortical_area_index: &self.neuron_cortical_area_index,
            neuron_global_burst_index_of_last_firing: &mut self.neuron_global_burst_index_of_last_firing,
            neuron_membrane_potential: &mut self.neuron_membrane_potential,
            neuron_fire_threshold: &mut self.neuron_fire_threshold,
            neuron_leak_coefficient: &mut self.neuron_leak_coefficient,
            neuron_flags: &mut self.neuron_flags,
            neuron_refractory_countdown: &mut self.neuron_refractory_countdown,
            neuron_consecutive_fire_count: &mut self.neuron_consecutive_fire_count,

            cortical_data: &self.cortical_data,
        }
    }

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: CorticalIndexQuant) -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>, FeagiNPUNeuronError> {
        let cortical_data = self.get_cortical_data_ref(cortical_area_index)?;
        let neuron_range = cortical_data.neuron_range.copy();

        DimensionalNeuronDataRefSliceSingleCorticalArea {
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
        }
    }

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: CorticalIndexQuant, executor: &impl NeuronFireThresholdExecutor<PotentialQuant, CoordQuant>) -> Result<(), FeagiNPUNeuronError> {
        let cortical_data = self.get_cortical_data_ref(cortical_area_index)?;
        executor.set_new_fire_thresholds(
            &mut self.neuron_fire_threshold[&cortical_data.neuron_range],
            &self.neuron_flags[&cortical_data.neuron_range],
            &cortical_data.dimensions
        )
    }
}


impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
DimensionalAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
for DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    /// Creates a cortical area of given dimensions and neuron density,
    /// and returns its cortical area index and range of neuron indexes it covers
    fn create_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel)
                                                 -> Result<(CorticalAreaIndex<CorticalIndexQuant>, Range<NPUNeuronIndex<NeuronIndexQuant>>), FeagiNPUNeuronError> 
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
                                                 cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 cortical_index: CorticalAreaIndex<CorticalIndexQuant>)
                                                 -> Result<(Range<NPUNeuronIndex<NeuronIndexQuant>>, Range<NPUNeuronIndex<NeuronIndexQuant>>), FeagiNPUNeuronError> {

        // no need to verify cortical index since the delete function handles that for us
        let deleted_indexes = self.delete_cortical_area(cortical_index)?;
        // RISKY: We know in current implementation, a deleted cortical index immediately goes back
        // to the available pool, so we *should* get the same one back, right?
        // TODO best to make an explicit system instead! We should be able to have a shared function here
        let new_indexes = self.create_cortical_area_with_default_neurons(cortical_area_dimensions, neurons_per_voxel)?;
        return Ok((deleted_indexes, new_indexes.1));
    }
}


impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
DimensionalStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
for DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

}


impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
BaseNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
for DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    /// Frees unused neuron vector capacity and invalid neurons (assuming they were sorted to the back first!)
    /// albeit allowing a buffer of free space. Returns the number of neurons that were freed.
    /// Returns 0 if no neurons were freed (nothing to free or spare capacity is at or less than
    /// what was requested). Note that invalid neurons not sorted to the back will not be freed.
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<NeuronIndexQuant>) -> NeuronCount<NeuronIndexQuant> {
        todo!()
    }

    /// Deletes a cortical area by invalidating all of its neurons. Returns the neuron indexes
    /// of the disabled neurons
    /// WARNING: BE SURE TO REMOVE ASSOCIATED SYNAPSE MAPPINGS!
    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<CorticalIndexQuant>)
                            -> Result<Range<NPUNeuronIndex<NeuronIndexQuant>>, FeagiNPUNeuronError> {
        self.verify_cortical_area_index_exist_and_valid(&cortical_index)?;
        self.invalidate_cortical_area(cortical_index)
    }




}


impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
BaseNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
for DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    const NUMBER_BYTES_PER_NEURON: usize = 0; // TODO

    /// Gets the maximum possible neuron index achievable by current quantization (or in the case
    /// of static implementations, the size of the neuron array).
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<NeuronIndexQuant> {
        NPUNeuronIndex::MAX_VALUE
    }

    /// Returns the count of valid neurons in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// NEURONS STORED!
    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<NeuronIndexQuant> {
        self.cache_number_valid_neurons
    }


    /// Returns the count of invalid neurons in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<NeuronIndexQuant> {
        self.cache_number_invalid_neurons
    }


    fn get_max_possible_cortical_area_index(&self) -> CorticalAreaIndex<CorticalIndexQuant> {
        CorticalAreaIndex::MAX_VALUE
    }
}

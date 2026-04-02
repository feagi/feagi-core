use core::ops::Range;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential};
use crate::neuron::flags::NeuronFlag;
use crate::neuron::interneuron::shared_funcs_and_structs::InterneuronCorticalData;
use crate::neuron::interneuron::traits::InterneuronAllocStorageTrait;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, LeakCoefficient, NPUNeuronIndex};
// In this implementation, we can do a lot by keeping neurons of a cortical area grouped together, albeit they may not be guaranteed to be in cortical index order


pub struct InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    // Per Neuron (including invalids)
    neuron_cortical_area_index: Vec<CorticalAreaIndex<CorticalIndexQuant>>, // faster than potentially reverse looking up a large hashmap
    neuron_global_burst_index_of_last_firing: Vec<BurstGlobalIndex<BurstIndexQuant>>,
    neuron_membrane_potential: Vec<NeuronMembranePotential<PotentialQuant>>,
    neuron_fire_threshold: Vec<FireThreshold<PotentialQuant>>,
    neuron_leak_coefficient: Vec<LeakCoefficient<PercentageQuant>>,
    neuron_flags: Vec<NeuronFlag>,
    neuron_refractory_countdown: Vec<BurstDelta<BurstDeltaQuant>>,
    neuron_consecutive_fire_count: Vec<BurstDelta<BurstDeltaQuant>>, // how many times the neuron fired burst recently

    // Per Cortical Area (including invalids)
    cortical_data: Vec<InterneuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>>,

    // Cached Data
    cache_number_valid_neurons: NeuronCount<NeuronIndexQuant>,
    cache_number_invalid_neurons: NeuronCount<NeuronIndexQuant>,
    cache_index_to_write_new_neurons: NPUNeuronIndex<NeuronIndexQuant>, // Index starting where new neurons will be written to
    cache_skipped_cortical_indexes: Vec<CorticalAreaIndex<CorticalIndexQuant>>, // when a cortical area is removed, put the index here, these will be the first given out
    cache_invalid_neuron_indexes: Vec<Range<CorticalAreaIndex<NeuronIndexQuant>>>,
}

// NOTE: Only define the constructor here, as we will be going through traits / generics for all data transfer!
impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    pub fn create_new_interneuron_ram_storage(number_neurons_to_preallocate_space_for: NeuronIndexQuant, number_cortical_areas_to_preallocate_space_for: CorticalIndexQuant) -> Result<Self, FeagiNPUDataError> {
        Ok(
            Self {
                neuron_cortical_area_index: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_global_burst_index_of_last_firing: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_membrane_potential: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_fire_threshold: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_leak_coefficient: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_flags: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_refractory_countdown: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_consecutive_fire_count: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),

                cortical_data: Vec::new_with_capacity(number_cortical_areas_to_preallocate_space_for as usize),

                cache_number_valid_neurons: NeuronIndexQuant:ZERO,
                cache_number_invalid_neurons: NeuronIndexQuant:ZERO,
                cache_index_to_write_new_neurons: NeuronIndexQuant:ZERO,
                cache_skipped_cortical_indexes: Vec::new(),
                cache_invalid_neuron_indexes: Vec::new(),
            }
        )
    }


    //region Internal Helper Functions

    /// Marks the neurons of a cortical area as invalid, as well as other cache work in this regard.
    /// Returns the range of neuron indexes invalidated.
    fn invalidate_cortical_area(&mut self, cortical_area_index: CorticalIndexQuant) -> Result<Range<NeuronIndexQuant>, FeagiNPUDataError> {
        // These basic checks are fast and we arent iterating over cortical areas THAT fast, right? // TODO shove checks in a debug?

        let cortical_data: InterneuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant> =
            self.get_cortical_data_ref_mut(&cortical_area_index);


        if !cortical_data.flags.is_valid() {
            return Err(FeagiNPUDataError::InvalidCorticalIndex{given_cortical_index: cortical_area_index as u32})
        }

        cortical_data.flags.toggle_validity();

        let number_of_neurons = NeuronIndexQuant::from_usize({
            // TODO (debug?) check for validity of range

            let neuron_flag_slice: &mut[InterneuronFlag] = self.neuron_flags[cortical_data.neuron_range];

            // so, since we actually do not care for any other flag in the neuron data except for
            // the is valid flag being set to false, just mass fill the area with the bitpack containing
            // that setting

            let invalid_flag = InterneuronFlag:INVALID_FLAG;
            neuron_flag_slice.fill(invalid_flag);
            neuron_flag_slice.len()
        });

        // Some neurons may have died on their own
        let number_of_neurons_invalidated: NeuronIndexQuant = number_of_neurons - cortical_data.neurons_invalid_from_degeneration;

        // Mark neurons as dead in the cache too
        self.cache_number_valid_neurons -= number_of_neurons_invalidated;
        self.cache_number_invalid_neurons += number_of_neurons_invalidated;
        self.cache_invalid_neuron_indexes.push(cortical_data.neuron_range.clone()); // TODO maybe we should have a smarter insert? in the case of connecting segments, make them one bigger segment instead

        // Mark this cortical index as free
        self.cache_skipped_cortical_indexes.push(cortical_area_index);
        self.cortical_cortical_metadata.remove(&cortical_area_index);
    }

    /// Adds cortical data to the next available cortical area slot (either at the end or in the middle if available. Returns the cortical ID used
    fn add_cortical_data_to_next_available_cortical_area_index(&mut self, new_cortical_data: InterneuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>) -> Result<CorticalIndexQuant, FeagiNPUDataError> {
        // TODO Extreme edge case error, when we hit quant limit
        if &self.cache_skipped_cortical_indexes.is_empty() {
            let cortical_index: CorticalIndexQuant = CorticalIndexQuant::from_u32(self.cortical_data.len());
            self.cortical_data.push(new_cortical_data);
        }
        else {
            let cortical_index: CorticalIndexQuant = self.cache_skipped_cortical_indexes.pop().unwrap();
            // TODO DEBUG: ensure we arent overwriting a valid cortical area!
            self.cortical_data[cortical_index as usize] = new_cortical_data;
        }
        return Ok(cortical_index)
    }

    /// Returns an empty result if a cortical area exists AND is valid. Otherwise errors.
    fn verify_cortical_area_index_exist_and_valid(&self, cortical_area_index: &CorticalIndexQuant) -> Result<(), FeagiNPUDataError> {
        let reference = self.get_cortical_data_ref(cortical_area_index)?;
        if reference.flags.is_valid() {
            return Ok(())
        }
        FeagiNPUDataError::InvalidCorticalIndex{given_cortical_index: cortical_area_index as u32}
    }

    /// Get the cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
    fn get_cortical_data_ref(&self, cortical_area_index: &CorticalIndexQuant) -> Result<&InterneuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>, FeagiNPUDataError> {
        self.cortical_data.get(cortical_area_index as usize)
            .ok_or_else(|| FeagiNPUDataError::InvalidCorticalIndex{given_cortical_index: cortical_area_index as u32})?
    }

    /// Get the mutable cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
    fn get_cortical_data_ref_mut(&mut self, cortical_area_index: &CorticalIndexQuant) -> Result<&mut InterneuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>, FeagiNPUDataError> {
        self.cortical_data.get(cortical_area_index as usize)
            .ok_or_else(|| FeagiNPUDataError::InvalidCorticalIndex{given_cortical_index: cortical_area_index as u32})?
    }

    //endregion

}


impl<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
InterneuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
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
    fn create_cortical_area_with_spanned_neuron(&mut self,
                                                cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                neurons_per_voxel: NumberNeuronsPerVoxel,
                                                neuron_global_burst_index_of_last_firing: BurstIndexQuant,
                                                neuron_membrane_potential: MembranePotential<PotentialQuant>,
                                                neuron_fire_threshold: FireThreshold<PotentialQuant>,
                                                neuron_leak_coefficient: PercentageQuant,
                                                neuron_refractory_countdown: BurstDeltaQuant,
                                                neuron_consecutive_fire_count: BurstDeltaQuant,
                                                cortical_excitability: PercentageQuant,
                                                cortical_refractory_period_limit: BurstDeltaQuant,
                                                cortical_fire_threshold_limit: FireThresholdLimit<PotentialQuant>,
                                                cortical_consecutive_fire_limit: BurstDeltaQuant,
                                                cortical_is_mp_charge_accumulation_enabled: bool,
                                                cortical_is_mp_driven_psp_enabled: bool)
                                                -> Result<(CorticalIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError> {

        // NOTE: for now neuron flag only checks for validity, so we dont need that parameter.
        let neuron_flag = InterneuronFlag::new_valid();

        // TODO debug: check against allocation with invalid neuron flag


        // Find where to write neuron data
        let number_of_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
        let (neuron_index_range, is_allocation_at_end_needed): (Range<NeuronIndexQuant>, bool) = {
            // TODO instead of allocating right to the end, what if we have a way to quickly check through cache_invalid_neuron_indexes (assuming we also group neighboring ranges) and put ourselves there if we fit?
            //if self.cache_number_invalid_neurons as usize > number_of_neurons {
            //
            //}
            // TODO size checks (not debug only, we need to be careful)
            let start = self.cache_index_to_write_new_neurons.clone();
            self.cache_index_to_write_new_neurons += number_of_neurons; // increment new index
            (start..(start + number_of_neurons), true)
        };

        // Create and write cortical data

        let mut cortical_flags: InterneuronCorticalFlag = InterneuronCorticalFlag::new_valid();
        cortical_flags.set_mp_charge_accumulation_enabled(cortical_is_mp_charge_accumulation_enabled);
        cortical_flags.cortical_is_mp_driven_psp_enabled(cortical_is_mp_driven_psp_enabled);

        let cortical_data = InterneuronCorticalData{
            flags: cortical_flags,
            neuron_range: neuron_index_range,
            number_neurons_invalid_from_degeneration: 0, // no neurons assumed dead yet
            dimensions: cortical_area_dimensions,
            number_neurons_per_voxel: neurons_per_voxel,
            excitability: cortical_excitability,
            refractory_period_limit: cortical_refractory_period_limit,
            fire_threshold_limit: cortical_fire_threshold_limit,
            consecutive_fire_limit: cortical_consecutive_fire_limit,
        };

        let cortical_index: CorticalIndexQuant = self.add_cortical_data_to_next_available_cortical_area_index(cortical_data)?;

        // Actually allocate if needed, otherwise write to existing memory
        if is_allocation_at_end_needed {
            self.neuron_cortical_area_index.extend(iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing.extend(iter::repeat_n(neuron_global_burst_index_of_last_firing, number_of_neurons));
            self.neuron_membrane_potential.extend(iter::repeat_n(neuron_membrane_potential, number_of_neurons));
            self.neuron_fire_threshold.extend(iter::repeat_n(neuron_fire_threshold, number_of_neurons));
            self.neuron_leak_coefficient.extend(iter::repeat_n(neuron_leak_coefficient, number_of_neurons));
            self.neuron_flags.extend(iter::repeat_n(neuron_flag, number_of_neurons));
            self.neuron_refractory_countdown.extend(iter::repeat_n(neuron_refractory_countdown, number_of_neurons));
            self.neuron_consecutive_fire_count.extend(iter::repeat_n(neuron_consecutive_fire_count, number_of_neurons));
        }
        else {
            self.neuron_cortical_area_index[neuron_index_range].fill(iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing[neuron_index_range].fill(iter::repeat_n(neuron_global_burst_index_of_last_firing, number_of_neurons));
            self.neuron_membrane_potential[neuron_index_range].fill(iter::repeat_n(neuron_membrane_potential, number_of_neurons));
            self.neuron_fire_threshold[neuron_index_range].fill(iter::repeat_n(neuron_fire_threshold, number_of_neurons));
            self.neuron_leak_coefficient[neuron_index_range].fill(iter::repeat_n(neuron_leak_coefficient, number_of_neurons));
            self.neuron_flags[neuron_index_range].fill(iter::repeat_n(neuron_flag, number_of_neurons));
            self.neuron_refractory_countdown[neuron_index_range].fill(iter::repeat_n(neuron_refractory_countdown, number_of_neurons));
            self.neuron_consecutive_fire_count[neuron_index_range].fill(iter::repeat_n(neuron_consecutive_fire_count, number_of_neurons));
        }

        return Ok((cortical_index, neuron_index_range))
    }


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_configured_neurons(&mut self,
                                                    cortical_area_data: InterneuronCorticalData,
                                                    neuron_data: InterneuronDataFromCorticalArea)
                                                    -> Result<(CorticalIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError> {

        // Find where to write neuron data
        let number_of_neurons: usize = cortical_area_data.get_number_contained_neurons_total() as usize;
        let (neuron_index_range, is_allocation_at_end_needed): (Range<NeuronIndexQuant>, bool) = {
            // TODO instead of allocating right to the end, what if we have a way to quickly check through cache_invalid_neuron_indexes (assuming we also group neighboring ranges) and put ourselves there if we fit?
            //if self.cache_number_invalid_neurons as usize > number_of_neurons {
            //
            //}
            // TODO size checks (not debug only, we need to be careful)
            let start = self.cache_index_to_write_new_neurons.clone();
            self.cache_index_to_write_new_neurons += number_of_neurons; // increment new index
            (start..(start + number_of_neurons), true)
        };

        let cortical_index: CorticalIndexQuant = self.add_cortical_data_to_next_available_cortical_area_index(cortical_area_data)?;

        // Actually allocate if needed, otherwise write to existing memory
        if is_allocation_at_end_needed {
            self.neuron_cortical_area_index.extend(iter::repeat_n(cortical_index, number_of_neurons));
            self.neuron_global_burst_index_of_last_firing.extend(neuron_data.neuron_global_burst_index_of_last_firing);
            self.neuron_membrane_potential.extend(neuron_data.neuron_membrane_potential);
            self.neuron_fire_threshold.extend(neuron_data.neuron_fire_threshold);
            self.neuron_leak_coefficient.extend(neuron_data.neuron_leak_coefficient);
            self.neuron_flags.extend(neuron_data.neuron_flags);
            self.neuron_refractory_countdown.extend(neuron_data.neuron_refractory_countdown);
            self.neuron_consecutive_fire_count.extend(neuron_data.neuron_consecutive_fire_count);
        }
        else {
            self.neuron_cortical_area_index[neuron_index_range].fill(iter::repeat_n(cortical_index, number_of_neurons));
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


impl InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
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
    fn get_all_neuron_values_to_process(&mut self) -> InterneuronDataRefSliceAllCorticalAreas<'_> {
        InterneuronDataRefSliceAllCorticalAreas {
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
    fn get_cortical_area_neuron_values_to_process(&mut self, cortical_area_index: CorticalIndexQuant)
                                                  -> Result<InterneuronDataRefSliceSingleCorticalArea<'_>, FeagiNPUDataError> {

        let cortical_data = self.get_cortical_data_ref(cortical_area_index)?;
        let neuron_range = cortical_data.neuron_range.copy();

        InterneuronDataRefSliceSingleCorticalArea {
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

    fn set_neuron_fire_threshold_with_increment(&mut self, cortical_area_index: CorticalIndexQuant, increment_function: &FireThresholdIncrementFunction)  // TODO FireThresholdIncrementFunction
                                                -> Result<(), FeagiNPUDataError>;

}


impl DimensionalAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
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
                                                 -> Result<(CorticalIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError> {

        let expected_number_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
        self.create_cortical_area_with_spanned_neuron(
            cortical_area_dimensions,
            neurons_per_voxel,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_MEMBRANE_POTENTIAL,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_FIRE_THRESHOLD,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_LEAK_COEFFICIENT,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_REFRACTORY_COUNTDOWN,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT,

            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_NEURONS_PER_VOXEL,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_EXCITABILITY,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED,
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
                                                 cortical_index: CorticalIndexQuant)
                                                 -> Result<(Range<NeuronIndexQuant>, Range<NeuronIndexQuant>), FeagiNPUDataError> {

        // no need to verify cortical index since the delete function handles that for us
        let deleted_indexes = self.delete_cortical_area(cortical_index)?;
        // RISKY: We know in current implementation, a deleted cortical index immediately goes back
        // to the available pool, so we *should* get the same one back, right?
        // TODO best to make an explicit system instead! We should be able to have a shared function here
        let new_indexes = self.create_cortical_area_with_default_neurons(cortical_area_dimensions, neurons_per_voxel)?;
        return Ok((deleted_indexes, new_indexes));
    }
}


impl DimensionalStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
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


impl BaseNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
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
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronIndexQuant) -> NeuronIndexQuant {
        todo!()
    }

    /// Deletes a cortical area by invalidating all of its neurons. Returns the neuron indexes
    /// of the disabled neurons
    /// WARNING: BE SURE TO REMOVE ASSOCIATED SYNAPSE MAPPINGS!
    fn delete_cortical_area(&mut self, cortical_index: CorticalIndexQuant)
                            -> Result<Range<NeuronIndexQuant>, FeagiNPUDataError> {
        self.verify_cortical_area_index_exist_and_valid(&cortical_index)?;
        self.invalidate_cortical_area(cortical_index)
    }




}


impl BaseNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    const NUMBER_BYTES_PER_NEURON = 0; // TODO

    /// Gets the maximum possible neuron index achievable by current quantization (or in the case
    /// of static implementations, the size of the neuron array).
    fn get_max_possible_neuron_index(&self) -> NeuronIndexQuant {
        NeuronIndexQuant::MAX
    }

    /// Returns the count of valid neurons in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// NEURONS STORED!
    fn get_total_number_of_valid_neurons(&self) -> NeuronIndexQuant {
        &self.cache_number_valid_neurons
    }


    /// Returns the count of invalid neurons in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_neurons(&self) -> NeuronIndexQuant {
        &self.cache_number_invalid_neurons
    }

    /// Gets the maximum possible cortical area index achievable by current quantization (or in the
    /// case of static implementations, the size of the neuron array).
    fn get_max_possible_cortical_areas(&self) -> CorticalIndexQuant {
        CorticalIndexQuant::MAX
    }

}

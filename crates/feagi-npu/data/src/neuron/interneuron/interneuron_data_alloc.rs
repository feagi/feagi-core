use ahash::AHashMap;

// TODO Rayon? Could the trait perhaps implement some sort of iterator support for rayon?

// In this implementation, we can do a lot by keeping neurons of a cortical area grouped together, albeit they may not be guaranteed to be in cortical index order


pub struct InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{
    // Per Neuron
    neuron_cortical_area_index: Vec<CorticalIndexQuant>, // faster than potentially reverse looking up a large hashmap
    neuron_membrane_potential: Vec<PotentialQuant>,
    neuron_threshold: Vec<PotentialQuant>,
    neuron_leak_coefficient: Vec<PercentageQuant>,
    neuron_flags: Vec<InterneuronFlag>,
    neuron_neuron_refractory_countdown: Vec<BurstQuant>,
    neuron_consecutive_fire_count: Vec<BurstQuant>,
    neuron_consecutive_fire_limit: Vec<BurstQuant>,
    neuron_snooze_period_countdown: Vec<BurstQuant>,
    neuron_snooze_period_limit: Vec<BurstQuant>,

    // Per Cortical Area // NOTE: due to implementation, its possible for these vectors to also have blank unused spots within them!
    cortical_refractory_period: Vec<BurstQuant>,
    cortical_excitability: Vec<PercentageQuant>,
    cortical_threshold_limit: Vec<PotentialQuant>,

    // Cached Data
    cache_cortical_metadata: AHashMap<CorticalIndexQuant, InterneuronCorticalData>,
    cache_number_valid_neurons: NeuronIndexQuant,
    cache_number_invalid_neurons: NeuronIndexQuant,
    cache_index_to_write_new_neurons: NeuronIndexQuant, // Index starting where new neurons will be written to
    cache_skipped_cortical_indexes: Vec<CorticalIndexQuant>, // when a cortical area is removed, put the index here, these will be the first given out
    cache_invalid_neuron_indexes: Vec<Range<NeuronIndexQuant>>,
}


impl InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{
    pub fn create_new_interneuron_ram_storage(number_neurons_to_preallocate_space_for: NeuronIndexQuant, number_cortical_areas_to_preallocate_space_for: CorticalIndexQuant) -> Result<Self, FeagiNeuronError> {
        Ok(
            Self {
                neuron_cortical_area_index: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_membrane_potential: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_threshold: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_leak_coefficient: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_flags: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                neuron_refractory_countdown: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                consecutive_fire_count: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                consecutive_fire_limit: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                snooze_period_countdown: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),
                snooze_period_limit: Vec::new_with_capacity(number_neurons_to_preallocate_space_for as usize),

                cortical_refractory_period: Vec::new_with_capacity(number_cortical_areas_to_preallocate_space_for as usize),
                cortical_excitability: Vec::new_with_capacity(number_cortical_areas_to_preallocate_space_for as usize),
                cortical_threshold_limit: Vec::new_with_capacity(number_cortical_areas_to_preallocate_space_for as usize),
                cortical_neurons_per_voxel: Vec::new_with_capacity(number_cortical_areas_to_preallocate_space_for as usize),
            }
        )
    }


    /// Marks the neurons of a cortical area as invalid, as well as other cache work in this regard.
    /// Returns the range of neuron indexes invalidated.
    fn invalidate_cortical_area(&mut self, cortical_area_index: CorticalIndexQuant) -> Result<Range<NeuronIndexQuant>, FeagiNPUDataError> {
        // These basic checks are fast and we arent iterating over cortical areas THAT fast, right? // TODO shove checks in a debug?

        let cortical_data: InterneuronCorticalData<NeuronIndexQuant> = self.cache_cortical_metadata.get_mut(&cortical_area_index)
            .copied().ok_or_else(|| FeagiNPUDataError::InvalidCorticalIndex{given_cortical_index: cortical_area_index as u32})?;

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
        self.cache_cortical_metadata.remove(&cortical_area_index);
    }


    //region Internal Helper Functions

    fn next_available_cortical_area_index(&self)  -> Result<CorticalIndexQuant, FeagiNPUDataError> { // TODO Extreme edge case error, when we hit quat limit
        if &self.cache_skipped_cortical_indexes.is_empty() {
            return Ok(NeuronIndexQuant::from_u32(self.cache_cortical_metadata.len()));
        }
        Ok(self.cache_skipped_cortical_indexes.last().unwrap()) // TODO is last() performant??
    }

    /// Returns an empty result if a cortical area exists AND is valid. Otherwise errors.
    fn verify_cortical_area_index_exist_and_valid(&self, cortical_area_index: CorticalIndexQuant) -> Result<(), FeagiNPUDataError> {
        let reference = self.get_cortical_data_ref(cortical_area_index)?;
        if reference.flags.is_valid() {
            return Ok(())
        }
        FeagiNPUDataError::InvalidCorticalIndex{given_cortical_index: cortical_area_index as u32}
    }

    /// Get the cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
    fn get_cortical_data_ref(&self, cortical_area_index: CorticalIndexQuant) -> Result<&InterneuronCorticalData<NeuronIndexQuant>, FeagiNPUDataError> {
        self.cache_cortical_metadata.get(&cortical_area_index)
            .ok_or_else(|| FeagiNPUDataError::InvalidCorticalIndex{given_cortical_index: cortical_area_index as u32})?
    }

    /// Get the mutable cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
    fn get_cortical_data_ref_mut(&mut self, cortical_area_index: CorticalIndexQuant) -> Result<&mut InterneuronCorticalData<NeuronIndexQuant>, FeagiNPUDataError> {
        self.cache_cortical_metadata.get_mut(&cortical_area_index)
            .ok_or_else(|| FeagiNPUDataError::InvalidCorticalIndex{given_cortical_index: cortical_area_index as u32})?
    }

    //endregion

}


impl InterneuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{

    /// Creates a cortical area of given dimensions but using a set of neuron values copied across
    /// all neurons.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_spanned_neuron(&mut self,
                                                cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                neurons_per_voxel: NumberNeuronsPerVoxel,
                                                neuron_membrane_potential: PotentialQuant,
                                                neuron_leak_coefficient: PercentageQuant,
                                                neuron_flag: InterneuronFlag,
                                                neuron_refractory_countdown: BurstQuant,
                                                neuron_consecutive_fire_count: BurstQuant,
                                                neuron_consecutive_fire_limit: BurstQuant,
                                                neuron_snooze_period_countdown: BurstQuant,
                                                neuron_snooze_period_limit: BurstQuant,
                                                cortical_refractory_period: BurstQuant,
                                                cortical_excitability: PercentageQuant,
                                                cortical_threshold_limit: PotentialQuant,
                                                cortical_is_mp_charge_accumulation_enabled: bool,
                                                cortical_is_mp_driven_psp_enabled: bool,
                                                cortical_neurons_per_voxel: NumberNeuronsPerVoxel)
                                                -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError> {

        let number_of_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);

        let neuron_index_range: Range<NeuronIndexQuant> = {
            // TODO instead of allocating right to the end, what if we have a way to quickly check through cache_invalid_neuron_indexes (assuming we also group neighboring ranges) and put ourselves there if we fit?
            //if self.cache_number_invalid_neurons as usize > number_of_neurons {
            //
            //}
            // TODO size checks (not debug only, we need to be careful)
            let start = self.cache_index_to_write_new_neurons.clone();
            self.cache_index_to_write_new_neurons += number_of_neurons;
            return start..(start + number_of_neurons);
        };

        let mut cortical_flags: InterneuronCorticalFlag = InterneuronCorticalFlag::new_valid();
        cortical_flags.set_mp_charge_accumulation_enabled(cortical_is_mp_charge_accumulation_enabled);
        cortical_flags.cortical_is_mp_driven_psp_enabled(cortical_is_mp_driven_psp_enabled);

        let cortical_index = self.next_available_cortical_area_index();
        _ = self.cache_cortical_metadata.insert(
            cortical_area_index,
            InterneuronCorticalData<NeuronIndexQuant, CoordQuant> {
                flags: cortical_flags,
                neuron_range: neuron_index_range.clone(),
                number_neurons_invalid_from_degeneration: 0,
                dimensions: cortical_area_dimensions
            }
        );

        // TODO insert into frag logic with above


        // TODO continue here!




    }


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_configured_neurons(&mut self,
                                                    cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                    neurons_per_voxel: NumberNeuronsPerVoxel,
                                                    neuron_data: InterneuronDataFromCorticalArea)
                                                    -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError>;

}


impl InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{



}


impl DimensionalAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{
    /// Creates a cortical area of given dimensions and neuron density,
    /// and returns its cortical area index and range of neuron indexes it covers
    fn create_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel)
                                                 -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError> {

        let expected_number_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
        self.create_cortical_area_with_spanned_neuron(
            cortical_area_dimensions,
            neurons_per_voxel,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_MEMBRANE_POTENTIAL,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_THRESHOLD,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_LEAK_COEFFICIENT,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_REFRACTORY_COUNTDOWN,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_CONSECUTIVE_FIRE_LIMIT,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_SNOOZE_PERIOD_COUNTDOWN,
            InterneuronStaticStorageTrait::DEFAULT_NEURON_SNOOZE_PERIOD_LIMIT,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_REFRACTORY_PERIOD,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_EXCITABILITY,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_THRESHOLD_LIMIT,
            InterneuronStaticStorageTrait::DEFAULT_CORTICAL_NEURONS_PER_VOXEL,
        )
    }


    /// Effectively deletes a cortical area (by invalidating their neurons), then rebuilds it to the
    /// new given dimensions and density. While cortical properties are preserved, neuron data is
    /// reset to default. Returns a tuple of the old invalid neuron index range, and the new
    /// created neuron index range.
    fn resize_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 cortical_index: CorticalIndexQuant)
                                                 -> Result<(Range<NeuronIndexQuant>, Range<NeuronIndexQuant>), FeagiNPUDataError>;
}


impl DimensionalStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{

}


impl BaseNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{

    /// Frees unused vector capacity and invalid neurons (assuming they were sorted to the back first!)
    /// albeit allowing a buffer of free space. Returns the number of neurons that were freed.
    /// Returns 0 if no neurons were freed (nothing to free or spare capacity is at or less than
    /// what was requested). Note that invalid neurons not sorted to the back will not be freed.
    fn free_unused_capacity(&mut self, spare_capacity_to_maintain: NeuronIndexQuant) -> NeuronIndexQuant {
        todo!()
    }

    /// Deletes a cortical area by invalidating all of its neurons. Returns the neuron indexes
    /// of the disabled neurons
    fn delete_cortical_area(&mut self, cortical_index: CorticalIndexQuant)
                            ->Result<Range<NeuronIndexQuant>, FeagiNPUDataError> {
        self.invalidate_cortical_area(cortical_index)
    }




}


impl BaseNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
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



#[derive(Debug, Clone)]
struct InterneuronCorticalData<NeuronIndexQuant, CoordQuant> where
    NeuronIndexQuant: InterneuronIndex,
    CoordQuant: QuantizableUInt,
{
    pub flags: InterneuronCorticalFlag,
    pub neuron_range: Range<NeuronIndexQuant>,
    pub number_neurons_invalid_from_degeneration: NeuronIndexQuant,
    pub dimensions: NeuronVoxelDimensions<CoordQuant>,
    pub number_neurons_per_voxel: NumberNeuronsPerVoxel
}
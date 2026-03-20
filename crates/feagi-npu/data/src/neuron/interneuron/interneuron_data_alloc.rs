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
    neuron_refractory_countdown: Vec<BurstQuant>,
    consecutive_fire_count: Vec<BurstQuant>,
    consecutive_fire_limit: Vec<BurstQuant>,
    snooze_period_countdown: Vec<BurstQuant>,
    snooze_period_limit: Vec<BurstQuant>,

    // Per Cortical Area
    cortical_refractory_period: Vec<BurstQuant>,
    cortical_excitability: Vec<PercentageQuant>,
    cortical_threshold_limit: Vec<PotentialQuant>,
    cortical_neurons_per_voxel: Vec<NumberNeuronsPerVoxel>,

    // Cached Data
    cache_cortical_neuron_mappings: AHashMap<CorticalIndexQuant, InterneuronCorticalData>,
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

    pub fn create_new_interneuron_storage(number_neurons_to_preallocate_space_for: NeuronIndexQuant, number_cortical_areas_to_preallocate_space_for: CorticalIndexQuant) -> Result<Self, FeagiNeuronError> {
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

        let cortical_data: InterneuronCorticalData<NeuronIndexQuant> = self.cache_cortical_neuron_mappings.get_mut(&cortical_area_index)
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
        self.cache_cortical_neuron_mappings.remove(&cortical_area_index);

    }

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

impl BaseNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> for InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{


    fn free_unused_capacity(&mut self, spare_capacity_to_maintain: NeuronIndexQuant) -> NeuronIndexQuant {
        todo!()
    }

    fn next_available_cortical_area_index(&self)  -> Result<&CorticalIndexQuant, FeagiNPUDataError> { // TODO Extreme edge case error, when we hit quat limit

        if &self.cache_skipped_cortical_indexes.is_empty() {
            return Ok(&NeuronIndexQuant::from_u32(self.cache_cortical_neuron_mappings.len()));
        }
        Ok(&self.cache_skipped_cortical_indexes.last().unwrap()) // TODO is last() performant??
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
    pub neurons_invalid_from_degeneration: NeuronIndexQuant,
    pub dimensions: NeuronVoxelDimensions<CoordQuant>,
}
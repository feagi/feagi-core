


pub trait InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>: BaseNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
// % synaptic attractivity
{
    // TODO are these defaults fine?
    const DEFAULT_NEURON_MEMBRANE_POTENTIAL: PotentialQuant = PotentialQuant::ZERO;
    const DEFAULT_NEURON_THRESHOLD: PotentialQuant = PotentialQuant::ZERO;
    const DEFAULT_NEURON_LEAK_COEFFICIENT: PercentageQuant = PercentageQuant::ZERO;
    const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstQuant = BurstQuant::ZERO;
    const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstQuant = BurstQuant::ZERO;
    const DEFAULT_NEURON_CONSECUTIVE_FIRE_LIMIT: BurstQuant = BurstQuant::ZERO;
    const DEFAULT_NEURON_SNOOZE_PERIOD_COUNTDOWN: BurstQuant = BurstQuant::ZERO;
    const DEFAULT_NEURON_SNOOZE_PERIOD_LIMIT: BurstQuant = BurstQuant::ZERO;

    const DEFAULT_CORTICAL_REFRACTORY_PERIOD: BurstQuant = BurstQuant::ZERO;
    const DEFAULT_CORTICAL_EXCITABILITY: PercentageQuant = PercentageQuant::ZERO;
    const DEFAULT_CORTICAL_THRESHOLD_LIMIT: PotentialQuant = PotentialQuant::ZERO;
    const DEFAULT_CORTICAL_NEURONS_PER_VOXEL: NumberNeuronsPerVoxel = 1;

    /// Returns a struct of references to the slices of all neuron data (include sparse invalids)
    fn get_all_neuron_values_to_process(&mut self) -> InterneuronDataRefSliceMultiCorticalArea<'_>;

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_cortical_area_neuron_values_to_process(&mut self, cortical_area_index: CorticalIndexQuant)
        -> Result<InterneuronDataRefSliceSingleCorticalArea<'_>, FeagiNPUDataError>;



    // TODO add more specific functions for getting specific fields for neuron processing

}





#[cfg(feature = "alloc")]
pub trait InterneuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>:
BaseNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> +
InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
// % synaptic attractivity
{
    /// Creates a cortical area of given dimensions and neuron density,
    /// and returns its cortical area index and range of neuron indexes it covers
    fn create_cortical_area_with_default_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel)
        -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError>;

    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_configured_neurons(&mut self,
                                                    cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                    neurons_per_voxel: NumberNeuronsPerVoxel,
                                                    neuron_data: InterneuronDataFromCorticalArea)
        -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError>;




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

// TODO I took a best guess at which fields should be mutable


/// Used to pass around slices easily at low cost for multiple cortical areas
pub struct InterneuronDataRefSliceMultiCorticalArea<'a> {
    pub neuron_cortical_area_index: &'a [CorticalIndexQuant],
    pub neuron_membrane_potential: &'a mut [PotentialQuant],
    pub neuron_threshold: &'a mut [PotentialQuant],
    pub neuron_leak_coefficient: &'a mut [PercentageQuant],
    pub neuron_flags: &'a [InterneuronFlag], // TODO for degeneracy, we may need this mutable so a neuron can flag itself as dead
    pub neuron_refractory_countdown: &'a mut [BurstQuant],
    pub consecutive_fire_count: &'a mut [BurstQuant],
    pub consecutive_fire_limit: &'a mut [BurstQuant],
    pub snooze_period_countdown: &'a mut [BurstQuant],
    pub snooze_period_limit: &'a mut [BurstQuant],

    pub cortical_refractory_period: &'a [BurstQuant],
    pub cortical_excitability: &'a [PercentageQuant],
    pub cortical_threshold_limit: &'a [PotentialQuant],
    pub cortical_neurons_per_voxel: &'a [NumberNeuronsPerVoxel],
}

/// Used to pass around slices easily at low cost for a single cortical area
pub struct InterneuronDataRefSliceSingleCorticalArea<'a> {
    pub neuron_membrane_potential: &'a mut [PotentialQuant],
    pub neuron_threshold: &'a mut [PotentialQuant],
    pub neuron_leak_coefficient: &'a mut [PercentageQuant],
    pub neuron_flags: &'a [InterneuronFlag], // TODO for degeneracy, we may need this mutable so a neuron can flag itself as dead
    pub neuron_refractory_countdown: &'a mut [BurstQuant],
    pub consecutive_fire_count: &'a mut [BurstQuant],
    pub consecutive_fire_limit: &'a mut [BurstQuant],
    pub snooze_period_countdown: &'a mut [BurstQuant],
    pub snooze_period_limit: &'a mut [BurstQuant],

    pub cortical_refractory_period: BurstQuant,
    pub cortical_excitability: PercentageQuant,
    pub cortical_threshold_limit: PotentialQuant,
    pub cortical_neurons_per_voxel: NumberNeuronsPerVoxel,
    pub neuron_index_offset: NeuronIndexQuant // offset from the total neuron array
}

impl InterneuronDataRefSliceSingleCorticalArea<'_> {
    // TODO add iterators and par iterators for coordinates, which are needed for some synapse stuff
}





/// Used to pass data of neurons to be added or moved for a cortical index
#[cfg(feature = "alloc")]
pub struct InterneuronDataFromCorticalArea {
    pub neuron_membrane_potential: Vec<PotentialQuant>,
    pub neuron_threshold: Vec<PotentialQuant>,
    pub neuron_leak_coefficient: Vec<PercentageQuant>,
    pub neuron_flags: Vec<InterneuronFlag>,
    pub neuron_refractory_countdown: Vec<BurstQuant>,
    pub consecutive_fire_count: Vec<BurstQuant>,
    pub consecutive_fire_limit: Vec<BurstQuant>,
    pub snooze_period_countdown: Vec<BurstQuant>,
    pub snooze_period_limit: Vec<BurstQuant>,

    pub cortical_refractory_period: BurstQuant,
    pub cortical_excitability: PercentageQuant,
    pub cortical_threshold_limit: PotentialQuant,
    pub cortical_neurons_per_voxel: NumberNeuronsPerVoxel,
}




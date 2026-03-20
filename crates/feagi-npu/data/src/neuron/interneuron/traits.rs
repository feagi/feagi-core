


pub trait InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>: BaseNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
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
DimensionalAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> +
DimensionalStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> +
BaseNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> +
InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
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
                                                cortical_neurons_per_voxel: NumberNeuronsPerVoxel)
        -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError>;


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_configured_neurons(&mut self,
                                                    cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                    neurons_per_voxel: NumberNeuronsPerVoxel,
                                                    neuron_data: InterneuronDataFromCorticalArea)
        -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError>;
}








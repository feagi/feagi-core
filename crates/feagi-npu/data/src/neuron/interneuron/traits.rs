
// TODO in static envs, why store neuron indexes for interneuron data? they are ordered and dont increase

pub trait InterneuronData<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>:
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
// % synaptic attractivity
{
    fn get_total_number_of_neurons(&self) -> NeuronIndexQuant;

    fn get_cortical_index(&self, neuron_id: NeuronIndexQuant) -> Result<CorticalIndexQuant, FeagiNPUDataError>;

    //TODO Set Fire threshold increment -> we need a function to be able to set thresholds in a gradiant across a cortical area

    //region Individual neuron Properties

    //region Membrane Potential
    fn get_neuron_membrane_potentials_slice(&self) -> &[PotentialQuant];

    fn get_neuron_membrane_potentials_slice_mut(&mut self) -> &mut [PotentialQuant];
    //endregion

    //region Neuron Voxel Coordinates

    // No setting functions since no neurons can move voxels in static environments

    fn get_neuron_voxel_coordinates_slice(&self) -> &[CoordQuant];
    //endregion

    //region Threshold
    fn get_neuron_thresholds_slice(&self) -> &[PotentialQuant];

    fn get_neuron_thresholds_slice_mut(&mut self) -> &mut [PotentialQuant];
    //endregion

    //region Threshold Limit
    fn get_neuron_threshold_limits_slice(&self) -> &[PotentialQuant];

    fn get_neuron_threshold_limits_slice_mut(&mut self) -> &mut [PotentialQuant];
    //endregion

    //region Leak Coefficient
    fn get_neuron_leak_coefficients_slice(&self) -> &[PercentageQuant];

    fn get_neuron_leak_coefficients_slice_mut(&mut self) -> &mut [PercentageQuant];
    //endregion

    //region Neuron Flags
    fn get_neuron_flags_slice(&self) -> &[InterneuronFlag];

    fn get_neuron_flags_slice_mut(&mut self) -> &mut [InterneuronFlag];
    //endregion

    //region Refractory Countdown
    fn get_neuron_refractory_countdowns_slice(&self) -> &[BurstQuant];

    fn get_neuron_refractory_countdowns_slice_mut(&mut self) -> &mut [BurstQuant];
    //endregion

    //region Fire Count
    fn get_consecutive_fire_count_slice(&self) -> &[BurstQuant];

    fn get_consecutive_fire_count_slice_mut(&mut self) -> &mut [BurstQuant];
    //endregion

    //region Fire Limit
    fn get_consecutive_fire_limit_slice(&self) -> &[BurstQuant];

    fn get_consecutive_fire_limit_slice_mut(&mut self) -> &mut [BurstQuant];
    //endregion

    //region Snooze Period Countdown
    fn get_snooze_period_countdown_slice(&self) -> &[BurstQuant];

    fn get_snooze_period_countdown_slice_mut(&mut self) -> &mut [BurstQuant];
    //endregion

    //region Snooze Period Limit
    fn get_snooze_period_limit_slice(&self) -> &[BurstQuant];

    fn get_snooze_period_limit_slice_mut(&mut self) -> &mut [BurstQuant];
    //endregion

    //endregion


    //region Individual Cortical Area Properties

    fn get_neuron_ids(&self, cortical_index: CorticalIndexQuant) -> Result<impl Iterator<Item = &NeuronIndexQuant>, FeagiNPUDataError>;

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // since this affects neuron count, the setting functions are only in the alloc extension
    fn get_number_of_neurons_per_voxel_descriptor_slice(&self) -> &[NumberNeuronsPerVoxel];
    //endregion

    //endregion
}

#[cfg(feature = "alloc")]
pub trait InterneuronDataAlloc<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>: InterneuronData<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
    // % synaptic attractivity
{
    //region Individual neuron Properties

    //region Neuron Voxel Coordinates
    fn get_neuron_voxel_coordinates_slice_mut(&mut self) -> &mut [CoordQuant];
    //endregion

    //endregion


    //region Individual Cortical Area Properties

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // get functions in base trait
    fn get_number_of_neurons_per_voxel_descriptor_slice_mut(&mut self) -> &mut [NumberNeuronsPerVoxel];
    //endregion

    //endregion

    // TODO functions for adding / removing neurons
}
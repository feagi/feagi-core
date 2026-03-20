

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
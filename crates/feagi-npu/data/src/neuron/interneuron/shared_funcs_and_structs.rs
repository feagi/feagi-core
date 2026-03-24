
// NOTE: The reason we make neuron data flat is because we operate on entire ranges at a time
// and we have some usecases where we quickly want to pull a range at a time
// with cortical areas, we generally rarely operate on entire ranges, so keeping those properties
// in structs makes our life easier


/// Stores data as to the property of cortical areas
/// WARNING: Do not allow modification of this struct outside their implemented interneuron structs, as
/// often values here are tied to other cache values and vice versa!
#[derive(Debug, Clone)]
pub(crate) struct InterneuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CoordQuant: QuantizableUInt,
    BurstDeltaQuant: BurstCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{
    pub flags: InterneuronCorticalFlag, // NOTE: do not allow modifying this structure outside this
    pub neuron_range: Range<NeuronIndexQuant>,
    pub number_neurons_invalid_from_degeneration: NeuronIndexQuant,
    pub dimensions: NeuronVoxelDimensions<CoordQuant>,
    pub number_neurons_per_voxel: NumberNeuronsPerVoxel,
    pub excitability: PercentageQuant,
    pub refractory_period_limit: BurstDeltaQuant,
    pub fire_threshold_limit: PotentialQuant,
    pub consecutive_fire_limit: BurstDeltaQuant,
}

impl InterneuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant> {
    pub fn get_number_contained_neurons_total(&self) -> NeuronIndexQuant {
        self.dimensions.get_number_neurons(self.number_neurons_per_voxel)
    }

    pub fn get_number_contained_neurons_valid(&self) -> NeuronIndexQuant {
        self.get_number_contained_neurons_total() - self.number_neurons_invalid_from_degeneration
    }
}


// TODO I took a best guess at which fields should be mutable

/// Used to pass around slices easily at low cost for all cortical areas
pub struct InterneuronDataRefSliceAllCorticalAreas<'a> {
    pub neuron_cortical_area_index: &'a [CorticalIndexQuant],
    pub neuron_global_burst_index_of_last_firing: &'a mut [BurstIndexQuant],
    pub neuron_membrane_potential: &'a mut [PotentialQuant],
    pub neuron_fire_threshold: &'a mut [PotentialQuant],
    pub neuron_leak_coefficient: &'a mut [PercentageQuant],
    pub neuron_flags: &'a mut [InterneuronFlag],
    pub neuron_refractory_countdown: &'a mut [BurstDeltaQuant],
    pub neuron_consecutive_fire_count: &'a mut [BurstDeltaQuant],

    pub cortical_data: &'a [InterneuronCorticalData],
}


/// Used to pass around slices easily at low cost for a single cortical area
pub struct InterneuronDataRefSliceSingleCorticalArea<'a> {
    pub neuron_global_burst_index_of_last_firing: &'a mut [BurstIndexQuant],
    pub neuron_membrane_potential: &'a mut [PotentialQuant],
    pub neuron_fire_threshold: &'a mut [PotentialQuant],
    pub neuron_leak_coefficient: &'a mut [PercentageQuant],
    pub neuron_flags: &'a mut [InterneuronFlag],
    pub neuron_refractory_countdown: &'a mut [BurstDeltaQuant],
    pub neuron_consecutive_fire_count: &'a mut [BurstDeltaQuant],

    pub cortical_data: &'a InterneuronCorticalData,
    pub global_neuron_index_range: Range<NeuronIndexQuant> // offset from the total neuron array
}


/// Used to pass data of neurons to be added or moved for a cortical index
#[cfg(feature = "alloc")]
pub struct InterneuronDataFromCorticalArea {
    pub neuron_global_burst_index_of_last_firing: Vec<BurstIndexQuant>,
    pub neuron_membrane_potential: Vec<PotentialQuant>,
    pub neuron_fire_threshold: Vec<PotentialQuant>,
    pub neuron_leak_coefficient: Vec<PercentageQuant>,
    pub neuron_flags: Vec<InterneuronFlag>,
    pub neuron_refractory_countdown: Vec<BurstQuant>,
    pub neuron_consecutive_fire_count: Vec<BurstQuant>,

    pub cortical_refractory_period: BurstQuant,
    pub cortical_excitability: PercentageQuant,
    pub cortical_threshold_limit: PotentialQuant,
    pub cortical_neurons_per_voxel: NumberNeuronsPerVoxel,
}
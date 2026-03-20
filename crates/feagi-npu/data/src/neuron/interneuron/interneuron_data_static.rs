use crate::descriptors::{
    BurstDeltaCount, CorticalAreaIndex, InterneuronIndex, NeuronVoxelCoordinate,
    NumberNeuronsPerVoxel, PercentageScale, PotentialUnit, QuantizableUInt,
};

use super::interneuron_flag::InterneuronFlag;

// NOTE: neuron_count_per_cortical_index: As interneuron count and cortical area count cannot
// change, we init this array with neurons grouped in xyz (incrementing in order) by cortical area,
// those groupings being ordered by cortical area size biggest to smallest (in terms of neuron
// count) such that the most common areas to hit (by chance) are at the start.
pub struct InterneuronDataStatic<
    NeuronIndexQuant,
    CorticalIndexQuant,
    CoordQuant,
    BurstQuant,
    PotentialQuant,
    PercentageQuant,
    const MAX_NEURON_INDEX: usize,
    const MAX_CORTICAL_AREA_INDEX: usize,
>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{
    neuron_end_index_and_neurons_per_voxel_per_cortical_index:
        [(NeuronIndexQuant, NumberNeuronsPerVoxel); MAX_CORTICAL_AREA_INDEX], // See note at top

    // Per Neuron
    neuron_cortical_area_index: [CorticalIndexQuant; MAX_NEURON_INDEX],
    neuron_membrane_potential: [PotentialQuant; MAX_NEURON_INDEX],
    neuron_voxel_coordinate: [NeuronVoxelCoordinate<CoordQuant>; MAX_NEURON_INDEX],
    neuron_threshold: [PotentialQuant; MAX_NEURON_INDEX],
    neuron_threshold_limit: [PotentialQuant; MAX_NEURON_INDEX],
    neuron_leak_coefficient: [PercentageQuant; MAX_NEURON_INDEX],
    neuron_flags: [InterneuronFlag; MAX_NEURON_INDEX],
    neuron_refractory_countdown: [BurstQuant; MAX_NEURON_INDEX],
    consecutive_fire_count: [BurstQuant; MAX_NEURON_INDEX],
    consecutive_fire_limit: [BurstQuant; MAX_NEURON_INDEX],
    snooze_period_countdown: [BurstQuant; MAX_NEURON_INDEX],
    snooze_period_limit: [BurstQuant; MAX_NEURON_INDEX],

    // Per Cortical Area
    cortical_excitability: [PercentageQuant; MAX_CORTICAL_AREA_INDEX],
    neurons_per_voxel: [NumberNeuronsPerVoxel; MAX_CORTICAL_AREA_INDEX],
}

impl<
    NeuronIndexQuant,
    CorticalIndexQuant,
    CoordQuant,
    BurstQuant,
    PotentialQuant,
    PercentageQuant,
    const MAX_NEURON_INDEX: usize,
    const MAX_CORTICAL_AREA_INDEX: usize,
>
InterneuronDataStatic<
    NeuronIndexQuant,
    CorticalIndexQuant,
    CoordQuant,
    BurstQuant,
    PotentialQuant,
    PercentageQuant,
    MAX_NEURON_INDEX,
    MAX_CORTICAL_AREA_INDEX,
>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{
    pub fn new_blank() -> Self {

        if NeuronIndexQuant::MAX as usize > MAX_NEURON_INDEX {
            upanic!("Neuron array created with quantization allowing max of {} when Neuron array size is {}", NeuronIndexQuant::MAX, MAX_NEURON_INDEX);
        }

        Self {
            neuron_end_index_and_neurons_per_voxel_per_cortical_index:
                [(NeuronIndexQuant::ZERO, 0); MAX_CORTICAL_AREA_INDEX],
            neuron_cortical_area_index: [CorticalIndexQuant::ZERO; MAX_NEURON_INDEX],
            neuron_membrane_potential: [PotentialQuant::ZERO; MAX_NEURON_INDEX],
            neuron_voxel_coordinate: [NeuronVoxelCoordinate {
                x: CoordQuant::ZERO,
                y: CoordQuant::ZERO,
                z: CoordQuant::ZERO,
            }; MAX_NEURON_INDEX],
            neuron_threshold: [PotentialQuant::ZERO; MAX_NEURON_INDEX],
            neuron_threshold_limit: [PotentialQuant::ZERO; MAX_NEURON_INDEX],
            neuron_leak_coefficient: [PercentageQuant::ZERO; MAX_NEURON_INDEX],
            neuron_flags: [InterneuronFlag::new_valid(); MAX_NEURON_INDEX],
            neuron_refractory_countdown: [BurstQuant::ZERO; MAX_NEURON_INDEX],
            consecutive_fire_count: [BurstQuant::ZERO; MAX_NEURON_INDEX],
            consecutive_fire_limit: [BurstQuant::ZERO; MAX_NEURON_INDEX],
            snooze_period_countdown: [BurstQuant::ZERO; MAX_NEURON_INDEX],
            snooze_period_limit: [BurstQuant::ZERO; MAX_NEURON_INDEX],
            cortical_excitability: [PercentageQuant::ZERO; MAX_CORTICAL_AREA_INDEX],
            neurons_per_voxel: [0; MAX_CORTICAL_AREA_INDEX],
        }
    }
}

// Intentionally deferred for now per request:
 impl InterneuronDataTrait<...> for InterneuronDataStatic<...> {
    fn get_max_possible_neuron_index(&self) -> NeuronIndexQuant;

    fn get_total_number_of_neurons(&self) -> NeuronIndexQuant;
}
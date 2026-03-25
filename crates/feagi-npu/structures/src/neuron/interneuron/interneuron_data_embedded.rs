use crate::descriptors::{
    BurstDeltaCount, CorticalAreaIndex, InterneuronIndex, NeuronVoxelCoordinate,
    NumberNeuronsPerVoxel, PercentageScale, PotentialUnit, QuantizableUInt,
};

use super::interneuron_flag::InterneuronFlag;

// NOTE: neuron_count_per_cortical_index: As interneuron count and cortical area count cannot
// change, we init this array with neurons grouped in xyz (incrementing in order) by cortical area,
// those groupings being ordered by cortical area size biggest to smallest (in terms of neuron
// count) such that the most common areas to hit (by chance) are at the start.
pub struct InterneuronStaticEmbeddedStorage<
    NeuronIndexQuant,
    CorticalIndexQuant,
    CoordQuant,
    BurstDeltaQuant,
    BurstIndexQuant,
    PotentialQuant,
    PercentageQuant,
    const MAX_NEURON_INDEX: usize,
    const MAX_CORTICAL_AREA_INDEX: usize,
>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstDeltaQuant: BurstCount,
    BurstIndexQuant: BurstCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{
    neuron_end_index_and_neurons_per_voxel_per_cortical_index:
        [(NeuronIndexQuant, NumberNeuronsPerVoxel); MAX_CORTICAL_AREA_INDEX], // See note at top

    // Per Neuron
    neuron_cortical_area_index: [CorticalIndexQuant; MAX_NEURON_INDEX],// TODO do we need this?
    neuron_membrane_potential: [PotentialQuant; MAX_NEURON_INDEX],
    neuron_threshold: [PotentialQuant; MAX_NEURON_INDEX],
    neuron_voxel_coordinate: [NeuronVoxelCoordinate<CoordQuant>; MAX_NEURON_INDEX], // TODO do we need this?
    neuron_leak_coefficient: [PercentageQuant; MAX_NEURON_INDEX],
    neuron_flags: [InterneuronFlag; MAX_NEURON_INDEX],
    neuron_refractory_countdown: [BurstQuant; MAX_NEURON_INDEX],
    consecutive_fire_count: [BurstQuant; MAX_NEURON_INDEX],
    consecutive_fire_limit: [BurstQuant; MAX_NEURON_INDEX], c*
    snooze_period_countdown: [BurstQuant; MAX_NEURON_INDEX],
    snooze_period_limit: [BurstQuant; MAX_NEURON_INDEX], c*

    cache_number_valid_neurons: NeuronIndexQuant,
    cache_number_invalid_neurons: NeuronIndexQuant,

    //cort
    neuron_threshold_limit: [PotentialQuant; MAX_NEURON_INDEX],


    // Per Cortical Area
    cortical_excitability: [PercentageQuant; MAX_CORTICAL_AREA_INDEX],
    neurons_per_voxel: [NumberNeuronsPerVoxel; MAX_CORTICAL_AREA_INDEX],
}

impl InterneuronStaticEmbeddedStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{

   // TODO can we even have a constructor? maybe as a const function?



}

impl InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronStaticEmbeddedStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{

    /// Returns a struct of references to the slices of all neuron data (include sparse invalids)
    fn get_all_neuron_values_to_process(&mut self) -> InterneuronDataRefSliceMultiCorticalArea<'_>;

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_cortical_area_neuron_values_to_process(&mut self, cortical_area_index: CorticalIndexQuant)
                                                  -> Result<InterneuronDataRefSliceSingleCorticalArea<'_>, FeagiNPUDataError>;


}




impl DimensionalStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronStaticEmbeddedStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: NeuronVoxelCoordinate<QuantizableUInt>,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale
{

}



impl BaseNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> for InterneuronStaticEmbeddedStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
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





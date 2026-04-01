
// TODO different firing / refractory mode support eventually

use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
use crate::descriptors::{FireThreshold, FireThresholdLimit, MembranePotential};
use crate::neuron::base_traits::BaseNeuronStaticStorageTrait;

pub trait InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>:
BaseNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType, // Using this here as we may be using coords or dimensions
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizableValueType,
{
    // TODO are these defaults fine?
    // Neuron Defaults
    const DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING: PotentialQuant = BurstIndexQuant::ZERO;
    const DEFAULT_NEURON_MEMBRANE_POTENTIAL: MembranePotential<PotentialQuant> = PotentialQuant::ZERO;
    const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<PotentialQuant> = PotentialQuant::ZERO;
    const DEFAULT_NEURON_LEAK_COEFFICIENT: PercentageQuant = PercentageQuant::ZERO;
    const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDeltaQuant = BurstDeltaQuant::ZERO;
    const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDeltaQuant = BurstDeltaQuant::ONE;

    // Cortical Area Defaults
    const DEFAULT_CORTICAL_NEURONS_PER_VOXEL: NumberNeuronsPerVoxel = 1;
    const DEFAULT_CORTICAL_EXCITABILITY: PercentageQuant = PercentageQuant::ZERO;
    const DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT: BurstDeltaQuant = BurstDeltaQuant::ZERO;
    const DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT: FireThresholdLimit<PotentialQuant> = PotentialQuant::ZERO;
    const DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT: BurstDeltaQuant = BurstDeltaQuant::ZERO;
    const DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED: bool = false;
    const DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED: bool = false;

    /// Returns a struct of references to the slices of all neuron data (include sparse invalids)
    fn get_all_neuron_values_to_process(&mut self) -> InterneuronDataRefSliceMultiCorticalArea<'_>;

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_cortical_area_neuron_values_to_process(&mut self, cortical_area_index: CorticalIndexQuant)
        -> Result<InterneuronDataRefSliceSingleCorticalArea<'_>, FeagiNPUDataError>;

    fn set_neuron_fire_threshold_with_increment(&mut self, cortical_area_index: CorticalIndexQuant, increment_function: &FireThresholdIncrementFunction)  // TODO FireThresholdIncrementFunction
        -> Result<(), FeagiNPUDataError>;



    // TODO add more specific functions for getting specific fields for neuron processing

}





#[cfg(feature = "alloc")]
pub trait InterneuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>:
DimensionalAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> +
DimensionalStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> +
BaseNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> +
InterneuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstDeltaQuant: BurstCount,
    BurstIndexQuant: BurstCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
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
        -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError>;


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_configured_neurons(&mut self,
                                                    cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                    neurons_per_voxel: NumberNeuronsPerVoxel,
                                                    neuron_data: InterneuronDataFromCorticalArea)
        -> Result<(NeuronIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError>;
}








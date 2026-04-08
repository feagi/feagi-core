
// TODO different firing / refractory mode support eventually

use core::ops::Range;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_dimension_traits::{DimensionalAllocStorageTrait, DimensionalStaticStorageTrait};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability};
use crate::neuron::base_traits::{BaseNeuronAllocStorageTrait, BaseNeuronStaticStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::dimensional_neurons::shared_funcs_and_structs::{DimensionalNeuronDataFromCorticalArea, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};


pub trait DimensionalNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>:
BaseNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    // TODO are these defaults fine?
    // Neuron Defaults
    const DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING: BurstGlobalIndex<BurstIndexQuant> = BurstGlobalIndex(BurstIndexQuant::ZERO);
    const DEFAULT_NEURON_MEMBRANE_POTENTIAL: NeuronMembranePotential<PotentialQuant> = NeuronMembranePotential(PotentialQuant::ZERO);
    const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<PotentialQuant> = FireThreshold(PotentialQuant::ZERO);
    const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<PercentageQuant> = LeakCoefficient(PercentageQuant::ZERO_PERCENT);
    const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<BurstDeltaQuant> = BurstDelta(BurstDeltaQuant::ZERO);
    const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<BurstDeltaQuant> = BurstDelta(BurstDeltaQuant::ONE);

    // Cortical Area Defaults
    const DEFAULT_CORTICAL_NEURONS_PER_VOXEL: NumberNeuronsPerVoxel = 1;
    const DEFAULT_CORTICAL_EXCITABILITY: NeuronExcitability<PercentageQuant> = NeuronExcitability(PercentageQuant::ZERO_PERCENT);
    const DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT: BurstDelta<BurstDeltaQuant> = BurstDelta(BurstDeltaQuant::ZERO);
    const DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT: FireThresholdLimit<PotentialQuant> = FireThresholdLimit(PotentialQuant::ZERO);
    const DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT: BurstDelta<BurstDeltaQuant> = BurstDelta(BurstDeltaQuant::ZERO);
    const DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED: bool = false;
    const DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED: bool = false;

    /// Returns a struct of references to the slices of all neuron data (include sparse invalids)
    fn get_neuron_values_of_all_dimensional_neuron_cortical_areas_to_process(&mut self) -> DimensionalNeuronDataRefSliceAllCorticalAreas<'_, NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>;

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: CorticalIndexQuant)
                                                                                 -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>, FeagiNPUNeuronError>;

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: CorticalIndexQuant, executor: &impl NeuronFireThresholdExecutor<PotentialQuant, CoordQuant>)
                                 -> Result<(), FeagiNPUNeuronError>;


    // TODO add more specific functions for getting specific fields for neuron processing

}





#[cfg(feature = "alloc")]
pub trait DimensionalNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>:
DimensionalAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> +
DimensionalStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> +
BaseNeuronAllocStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> +
DimensionalNeuronStaticStorageTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    /// Creates a cortical area of given dimensions but using a set of neuron values copied across
    /// all neurons.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_uniform_neurons(&mut self,
                                                 cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 neuron_global_burst_index_of_last_firing: BurstGlobalIndex<BurstIndexQuant>,
                                                 neuron_membrane_potential: NeuronMembranePotential<PotentialQuant>,
                                                 neuron_fire_threshold: FireThreshold<PotentialQuant>,
                                                 neuron_leak_coefficient: LeakCoefficient<PercentageQuant>,
                                                 neuron_refractory_countdown: BurstDelta<BurstDeltaQuant>,
                                                 neuron_consecutive_fire_count: BurstDelta<BurstDeltaQuant>,
                                                 cortical_excitability: NeuronExcitability<PercentageQuant>,
                                                 cortical_refractory_period_limit: BurstDelta<BurstDeltaQuant>,
                                                 cortical_fire_threshold_limit: FireThresholdLimit<PotentialQuant>,
                                                 cortical_consecutive_fire_limit: BurstDelta<BurstDeltaQuant>,
                                                 cortical_is_mp_charge_accumulation_enabled: bool,
                                                 cortical_is_mp_driven_psp_enabled: bool)
                                                 -> Result<(CorticalAreaIndex<CorticalIndexQuant>, Range<NPUNeuronIndex<NeuronIndexQuant>>), FeagiNPUNeuronError>;


    /// Creates a cortical area of given dimensions but using prefilled neuron data values.
    /// Returns the cortical area index and the range of neuron indexes it covers
    fn create_cortical_area_with_individualized_neurons(&mut self,
                                                        cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                        neurons_per_voxel: NumberNeuronsPerVoxel,
                                                        neuron_data: DimensionalNeuronDataFromCorticalArea<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>)
                                                        -> Result<(CorticalAreaIndex<CorticalIndexQuant>, Range<NPUNeuronIndex<NeuronIndexQuant>>), FeagiNPUNeuronError>;
}









// NOTE: The reason we make neuron data flat is because we operate on entire ranges at a time
// we have some usecases where we quickly want to pull a range at a time
// with cortical areas, we generally rarely operate on entire ranges, so keeping those properties
// in structs makes our life easier

use core::ops::Range;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability};
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::neuron::dimensional_neurons::traits::DimensionalNeuronStaticStorageTrait;

/// Stores data as to the property of cortical areas
/// WARNING: Do not allow modification of this struct outside their implemented dimensional_neuron structs, as
/// often values here are tied to other cache values and vice versa!
#[derive(Debug, Clone)]
pub(crate) struct DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    pub flags: DimensionalNeuronCorticalFlag, // NOTE: do not allow modifying this structure outside this
    pub neuron_range: Range<NPUNeuronIndex<NeuronIndexQuant>>,
    pub number_neurons_invalid_from_degeneration: NeuronCount<NeuronIndexQuant>,
    pub dimensions: NeuronVoxelDimensions<CoordQuant>,
    pub number_neurons_per_voxel: NumberNeuronsPerVoxel,
    pub excitability: NeuronExcitability<PercentageQuant>,
    pub refractory_period_limit: BurstDelta<BurstDeltaQuant>,
    pub fire_threshold_limit: FireThresholdLimit<PotentialQuant>,
    pub consecutive_fire_limit: BurstDelta<BurstDeltaQuant>,
}

impl<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant> DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    pub const fn new_default_valid(neuron_range: Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                   voxel_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                   number_neurons_per_voxel: NumberNeuronsPerVoxel) -> Self {
        DimensionalNeuronCorticalData {
            flags: DimensionalNeuronCorticalFlag::new_valid(),
            neuron_range,
            number_neurons_invalid_from_degeneration: NeuronCount::ZERO,
            dimensions: voxel_dimensions,
            number_neurons_per_voxel,
            excitability: DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_EXCITABILITY,
            refractory_period_limit: DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT,
            fire_threshold_limit: DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT,
            consecutive_fire_limit: DimensionalNeuronStaticStorageTrait::DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT,
        }
    }
}


// TODO I took a best guess at which fields should be mutable

/// Used to pass around slices easily at low cost for all cortical areas
pub struct DimensionalNeuronDataRefSliceAllCorticalAreas<'a, NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    pub neuron_cortical_area_index: &'a [CorticalAreaIndex<CorticalIndexQuant>],
    pub neuron_global_burst_index_of_last_firing: &'a mut [BurstGlobalIndex<BurstIndexQuant>],
    pub neuron_membrane_potential: &'a mut [NeuronMembranePotential<PotentialQuant>],
    pub neuron_fire_threshold: &'a mut [FireThreshold<PotentialQuant>],
    pub neuron_leak_coefficient: &'a mut [LeakCoefficient<PercentageQuant>],
    pub neuron_flags: &'a mut [NeuronFlag],
    pub neuron_refractory_countdown: &'a mut [BurstDelta<BurstDeltaQuant>],
    pub neuron_consecutive_fire_count: &'a mut [BurstDelta<BurstDeltaQuant>],

    pub cortical_data: &'a [DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>],
}


/// Used to pass around slices easily at low cost for a single cortical area
pub struct DimensionalNeuronDataRefSliceSingleCorticalArea<'a, NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    pub neuron_cortical_area_index: &'a [CorticalAreaIndex<CorticalIndexQuant>],
    pub neuron_global_burst_index_of_last_firing: &'a mut [BurstGlobalIndex<BurstIndexQuant>],
    pub neuron_membrane_potential: &'a mut [NeuronMembranePotential<PotentialQuant>],
    pub neuron_fire_threshold: &'a mut [FireThreshold<PotentialQuant>],
    pub neuron_leak_coefficient: &'a mut [LeakCoefficient<PercentageQuant>],
    pub neuron_flags: &'a mut [NeuronFlag],
    pub neuron_refractory_countdown: &'a mut [BurstDelta<BurstDeltaQuant>],
    pub neuron_consecutive_fire_count: &'a mut [BurstDelta<BurstDeltaQuant>],

    pub cortical_data: &'a DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>,
    /// Sub-range of global neuron indices covered by the slices above (same shape as [`DimensionalNeuronCorticalData::neuron_range`]).
    pub global_neuron_index_range: Range<NPUNeuronIndex<NeuronIndexQuant>>,
}


/// Used to pass data of neurons to be added or moved for a cortical index
#[cfg(feature = "alloc")]
pub struct DimensionalNeuronDataFromCorticalArea<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    pub neuron_global_burst_index_of_last_firing: Vec<BurstGlobalIndex<BurstIndexQuant>>,
    pub neuron_membrane_potential: Vec<NeuronMembranePotential<PotentialQuant>>,
    pub neuron_fire_threshold: Vec<FireThreshold<PotentialQuant>>,
    pub neuron_leak_coefficient: Vec<LeakCoefficient<PercentageQuant>>,
    pub neuron_flags: Vec<NeuronFlag>,
    pub neuron_refractory_countdown: Vec<BurstDelta<BurstDeltaQuant>>,
    pub neuron_consecutive_fire_count: Vec<BurstDelta<BurstDeltaQuant>>,

    pub cortical_data: DimensionalNeuronCorticalData<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, PotentialQuant, PercentageQuant>,
}
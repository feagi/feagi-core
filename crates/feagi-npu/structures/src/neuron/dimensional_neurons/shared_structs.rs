
// NOTE: The reason we make neuron data flat is because we operate on entire ranges at a time
// we have some usecases where we quickly want to pull a range at a time
// with cortical areas, we generally rarely operate on entire ranges, so keeping those properties
// in structs makes our life easier

use core::ops::Range;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use feagi_structures::useful_structs::indexed_data_tracker;
use crate::neuron::defaults::DimensionalNeuronDefaults;
use crate::quantizables::{NPUGlobalQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability};
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct DimensionalTypedNeuronIndex<T: QuantizableUIntType> {
    pub index: NPUNeuronIndex<T>,
    pub dimensional_type: DimensionCorticalAreaType
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct DimensionalTypedCorticalIndex<T: QuantizableUIntType> {
    pub index: CorticalAreaIndex<T>,
    pub dimensional_type: DimensionCorticalAreaType
}


/// Stores data as to the property of cortical areas
/// WARNING: Do not allow modification of this struct outside their implemented dimensional_neuron structs, as
/// often values here are tied to other cache values and vice versa!
#[derive(Debug, Clone)]
pub struct DimensionalNeuronCorticalData<Q: NPUGlobalQuantization>
{
    pub flags: DimensionalNeuronCorticalFlag, // NOTE: do not allow modifying this structure outside this
    pub neuron_range: Range<NPUNeuronIndex<Q::NeuronIndexQuant>>,
    pub number_neurons_invalid_from_degeneration: NeuronCount<Q::NeuronIndexQuant>,
    pub dimensions: NeuronVoxelDimensions<Q::CoordQuantQuant>,
    pub number_neurons_per_voxel: NumberNeuronsPerVoxel,
    pub excitability: NeuronExcitability<Q::PercentageQuant>,
    pub refractory_period_limit: BurstDelta<Q::BurstDeltaQuant>,
    pub fire_threshold_limit: FireThresholdLimit<Q::ValueQuant>,
    pub consecutive_fire_limit: BurstDelta<Q::BurstDeltaQuant>,
}

impl<Q: NPUGlobalQuantization> DimensionalNeuronCorticalData<Q>
{

    pub const fn new_default_valid<D: DimensionalNeuronDefaults<Q>>(neuron_range: Range<NPUNeuronIndex<Q::NeuronIndexQuant>>,
                                                                    voxel_dimensions: NeuronVoxelDimensions<Q::CoordQuantQuant>,
                                                                    number_neurons_per_voxel: NumberNeuronsPerVoxel) -> Self {
        DimensionalNeuronCorticalData {
            flags: DimensionalNeuronCorticalFlag::new_valid(),
            neuron_range,
            number_neurons_invalid_from_degeneration: NeuronCount::ZERO,
            dimensions: voxel_dimensions,
            number_neurons_per_voxel,
            excitability: D::DEFAULT_CORTICAL_EXCITABILITY,
            refractory_period_limit: D::DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT,
            fire_threshold_limit: D::DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT,
            consecutive_fire_limit: D::DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT,
        }
    }
}


// TODO I took a best guess at which fields should be mutable

/// Used to pass around slices easily at low cost for all cortical areas
pub struct DimensionalNeuronDataRefSliceAllCorticalAreas<'a, Q: NPUGlobalQuantization>
{
    pub neuron_cortical_area_index: &'a [CorticalAreaIndex<Q::CorticalIndexQuant>],
    pub neuron_global_burst_index_of_last_firing: &'a mut [BurstGlobalIndex<Q::GlobalBurstIndexQuant>],
    pub neuron_membrane_potential: &'a mut [NPUNeuronMembranePotential<Q::ValueQuant>],
    pub neuron_fire_threshold: &'a mut [FireThreshold<Q::ValueQuant>],
    pub neuron_leak_coefficient: &'a mut [LeakCoefficient<Q::PercentageQuant>],
    pub neuron_flags: &'a mut [NeuronFlag],
    pub neuron_refractory_countdown: &'a mut [BurstDelta<Q::BurstDeltaQuant>],
    pub neuron_consecutive_fire_count: &'a mut [BurstDelta<Q::BurstDeltaQuant>],

    pub cortical_data: &'a IndexedDataTracker<DimensionalNeuronCorticalData<Q>, CorticalAreaIndex<Q::CorticalIndexQuant>>,
}


/// Used to pass around slices easily at low cost for a single cortical area
pub struct DimensionalNeuronDataRefSliceSingleCorticalArea<'a, Q: NPUGlobalQuantization>
{
    pub neuron_cortical_area_index: &'a [CorticalAreaIndex<Q::CorticalIndexQuant>],
    pub neuron_global_burst_index_of_last_firing: &'a mut [BurstGlobalIndex<Q::GlobalBurstIndexQuant>],
    pub neuron_membrane_potential: &'a mut [NPUNeuronMembranePotential<Q::ValueQuant>],
    pub neuron_fire_threshold: &'a mut [FireThreshold<Q::ValueQuant>],
    pub neuron_leak_coefficient: &'a mut [LeakCoefficient<Q::PercentageQuant>],
    pub neuron_flags: &'a mut [NeuronFlag],
    pub neuron_refractory_countdown: &'a mut [BurstDelta<Q::BurstDeltaQuant>],
    pub neuron_consecutive_fire_count: &'a mut [BurstDelta<Q::BurstDeltaQuant>],

    pub cortical_data: &'a DimensionalNeuronCorticalData<Q>,
    /// Sub-range of global neuron indices covered by the slices above (same shape as [`DimensionalNeuronCorticalData::neuron_range`]).
    pub global_neuron_index_range: Range<NPUNeuronIndex<Q::NeuronIndexQuant>>,
}


/// Used to pass data of neurons to be added or moved for a cortical index
#[cfg(feature = "alloc")]
pub struct DimensionalNeuronDataFromCorticalArea<Q: NPUGlobalQuantization>
{
    pub neuron_global_burst_index_of_last_firing: Vec<BurstGlobalIndex<Q::GlobalBurstIndexQuant>>,
    pub neuron_membrane_potential: Vec<NPUNeuronMembranePotential<Q::ValueQuant>>,
    pub neuron_fire_threshold: Vec<FireThreshold<Q::ValueQuant>>,
    pub neuron_leak_coefficient: Vec<LeakCoefficient<Q::PercentageQuant>>,
    pub neuron_flags: Vec<NeuronFlag>,
    pub neuron_refractory_countdown: Vec<BurstDelta<Q::BurstDeltaQuant>>,
    pub neuron_consecutive_fire_count: Vec<BurstDelta<Q::BurstDeltaQuant>>,

    pub cortical_data: DimensionalNeuronCorticalData<Q>,
}

use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::NeuronCount;
use crate::neuron::dimensional_neurons::shared_structs::DimensionalNeuronCorticalData;
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::quantizables::{BurstDelta, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NPUQuantization, NeuronExcitability};

pub const NUMBER_SINGLE_NEURON_CORE_AREAS: usize  = 3;

pub(crate) struct CoreNeuronPowerDefaults;

impl<Q: NPUQuantization> CoreNeuronPowerDefaults {
    pub const NEURON_INDEX: NPUNeuronIndex<Q::NeuronIndex> = NPUNeuronIndex::ZERO;
    pub const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::Value> = FireThreshold::ZERO;
    pub const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::Value> = LeakCoefficient::ZERO_PERCENT;
    pub const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDelta> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDelta> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_FLAG: NeuronFlag = NeuronFlag::new_valid();
    pub const DEFAULT_CORTICAL_DATA: DimensionalNeuronCorticalData<Q> = {
        DimensionalNeuronCorticalData{
            flags: {
                let mut flag = DimensionalNeuronCorticalFlag::new_valid();
                flag.set_mp_driven_psp_enabled(false);
                flag.set_mp_charge_accumulation_enabled(false);
                flag
            },
            neuron_range: Self::NEURON_INDEX..Self::NEURON_INDEX + NPUNeuronIndex::ONE,
            number_neurons_invalid_from_degeneration: NeuronCount::ZERO,
            dimensions: NeuronVoxelDimensions::new_cube(1).unwrap(),
            number_neurons_per_voxel: 1,
            excitability: NeuronExcitability::HUNDRED_PERCENT,
            refractory_period_limit: BurstDelta::ZERO,
            fire_threshold_limit: FireThresholdLimit::ZERO,
            consecutive_fire_limit: BurstDelta::ZERO,
        }
    };
}

pub(crate) struct CoreNeuronDeathDefaults;

impl<Q: NPUQuantization> CoreNeuronDeathDefaults {
    pub const NEURON_INDEX: NPUNeuronIndex<Q::NeuronIndex> = NPUNeuronIndex::ONE;
    pub const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::Value> = FireThreshold::ZERO;
    pub const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::Value> = LeakCoefficient::ZERO_PERCENT;
    pub const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDelta> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDelta> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_FLAG: NeuronFlag = NeuronFlag::new_valid();
    pub const DEFAULT_CORTICAL_DATA: DimensionalNeuronCorticalData<Q> = {
        DimensionalNeuronCorticalData{
            flags: {
                let mut flag = DimensionalNeuronCorticalFlag::new_valid();
                flag.set_mp_driven_psp_enabled(false);
                flag.set_mp_charge_accumulation_enabled(false);
                flag
            },
            neuron_range: Self::NEURON_INDEX..Self::NEURON_INDEX + NPUNeuronIndex::ONE,
            number_neurons_invalid_from_degeneration: NeuronCount::ZERO,
            dimensions: NeuronVoxelDimensions::new_cube(1).unwrap(),
            number_neurons_per_voxel: 1,
            excitability: NeuronExcitability::HUNDRED_PERCENT,
            refractory_period_limit: BurstDelta::ZERO,
            fire_threshold_limit: FireThresholdLimit::ZERO,
            consecutive_fire_limit: BurstDelta::ZERO,
        }
    };
}


pub(crate) struct CoreNeuronFatigueDefaults;

impl<Q: NPUQuantization> CoreNeuronFatigueDefaults {
    pub const NEURON_INDEX: NPUNeuronIndex<Q::NeuronIndex> = NPUNeuronIndex::ONE + NPUNeuronIndex::ONE;
    pub const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::Value> = FireThreshold::ZERO;
    pub const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::Value> = LeakCoefficient::ZERO_PERCENT;
    pub const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDelta> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDelta> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_FLAG: NeuronFlag = NeuronFlag::new_valid();
    pub const DEFAULT_CORTICAL_DATA: DimensionalNeuronCorticalData<Q> = {
        DimensionalNeuronCorticalData{
            flags: {
                let mut flag = DimensionalNeuronCorticalFlag::new_valid();
                flag.set_mp_driven_psp_enabled(false);
                flag.set_mp_charge_accumulation_enabled(false);
                flag
            },
            neuron_range: Self::NEURON_INDEX..Self::NEURON_INDEX + NPUNeuronIndex::ONE,
            number_neurons_invalid_from_degeneration: NeuronCount::ZERO,
            dimensions: NeuronVoxelDimensions::new_cube(1).unwrap(),
            number_neurons_per_voxel: 1,
            excitability: NeuronExcitability::HUNDRED_PERCENT,
            refractory_period_limit: BurstDelta::ZERO,
            fire_threshold_limit: FireThresholdLimit::ZERO,
            consecutive_fire_limit: BurstDelta::ZERO,
        }
    };
}
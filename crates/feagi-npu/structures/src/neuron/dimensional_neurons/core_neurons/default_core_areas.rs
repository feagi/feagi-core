use core::marker::PhantomData;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::NeuronCount;
use crate::neuron::dimensional_neurons::shared_structs::DimensionalNeuronCorticalData;
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::quantizables::{BurstDelta, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NPUQuantization, NeuronExcitability};

// This file can be used to define the default values of Core cortical areas

pub const NUMBER_SINGLE_NEURON_CORE_AREAS: usize  = 3;

pub(crate) struct CoreNeuronPowerDefaults<Q: NPUQuantization> {
    phantom_data: PhantomData<Q>
}

impl<Q: NPUQuantization> CoreNeuronPowerDefaults<Q> {
    pub const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::ValueQuant> = FireThreshold::ZERO;
    pub const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::PercentageQuant> = LeakCoefficient::ZERO_PERCENT;
    pub const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDeltaQuant> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDeltaQuant> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_FLAG: NeuronFlag = NeuronFlag::new_default_core();
    
    #[inline(always)]
    pub fn get_default_cortical_data() -> DimensionalNeuronCorticalData<Q> {
        DimensionalNeuronCorticalData{
            flags: {
                let mut flag = DimensionalNeuronCorticalFlag::new_valid();
                flag.set_mp_driven_psp_enabled(false);
                flag.set_mp_charge_accumulation_enabled(false);
                flag
            },
            neuron_range: NPUNeuronIndex::DEFAULT_CORE_POWER..NPUNeuronIndex::DEFAULT_CORE_POWER + NPUNeuronIndex::ONE,
            number_neurons_invalid_from_degeneration: NeuronCount::ZERO,
            dimensions: NeuronVoxelDimensions::default_1_1_1_cube(),
            number_neurons_per_voxel: 1,
            excitability: NeuronExcitability::HUNDRED_PERCENT,
            refractory_period_limit: BurstDelta::ZERO,
            fire_threshold_limit: FireThresholdLimit::ZERO,
            consecutive_fire_limit: BurstDelta::ZERO,
        }
    }
}

pub(crate) struct CoreNeuronDeathDefaults<Q: NPUQuantization> {
    phantom_data: PhantomData<Q>
}

impl<Q: NPUQuantization> CoreNeuronDeathDefaults<Q> {
    pub const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::ValueQuant> = FireThreshold::ZERO;
    pub const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::PercentageQuant> = LeakCoefficient::ZERO_PERCENT;
    pub const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDeltaQuant> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDeltaQuant> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_FLAG: NeuronFlag = NeuronFlag::new_default_core();
    
    #[inline(always)]
    pub fn get_default_cortical_data() -> DimensionalNeuronCorticalData<Q> {
        DimensionalNeuronCorticalData{
            flags: {
                let mut flag = DimensionalNeuronCorticalFlag::new_valid();
                flag.set_mp_driven_psp_enabled(false);
                flag.set_mp_charge_accumulation_enabled(false);
                flag
            },
            neuron_range: NPUNeuronIndex::DEFAULT_CORE_DEATH..NPUNeuronIndex::DEFAULT_CORE_DEATH + NPUNeuronIndex::ONE,
            number_neurons_invalid_from_degeneration: NeuronCount::ZERO,
            dimensions: NeuronVoxelDimensions::default_1_1_1_cube(),
            number_neurons_per_voxel: 1,
            excitability: NeuronExcitability::HUNDRED_PERCENT,
            refractory_period_limit: BurstDelta::ZERO,
            fire_threshold_limit: FireThresholdLimit::ZERO,
            consecutive_fire_limit: BurstDelta::ZERO,
        }
    }
}


pub(crate) struct CoreNeuronFatigueDefaults<Q: NPUQuantization> {
    phantom_data: PhantomData<Q>
}

impl<Q: NPUQuantization> CoreNeuronFatigueDefaults<Q> {
    pub const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::ValueQuant> = FireThreshold::ZERO;
    pub const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::PercentageQuant> = LeakCoefficient::ZERO_PERCENT;
    pub const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDeltaQuant> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDeltaQuant> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_FLAG: NeuronFlag = NeuronFlag::new_default_core();
    
    #[inline(always)]
    pub fn get_default_cortical_data() -> DimensionalNeuronCorticalData<Q> {
        DimensionalNeuronCorticalData{
            flags: {
                let mut flag = DimensionalNeuronCorticalFlag::new_valid();
                flag.set_mp_driven_psp_enabled(false);
                flag.set_mp_charge_accumulation_enabled(false);
                flag
            },
            neuron_range: NPUNeuronIndex::DEFAULT_CORE_FATIGUE..NPUNeuronIndex::DEFAULT_CORE_FATIGUE + NPUNeuronIndex::ONE,
            number_neurons_invalid_from_degeneration: NeuronCount::ZERO,
            dimensions: NeuronVoxelDimensions::default_1_1_1_cube(),
            number_neurons_per_voxel: 1,
            excitability: NeuronExcitability::HUNDRED_PERCENT,
            refractory_period_limit: BurstDelta::ZERO,
            fire_threshold_limit: FireThresholdLimit::ZERO,
            consecutive_fire_limit: BurstDelta::ZERO,
        }
    }
}
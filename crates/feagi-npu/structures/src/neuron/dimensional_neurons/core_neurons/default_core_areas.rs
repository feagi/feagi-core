use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use crate::quantizables::{BurstDelta, FireThreshold, LeakCoefficient, NPUQuantization};

pub(crate) struct CoreNeuronPowerDefaults;

impl<Q: NPUQuantization> CoreNeuronPowerDefaults {
    pub const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::Value> = FireThreshold::ZERO;
    pub const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::Value> = LeakCoefficient::ZERO_PERCENT;
    pub const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDelta> = BurstDelta::ZERO;
    pub const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDelta> = BurstDelta::ZERO;
    pub const DEFAULT_
}
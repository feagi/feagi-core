use core::marker::PhantomData;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
use crate::quantizables::{NPUQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronMembranePotential, NeuronExcitability};

//region Dimension Neurons

// NOTE: Core has special handling!

pub(crate) trait DimensionalNeuronDefaults<Q: NPUQuantization> {
    // Neuron Defaults
    const DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING: BurstGlobalIndex<Q::BurstIndex>;
    const DEFAULT_NEURON_MEMBRANE_POTENTIAL: NPUNeuronMembranePotential<Q::Value>;
    const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::Value>;
    const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::Percentage>;
    const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDelta>;
    const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDelta>;

    // Cortical Area Defaults
    const DEFAULT_CORTICAL_NEURONS_PER_VOXEL: NumberNeuronsPerVoxel;
    const DEFAULT_CORTICAL_EXCITABILITY: NeuronExcitability<Q::Percentage>;
    const DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT: BurstDelta<Q::BurstDelta>;
    const DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT: FireThresholdLimit<Q::Value>;
    const DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT: BurstDelta<Q::BurstDelta>;
    const DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED: bool;
    const DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED: bool;
}

pub(crate) struct SensoryNeuronDefaults<Q: NPUQuantization>(PhantomData<Q>);
pub(crate) struct MotorNeuronsDefaults<Q: NPUQuantization>(PhantomData<Q>);
pub(crate) struct InterNeuronsDefaults<Q: NPUQuantization>(PhantomData<Q>);

macro_rules! impl_dimensional_neuron_defaults {
    ($defaults_type:ident) => {
        impl<Q: NPUQuantization> DimensionalNeuronDefaults<Q> for $defaults_type<Q> {
            // TODO are these defaults fine?
            // Neuron Defaults
            const DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING: BurstGlobalIndex<Q::BurstIndex> =
                BurstGlobalIndex(Q::BurstIndex::ZERO);
            const DEFAULT_NEURON_MEMBRANE_POTENTIAL: NPUNeuronMembranePotential<Q::Value> =
                NPUNeuronMembranePotential(Q::Value::ZERO);
            const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::Value> =
                FireThreshold(Q::Value::ZERO);
            const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::Percentage> =
                LeakCoefficient(Q::Percentage::ZERO_PERCENT);
            const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDelta> =
                BurstDelta(Q::BurstDelta::ZERO);
            const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDelta> =
                BurstDelta(Q::BurstDelta::ONE);

            // Cortical Area Defaults
            const DEFAULT_CORTICAL_NEURONS_PER_VOXEL: NumberNeuronsPerVoxel = 1;
            const DEFAULT_CORTICAL_EXCITABILITY: NeuronExcitability<Q::Percentage> =
                NeuronExcitability(Q::Percentage::ZERO_PERCENT);
            const DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT: BurstDelta<Q::BurstDelta> =
                BurstDelta(Q::BurstDelta::ZERO);
            const DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT: FireThresholdLimit<Q::Value> =
                FireThresholdLimit(Q::Value::ZERO);
            const DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT: BurstDelta<Q::BurstDelta> =
                BurstDelta(Q::BurstDelta::ZERO);
            const DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED: bool = false;
            const DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED: bool = false;
        }

        impl<Q: NPUQuantization> $defaults_type<Q> {
            pub(crate) const DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING: BurstGlobalIndex<Q::BurstIndex> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_NEURON_GLOBAL_BURST_INDEX_OF_LAST_FIRING;
            pub(crate) const DEFAULT_NEURON_MEMBRANE_POTENTIAL: NPUNeuronMembranePotential<Q::Value> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_NEURON_MEMBRANE_POTENTIAL;
            pub(crate) const DEFAULT_NEURON_FIRE_THRESHOLD: FireThreshold<Q::Value> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_NEURON_FIRE_THRESHOLD;
            pub(crate) const DEFAULT_NEURON_LEAK_COEFFICIENT: LeakCoefficient<Q::Percentage> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_NEURON_LEAK_COEFFICIENT;
            pub(crate) const DEFAULT_NEURON_REFRACTORY_COUNTDOWN: BurstDelta<Q::BurstDelta> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_NEURON_REFRACTORY_COUNTDOWN;
            pub(crate) const DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT: BurstDelta<Q::BurstDelta> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT;

            pub(crate) const DEFAULT_CORTICAL_NEURONS_PER_VOXEL: NumberNeuronsPerVoxel =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_CORTICAL_NEURONS_PER_VOXEL;
            pub(crate) const DEFAULT_CORTICAL_EXCITABILITY: NeuronExcitability<Q::Percentage> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_CORTICAL_EXCITABILITY;
            pub(crate) const DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT: BurstDelta<Q::BurstDelta> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_CORTICAL_REFRACTORY_PERIOD_LIMIT;
            pub(crate) const DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT: FireThresholdLimit<Q::Value> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_CORTICAL_FIRE_THRESHOLD_LIMIT;
            pub(crate) const DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT: BurstDelta<Q::BurstDelta> =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_CORTICAL_CONSECUTIVE_FIRE_LIMIT;
            pub(crate) const DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED: bool =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_CORTICAL_IS_MP_CHARGE_ACCUMULATION_ENABLED;
            pub(crate) const DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED: bool =
                <Self as DimensionalNeuronDefaults<Q>>::DEFAULT_CORTICAL_IS_MP_DRIVEN_PSP_ENABLED;
        }
    };
}

impl_dimensional_neuron_defaults!(SensoryNeuronDefaults);
impl_dimensional_neuron_defaults!(MotorNeuronsDefaults);
impl_dimensional_neuron_defaults!(InterNeuronsDefaults);

//endregion

use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUDimensionalNeuronQuantization, NPUGlobalQuantization, NPUNeuronMembranePotential, NeuronExcitability};

// TODO Outside of maybe unified verify functions, do not group these in traits as they are all specific

pub struct FeagiStandardUniformNeuronLoader<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {
    pub cortical_area_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
    pub neurons_per_voxel: NumberNeuronsPerVoxel,
    pub neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::GlobalBurstIndexQuant>,
    pub neuron_membrane_potential: NPUNeuronMembranePotential<DNQ::ValueQuant>,
    pub neuron_fire_threshold: FireThreshold<DNQ::ValueQuant>,
    pub neuron_leak_coefficient: LeakCoefficient<DNQ::PercentageQuant>,
    pub neuron_refractory_countdown: BurstDelta<DNQ::BurstDeltaQuant>,
    pub neuron_consecutive_fire_count: BurstDelta<DNQ::BurstDeltaQuant>,
    pub cortical_excitability: NeuronExcitability<DNQ::PercentageQuant>,
    pub cortical_refractory_period_limit: BurstDelta<DNQ::BurstDeltaQuant>,
    pub cortical_fire_threshold_limit: FireThresholdLimit<DNQ::ValueQuant>,
    pub cortical_consecutive_fire_limit: BurstDelta<DNQ::BurstDeltaQuant>,
    pub cortical_is_mp_charge_accumulation_enabled: bool,
    pub cortical_is_mp_driven_psp_enabled: bool
}



pub struct FeagiStandardFullNeuronLoader<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {
    cortical_area_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
    neurons_per_voxel: NumberNeuronsPerVoxel,
    neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::GlobalBurstIndexQuant>, // delete me
    // TODO vectors, verification function
}
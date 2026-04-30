

// NOTE: Data should be flat as it is optimal in certain common operations

use feagi_structures::define_ref_access_trait_methods;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUGlobalQuantization, NPUDimensionalNeuronQuantization, NeuronExcitability};

/// Defines the base data (both cortical settings and neuron data) shared by all dimensional cortical areas
pub(crate) trait DimensionalNeuronDataArrayTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    type DimensionalCorticalConfigurationQuant: DimensionalCorticalConfigurationTrait<Q, DNQ>;

    define_ref_access_trait_methods!(dimensional_cortical_configuration, Self::DimensionalCorticalConfigurationQuant);
    define_ref_access_trait_methods!(neuron_global_burst_index_of_last_firing, [BurstGlobalIndex<Q::GlobalBurstIndexQuant>]);
    define_ref_access_trait_methods!(neuron_membrane_potential, [NeuronMembranePotential<DNQ::ValueQuant>]);
    define_ref_access_trait_methods!(neuron_fire_threshold, [FireThreshold<DNQ::ValueQuant>]);
    define_ref_access_trait_methods!(neuron_leak_coefficient, [LeakCoefficient<DNQ::PercentageQuant>]);
    define_ref_access_trait_methods!(neuron_flags, [NeuronFlag]);
    define_ref_access_trait_methods!(neuron_refractory_countdown, [BurstDelta<DNQ::BurstDeltaQuant>]);
    define_ref_access_trait_methods!(neuron_consecutive_fire_countdown, [BurstDelta<DNQ::BurstDeltaQuant>]);
}


pub(crate) trait DimensionalNeuronDataVectorTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronDataArrayTrait<Q, DNQ>
{
    fn resize_cortical_area(&mut self, dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>, density: NumberNeuronsPerVoxel);
}

/// Defines the base cortical settings shared by all dimensional cortical areas
pub(crate) trait DimensionalCorticalConfigurationTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    define_ref_access_trait_methods!(cortical_flag, DimensionalNeuronCorticalFlag);
    define_ref_access_trait_methods!(cortical_dimensions, NeuronVoxelDimensions<DNQ::CoordQuant>);
    define_ref_access_trait_methods!(number_neurons_per_voxel, NeuronCount<NumberNeuronsPerVoxel>);
    define_ref_access_trait_methods!(number_neurons_invalid_from_degeneration, NeuronCount<DNQ::NeuronCountQuant>);
    define_ref_access_trait_methods!(excitability, NeuronExcitability<DNQ::PercentageQuant>);
    define_ref_access_trait_methods!(refractory_period_limit, BurstDelta<DNQ::BurstDeltaQuant>);
    define_ref_access_trait_methods!(fire_threshold_limit, FireThresholdLimit<DNQ::ValueQuant>);
    define_ref_access_trait_methods!(consecutive_fire_limit, BurstDelta<DNQ::BurstDeltaQuant>);
}


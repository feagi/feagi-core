

// NOTE: Data should be flat as it is optimal in certain common operations

use feagi_structures::define_ref_access_trait_methods;
use feagi_structures::neurons::descriptors::{NeuronMembranePotential};
use crate::neuron::flags::{NeuronFlag};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, LeakCoefficient, NPUGlobalQuantization, NPUDimensionalNeuronQuantization};

/// Defines the base data (both cortical settings and neuron data) shared by all dimensional cortical areas
pub(crate) trait DimensionalNeuronModelDataSharedTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    define_ref_access_trait_methods!(neuron_global_burst_index_of_last_firing, [BurstGlobalIndex<Q::GlobalBurstIndexQuant>]);
    define_ref_access_trait_methods!(neuron_membrane_potential, [NeuronMembranePotential<DNQ::ValueQuant>]);
    define_ref_access_trait_methods!(neuron_fire_threshold, [FireThreshold<DNQ::ValueQuant>]);
    define_ref_access_trait_methods!(neuron_leak_coefficient, [LeakCoefficient<DNQ::PercentageQuant>]);
    define_ref_access_trait_methods!(neuron_flags, [NeuronFlag]);
    define_ref_access_trait_methods!(neuron_refractory_countdown, [BurstDelta<DNQ::BurstDeltaQuant>]);
    define_ref_access_trait_methods!(neuron_consecutive_fire_countdown, [BurstDelta<DNQ::BurstDeltaQuant>]);
}

pub(crate) trait DimensionalNeuronModelDataFixedTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronModelDataSharedTrait<Q, DNQ>
{

}

pub(crate) trait DimensionalNeuronModelDataResizableTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronModelDataSharedTrait<Q, DNQ>
{

}



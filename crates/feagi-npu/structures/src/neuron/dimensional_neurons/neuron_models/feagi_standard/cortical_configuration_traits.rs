use crate::neuron::dimensional_neurons::neuron_models::dimensional_cortical_configuration_traits::DimensionalCorticalConfigurationTrait;
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};


pub(crate) trait FeagiStandardCorticalConfigurationTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalCorticalConfigurationTrait<Q, DNQ>
{
    // Something may be specific to here, include it here
}
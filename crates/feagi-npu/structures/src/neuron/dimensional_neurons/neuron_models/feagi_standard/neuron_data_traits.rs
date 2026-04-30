use crate::neuron::dimensional_neurons::neuron_models::dimensional_neuron_data_traits::{DimensionalNeuronModelDataFixedTrait, DimensionalNeuronModelDataResizableTrait, DimensionalNeuronModelDataSharedTrait};
use crate::neuron::dimensional_neurons::neuron_models::feagi_standard::cortical_configuration_traits::FeagiStandardCorticalConfigurationTrait;
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};


pub(crate) trait FeagiStandardNeuronModelDataSharedTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronModelDataSharedTrait<Q, DNQ>
{
    fn get_feagi_standard_cortical_configuration_impl(&self) -> &impl FeagiStandardCorticalConfigurationTrait<Q, DNQ>;

    fn get_feagi_standard_cortical_configuration_impl_mut(&mut self) -> &mut impl FeagiStandardCorticalConfigurationTrait<Q, DNQ>;
}

pub(crate) trait FeagiStandardNeuronModelDataFixedTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
FeagiStandardNeuronModelDataSharedTrait<Q, DNQ> +
DimensionalNeuronModelDataFixedTrait<Q, DNQ>
{

}

pub(crate) trait FeagiStandardNeuronModelDataResizableTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
FeagiStandardNeuronModelDataSharedTrait<Q, DNQ> +
DimensionalNeuronModelDataResizableTrait<Q, DNQ>
{
    
}
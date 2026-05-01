use crate::neuron::neuron_models::dimensional_models::dimensional_neuron_data_traits::{DimensionalNeuronModelDataFixedTrait, DimensionalNeuronModelDataResizableTrait, DimensionalNeuronModelDataSharedTrait};
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_area_generator_traits::DimensionalCorticalAreaGeneratorTrait;
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_configuration_traits::DimensionalCorticalConfigurationTrait;
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};

//region Neuron Model Trait
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

#[cfg(feature = "alloc")]
pub(crate) trait FeagiStandardNeuronModelDataResizableTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
FeagiStandardNeuronModelDataSharedTrait<Q, DNQ> +
DimensionalNeuronModelDataResizableTrait<Q, DNQ>
{

}
//endregion

//region Cortical Configuration Trait
pub(crate) trait FeagiStandardCorticalConfigurationTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalCorticalConfigurationTrait<Q, DNQ>
{
    // Something may be specific to here, include it here
}
//endregion

//region Cortical Area Generator

#[cfg(feature = "alloc")]
pub trait FeagiStandardCorticalAreaGenerator<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalCorticalAreaGeneratorTrait<Q, DNQ>
{
    
}

//endregion
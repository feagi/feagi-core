
// NOTE: Data should be flat as it is optimal in certain common operations

use feagi_structures::define_ref_access_trait_methods;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential};
use crate::neuron::dimensional_neurons::neuron_models::dimensional_cortical_area_generator_traits::DimensionalCorticalAreaGeneratorTrait;
use crate::neuron::dimensional_neurons::neuron_models::DimensionalCorticalConfigurationTrait;
use crate::neuron::flags::{NeuronFlag};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, LeakCoefficient, NPUGlobalQuantization, NPUDimensionalNeuronQuantization};

/// Defines the base data (both cortical settings and neuron data) shared by all dimensional cortical areas
pub(crate) trait DimensionalNeuronModelDataSharedTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    type DimensionalCorticalConfigurationType: DimensionalCorticalConfigurationTrait<Q, DNQ>;
    define_ref_access_trait_methods!(cortical_data, Self::DimensionalCorticalConfigurationType);
    define_ref_access_trait_methods!(neuron_global_burst_index_of_last_firing, [BurstGlobalIndex<Q::GlobalBurstIndexQuant>]);
    define_ref_access_trait_methods!(neuron_membrane_potential, [NeuronMembranePotential<DNQ::ValueQuant>]);
    define_ref_access_trait_methods!(neuron_fire_threshold, [FireThreshold<DNQ::ValueQuant>]);
    define_ref_access_trait_methods!(neuron_leak_coefficient, [LeakCoefficient<DNQ::PercentageQuant>]);
    define_ref_access_trait_methods!(neuron_flags, [NeuronFlag]);
    define_ref_access_trait_methods!(neuron_refractory_countdown, [BurstDelta<DNQ::BurstDeltaQuant>]);
    define_ref_access_trait_methods!(neuron_consecutive_fire_countdown, [BurstDelta<DNQ::BurstDeltaQuant>]);

    /// Returns the total number of neurons in the Dimensional cortical area.
    /// Includes both live and dead neurons
   fn get_total_number_neurons(&self) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
       self.get_cortical_data().get_total_number_neurons()
   }
}

pub(crate) trait DimensionalNeuronModelDataFixedTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronModelDataSharedTrait<Q, DNQ>
{

}

pub(crate) trait DimensionalNeuronModelDataResizableTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>:
DimensionalNeuronModelDataSharedTrait<Q, DNQ>
{
    type DimensionalUniformNeuronLoaderType: DimensionalCorticalAreaGeneratorTrait<Q, DNQ>;
    fn overwrite_dead_self_with_uniform_neuron_loader(&mut self, uniform_loader: Self::DimensionalUniformNeuronLoaderType);
    // TODO other neuron loader types?

}



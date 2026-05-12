

// NOTE: Data should be flat as it is optimal in certain common operations

use feagi_structures::define_ref_access_trait_methods;
use feagi_structures::genomic::cortical_area::CorticalAreaModelType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_area_generator_traits::DimensionalCorticalAreaGeneratorTrait;
use crate::neuron::flags::{NeuronFlag};
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_configuration_traits::DimensionalCorticalConfigurationTrait;
use crate::neuron::neuron_models::dimensional_models::feagi_standard::FeagiStandardCorticalAreaGenerator;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, LeakCoefficient, NPUGlobalQuantization, NPUDimensionalNeuronQuantization};

// TODO Multiple data models for Memory as well? I so we need a super trait

// Each model needs its own cortical flag struct, their own neuron flag struct.
// the only commonality between all neuron models is the existence of a neuron potential, fire threshold, neuron flags (of type)

/// Defines the base data (both cortical settings and neuron data) shared by all dimensional cortical areas
pub(crate) trait DimensionalNeuronModelDataSharedTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    /// Gets the model type as an enum
    const CORTICAL_AREA_MODEL_TYPE: CorticalAreaModelType;
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
   fn get_total_number_neurons(&self) -> NeuronCount<Q::NeuronIndexCountQuant> {
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
    /// Resizes internal vectors for new size,
    /// ONLY TO BE CALLED BY 'DimensionalCorticalAreaResizerTrait' IMPLEMENTATIONS!
    fn resize_neuron_data_vectors_for_new_dimensions(&mut self,
                                                     new_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
                                                     neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>);
}



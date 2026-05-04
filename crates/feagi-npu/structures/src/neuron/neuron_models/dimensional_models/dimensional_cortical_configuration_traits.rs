use feagi_structures::define_ref_access_trait_methods;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::neuron::flags::DimensionalNeuronCorticalFlag;
use crate::quantizables::{BurstDelta, FireThresholdLimit, NPUDimensionalNeuronQuantization, NPUGlobalQuantization, NeuronExcitability};


/// Defines the base cortical setting logic shared by all dimensional cortical areas
pub(crate) trait DimensionalCorticalConfigurationTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    define_ref_access_trait_methods!(cortical_flag, DimensionalNeuronCorticalFlag);
    define_ref_access_trait_methods!(cortical_dimensions, NeuronVoxelDimensions<DNQ::CoordQuant>);
    define_ref_access_trait_methods!(number_neurons_per_voxel, NeuronCount<NumberNeuronsPerVoxel>);
    define_ref_access_trait_methods!(number_neurons_invalid_from_degeneration, NeuronCount<Q::NeuronIndexCountQuant>);
    define_ref_access_trait_methods!(excitability, NeuronExcitability<DNQ::PercentageQuant>);
    define_ref_access_trait_methods!(refractory_period_limit, BurstDelta<DNQ::BurstDeltaQuant>);
    define_ref_access_trait_methods!(fire_threshold_limit, FireThresholdLimit<DNQ::ValueQuant>);
    define_ref_access_trait_methods!(consecutive_fire_limit, BurstDelta<DNQ::BurstDeltaQuant>);

    fn get_total_number_neurons(&self) -> NeuronCount<Q::NeuronIndexCountQuant> {
        self.get_cortical_dimensions().get_number_neurons(self.get_number_neurons_per_voxel())
    }
    
    #[inline]
    fn get_cortical_dimensions_usize(&self) -> NeuronVoxelDimensions<usize> {
        self.get_cortical_dimensions().to_usize()
    }
}
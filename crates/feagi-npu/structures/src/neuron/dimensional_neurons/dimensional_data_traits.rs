

// NOTE: Data should be flat as it is optimal in certain common operations

use feagi_structures::base_quantizable::{QuantizableUIntType};
use feagi_structures::define_ref_access_trait_methods;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndexQuantization;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, LeakCoefficient, NPUNeuronIndexType, NPUDataQuantization, NeuronExcitability, FireThresholdLimit};

// NOTE: We do NOT want to per neuron type quantize values like membrane potential due to exponential variation that can occur between genomes as a result!
/// Groups quantization for Dimensional Neuron types. Focuses only on indexing which can be compressed
/// in some cases.
pub trait NPUDimensionalNeuronQuantization {
    type NeuronIndexQuant: QuantizableUIntType + NPUNeuronIndexType;
    type NeuronCountQuant: QuantizableUIntType;
    type CorticalIndexQuant: CorticalAreaIndexQuantization;
    type CorticalCountQuant: QuantizableUIntType;
    type CoordQuant: QuantizableUIntType;
}

/// Defines the base data (both cortical settings and neuron data) shared by all dimensional cortical areas
pub(crate) trait DimensionalNeuronDataArrayTrait<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization, DimensionalCorticalConfigurationQuant: DimensionalCorticalConfigurationTrait<Q, DNQ>>
{
    define_ref_access_trait_methods!(dimensional_cortical_configuration, DimensionalCorticalConfigurationQuant);
    define_ref_access_trait_methods!(neuron_global_burst_index_of_last_firing, [BurstGlobalIndex<Q::GlobalBurstIndexQuant>]);
    define_ref_access_trait_methods!(neuron_membrane_potential, [NeuronMembranePotential<Q::ValueQuant>]);
    define_ref_access_trait_methods!(neuron_fire_threshold, [FireThreshold<Q::ValueQuant>]);
    define_ref_access_trait_methods!(neuron_leak_coefficient, [LeakCoefficient<Q::PercentageQuant>]);
    define_ref_access_trait_methods!(neuron_flags, [NeuronFlag]);
    define_ref_access_trait_methods!(neuron_refractory_countdown, [BurstDelta<Q::BurstDeltaQuant>]);
    define_ref_access_trait_methods!(neuron_consecutive_fire_count, [BurstDelta<Q::BurstDeltaQuant>]);

}


pub(crate) trait DimensionalNeuronDataVectorTrait<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization, DimensionalCorticalConfigurationQuant: DimensionalCorticalConfigurationTrait<Q, DNQ>>:
DimensionalNeuronDataArrayTrait<Q, DNQ, DimensionalCorticalConfigurationQuant>
{
    fn resize_cortical_area(&mut self, dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>, density: NeuronCount<NumberNeuronsPerVoxel>);
}

/// Defines the base cortical settings shared by all dimensional cortical areas
pub(crate) trait DimensionalCorticalConfigurationTrait<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    define_ref_access_trait_methods!(cortical_flag, DimensionalNeuronCorticalFlag);
    define_ref_access_trait_methods!(cortical_dimensions, NeuronVoxelDimensions<DNQ::CoordQuant>);
    define_ref_access_trait_methods!(number_neurons_per_voxel, NeuronCount<NumberNeuronsPerVoxel>);
    define_ref_access_trait_methods!(number_neurons_invalid_from_degeneration, NeuronCount<DNQ::NeuronCountQuant>);
    define_ref_access_trait_methods!(excitability, NeuronExcitability<Q::PercentageQuant>);
    define_ref_access_trait_methods!(refractory_period_limit, BurstDelta<Q::BurstDeltaQuant>);
    define_ref_access_trait_methods!(fire_threshold_limit, FireThresholdLimit<Q::ValueQuant>);
    define_ref_access_trait_methods!(consecutive_fire_limit, BurstDelta<Q::BurstDeltaQuant>);
}


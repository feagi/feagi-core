

// NOTE: Data should be flat as it is optimal in certain common operations

use feagi_structures::base_quantizable::{QuantizableUIntType};
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
pub(crate) trait DimensionalNeuronDataArrayTrait<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization, DimensionalCorticalConfigurationQuant: DimensionalCorticalConfigurationTrait<DNQ>>
{
    fn get_dimensional_cortical_configuration(&self) -> &DimensionalCorticalConfigurationQuant;
    fn get_dimensional_cortical_configuration_mut(&mut self) -> &mut DimensionalCorticalConfigurationQuant;
    fn get_neuron_global_burst_index_of_last_firing(&self) -> &[BurstGlobalIndex<Q::GlobalBurstIndexQuant>];
    fn get_neuron_global_burst_index_of_last_firing_mut(&mut self) -> &mut [BurstGlobalIndex<Q::GlobalBurstIndexQuant>];
    fn get_neuron_membrane_potential(&self) -> &[NeuronMembranePotential<Q::ValueQuant>];
    fn get_neuron_membrane_potential_mut(&mut self) -> &mut [NeuronMembranePotential<Q::ValueQuant>];

    fn get_neuron_fire_threshold(&self) -> &[FireThreshold<Q::ValueQuant>];
    fn get_neuron_fire_threshold_mut(&mut self) -> &mut [FireThreshold<Q::ValueQuant>];
    fn get_neuron_leak_coefficient(&self) -> &[LeakCoefficient<Q::PercentageQuant>];
    fn get_neuron_leak_coefficient_mut(&mut self) -> &mut [LeakCoefficient<Q::PercentageQuant>];
    fn get_neuron_flags(&self) -> &[NeuronFlag];
    fn get_neuron_flags_mut(&mut self) -> &mut [NeuronFlag];
    fn get_neuron_refractory_countdown(&self) -> &[BurstDelta<Q::BurstDeltaQuant>];
    fn get_neuron_refractory_countdown_mut(&mut self) -> &mut [BurstDelta<Q::BurstDeltaQuant>];
    fn get_neuron_consecutive_fire_count(&self) -> &[BurstDelta<Q::BurstDeltaQuant>];
    fn get_neuron_consecutive_fire_count_mut(&mut self) -> &mut [BurstDelta<Q::BurstDeltaQuant>];

}


pub(crate) trait DimensionalNeuronDataVectorTrait<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization, DimensionalCorticalConfigurationQuant: DimensionalCorticalConfigurationTrait<DNQ>>:
DimensionalNeuronDataArrayTrait<Q, DNQ, DimensionalCorticalConfigurationQuant>
{
    fn resize_cortical_area(&mut self, dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>, density: NeuronCount<NumberNeuronsPerVoxel>);
}

/// Defines the base cortical settings shared by all dimensional cortical areas
pub(crate) trait DimensionalCorticalConfigurationTrait<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    fn get_cortical_flag(&self) -> &DimensionalNeuronCorticalFlag;
    fn get_cortical_flag_mut(&mut self) -> &mut DimensionalNeuronCorticalFlag;
    fn get_cortical_dimensions(&self) -> &NeuronVoxelDimensions<DNQ::CoordQuant>;
    fn get_cortical_dimensions_mut(&mut self) -> &mut NeuronVoxelDimensions<DNQ::CoordQuant>;
    fn get_number_neurons_per_voxel(&self) -> &NeuronCount<NumberNeuronsPerVoxel>;
    fn get_number_neurons_per_voxel_mut(&self) -> &NeuronCount<NumberNeuronsPerVoxel>;
    fn get_number_neurons_invalid_from_degeneration(&self) -> &NeuronCount<DNQ::NeuronCountQuant>;
    fn get_number_neurons_invalid_from_degeneration_mut(&mut self) -> &mut NeuronCount<DNQ::NeuronCountQuant>;
    fn get_excitability(&self) -> &NeuronExcitability<Q::PercentageQuant>;
    fn get_excitability_mut(&mut self) -> &mut NeuronExcitability<Q::PercentageQuant>;
    fn get_refractory_period_limit(&self) -> &BurstDelta<Q::BurstDeltaQuant>;
    fn get_refractory_period_limit_mut(&mut self) -> &mut BurstDelta<Q::BurstDeltaQuant>;
    fn get_fire_threshold_limit(&self) -> &FireThresholdLimit<Q::ValueQuant>;
    fn get_fire_threshold_limit_mut(&mut self) -> &mut FireThresholdLimit<Q::ValueQuant>;
    fn get_consecutive_fire_limit(&self) -> &BurstDelta<Q::BurstDeltaQuant>;
    fn get_consecutive_fire_limit_mut(&mut self) -> &mut BurstDelta<Q::BurstDeltaQuant>;
}


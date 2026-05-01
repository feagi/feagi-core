use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_area_generator_traits::DimensionalCorticalAreaGeneratorTrait;
use crate::neuron::neuron_models::dimensional_models::feagi_standard::FeagiStandardCorticalAreaGenerator;
use crate::neuron::neuron_models::dimensional_models::feagi_standard::ram::structs_ram::FeagiStandardNeuronDataRam;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUDimensionalNeuronQuantization, NPUGlobalQuantization, NPUNeuronMembranePotential, NeuronExcitability};

/// Other generators can be defined, including outside this crate, but this is an example one


//region Cortical Area Generators

//region Uniform Cortical Area Generator

/// Generates a Feagi Standard model cortical area in ram using uniform data. This struct is an example
#[cfg(feature = "alloc")]
pub struct FeagiStandardCorticalAreaGeneratorRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {
    cortical_area_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
    cortical_neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>,
    cortical_is_mp_charge_accumulation_enabled: bool,
    cortical_is_mp_driven_psp_enabled: bool,
    cortical_excitability: NeuronExcitability<DNQ::PercentageQuant>,
    cortical_refractory_period_limit: BurstDelta<DNQ::BurstDeltaQuant>,
    cortical_fire_threshold_limit: FireThresholdLimit<DNQ::ValueQuant>,
    cortical_consecutive_fire_limit: BurstDelta<DNQ::BurstDeltaQuant>,
    neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::GlobalBurstIndexQuant>,
    neuron_membrane_potential: NPUNeuronMembranePotential<DNQ::ValueQuant>,
    neuron_fire_threshold: FireThreshold<DNQ::ValueQuant>,
    neuron_leak_coefficient: LeakCoefficient<DNQ::PercentageQuant>,
    neuron_refractory_countdown: BurstDelta<DNQ::BurstDeltaQuant>,
    neuron_consecutive_fire_count: BurstDelta<DNQ::BurstDeltaQuant>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalCorticalAreaGeneratorTrait<Q, DNQ> for FeagiStandardCorticalAreaGeneratorRam<Q, DNQ> {
    type DimensionNeuronModelType = FeagiStandardNeuronDataRam<Q, DNQ>;

    fn number_of_neurons(&self) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
        self.cortical_area_dimensions.get_number_neurons(&self.cortical_neurons_per_voxel)
    }

    fn generate_new_cortical_area_data_ram(&self) -> Self::DimensionNeuronModelType {
        todo!()
    }

    fn overwrite_dead_cortical_area_data_ram(&self, dead_area_overwriting: &mut Self::DimensionNeuronModelType) -> Result<(), FeagiNPUNeuronError> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> FeagiStandardCorticalAreaGenerator<Q, DNQ> for FeagiStandardCorticalAreaGeneratorRam<Q, DNQ> {

}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> FeagiStandardCorticalAreaGeneratorRam<Q, DNQ> {
    pub fn new() -> Result<Self, FeagiNPUNeuronError> {
        todo!() // make sure to validate variables for sanity
    }
}

//endregion

//endregion
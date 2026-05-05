use core::marker::PhantomData;
use feagi_structures::base_feagi_types::::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::CorticalAreaModelType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_configuration_traits::DimensionalCorticalConfigurationTrait;
use crate::neuron::neuron_models::dimensional_models::dimensional_neuron_data_traits::{DimensionalNeuronModelDataResizableTrait, DimensionalNeuronModelDataSharedTrait};
use crate::neuron::neuron_models::dimensional_models::feagi_standard::feagi_standard_traits::{FeagiStandardNeuronModelDataResizableTrait, FeagiStandardNeuronModelDataSharedTrait};
use crate::neuron::neuron_models::dimensional_models::feagi_standard::{FeagiStandardCorticalAreaGenerator, FeagiStandardCorticalConfigurationTrait};
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUDimensionalNeuronQuantization, NPUGlobalQuantization, NeuronExcitability};

//region Neuron Data

pub(crate) struct FeagiStandardNeuronDataRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {
    cortical_configuration: FeagiStandardCorticalConfigurationRam<Q, DNQ>,
    neuron_global_burst_index_of_last_firing: Vec<BurstGlobalIndex<Q::GlobalBurstIndexQuant>>,
    neuron_membrane_potential: Vec<NeuronMembranePotential<DNQ::ValueQuant>>,
    neuron_fire_threshold: Vec<FireThreshold<DNQ::ValueQuant>>,
    neuron_leak_coefficient: Vec<LeakCoefficient<DNQ::PercentageQuant>>,
    neuron_flags: Vec<NeuronFlag>,
    neuron_refractory_countdown: Vec<BurstDelta<DNQ::BurstDeltaQuant>>,
    neuron_consecutive_fire_countdown: Vec<BurstDelta<DNQ::BurstDeltaQuant>>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
FeagiStandardNeuronModelDataResizableTrait<Q, DNQ> for FeagiStandardNeuronDataRam<Q, DNQ> {

}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
FeagiStandardNeuronModelDataSharedTrait<Q, DNQ> for FeagiStandardNeuronDataRam<Q, DNQ> {
    fn get_feagi_standard_cortical_configuration_impl(&self) -> &impl FeagiStandardCorticalConfigurationTrait<Q, DNQ> {
        &self.cortical_configuration
    }

    fn get_feagi_standard_cortical_configuration_impl_mut(&mut self) -> &mut impl FeagiStandardCorticalConfigurationTrait<Q, DNQ> {
        &mut self.cortical_configuration
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
DimensionalNeuronModelDataResizableTrait<Q, DNQ> for FeagiStandardNeuronDataRam<Q, DNQ> {

    // TODO maybe an unsafe variant that rapidly scales the size of the cortical area?
    fn resize_neuron_data_vectors_for_new_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>, neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>) {
        if new_dimensions == self.cortical_configuration.cortical_dimensions &&
            neurons_per_voxel == self.cortical_configuration.number_neurons_per_voxel {
            // No size to change?
            return
        }

        let new_neuron_count = new_dimensions.get_number_neurons(&neurons_per_voxel);

        if new_neuron_count > self.get_total_number_neurons() {
            // We need to increase allocation
            let extend_by = (new_neuron_count - self.get_total_number_neurons()).to_usize();

            // TODO if large amount maybe use rayon?

            self.neuron_global_burst_index_of_last_firing.extend(core::iter::repeat_n(BurstGlobalIndex::ZERO, extend_by));
            self.neuron_membrane_potential.extend(core::iter::repeat_n(NeuronMembranePotential::ZERO, extend_by));
            self.neuron_fire_threshold.extend(core::iter::repeat_n(FireThreshold::ZERO, extend_by));
            self.neuron_leak_coefficient.extend(core::iter::repeat_n(LeakCoefficient::ZERO_PERCENT, extend_by));
            self.neuron_flags.extend(core::iter::repeat_n(NeuronFlag::ALL_ZEROS, extend_by));
            self.neuron_refractory_countdown.extend(core::iter::repeat_n(BurstDelta::ZERO, extend_by));
            self.neuron_consecutive_fire_countdown.extend(core::iter::repeat_n(BurstDelta::ZERO, extend_by));

        } else {
            let new_length = new_neuron_count.to_usize();
            self.neuron_global_burst_index_of_last_firing.truncate(new_length);
            self.neuron_membrane_potential.truncate(new_length);
            self.neuron_fire_threshold.truncate(new_length);
            self.neuron_leak_coefficient.truncate(new_length);
            self.neuron_flags.truncate(new_length);
            self.neuron_refractory_countdown.truncate(new_length);
            self.neuron_consecutive_fire_countdown.truncate(new_length);
        }

        self.cortical_configuration.number_neurons_per_voxel = neurons_per_voxel;
        self.cortical_configuration.cortical_dimensions = new_dimensions;
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
DimensionalNeuronModelDataSharedTrait<Q, DNQ> for FeagiStandardNeuronDataRam<Q, DNQ> {

    const CORTICAL_AREA_MODEL_TYPE: CorticalAreaModelType = CorticalAreaModelType::FeagiStandard;

    type DimensionalCorticalConfigurationType = FeagiStandardCorticalConfigurationRam<Q, DNQ>;

    #[inline]
    fn get_cortical_data(&self) -> &Self::DimensionalCorticalConfigurationType {
        &self.cortical_configuration
    }

    #[inline]
    fn get_cortical_data_mut(&mut self) -> &mut Self::DimensionalCorticalConfigurationType {
        &mut self.cortical_configuration
    }

    #[inline]
    fn get_neuron_global_burst_index_of_last_firing(&self) -> &[BurstGlobalIndex<Q::GlobalBurstIndexQuant>] {
        &self.neuron_global_burst_index_of_last_firing
    }

    #[inline]
    fn get_neuron_global_burst_index_of_last_firing_mut(&mut self) -> &mut [BurstGlobalIndex<Q::GlobalBurstIndexQuant>] {
        &mut self.neuron_global_burst_index_of_last_firing
    }

    #[inline]
    fn get_neuron_membrane_potential(&self) -> &[NeuronMembranePotential<DNQ::ValueQuant>] {
        &self.neuron_membrane_potential
    }

    #[inline]
    fn get_neuron_membrane_potential_mut(&mut self) -> &mut [NeuronMembranePotential<DNQ::ValueQuant>] {
        &mut self.neuron_membrane_potential
    }

    #[inline]
    fn get_neuron_fire_threshold(&self) -> &[FireThreshold<DNQ::ValueQuant>] {
        &self.neuron_fire_threshold
    }

    #[inline]
    fn get_neuron_fire_threshold_mut(&mut self) -> &mut [FireThreshold<DNQ::ValueQuant>] {
        &mut self.neuron_fire_threshold
    }

    #[inline]
    fn get_neuron_leak_coefficient(&self) -> &[LeakCoefficient<DNQ::PercentageQuant>] {
        &self.neuron_leak_coefficient
    }

    #[inline]
    fn get_neuron_leak_coefficient_mut(&mut self) -> &mut [LeakCoefficient<DNQ::PercentageQuant>] {
        &mut self.neuron_leak_coefficient
    }

    #[inline]
    fn get_neuron_flags(&self) -> &[NeuronFlag] {
        &self.neuron_flags
    }

    #[inline]
    fn get_neuron_flags_mut(&mut self) -> &mut [NeuronFlag] {
        &mut self.neuron_flags
    }

    #[inline]
    fn get_neuron_refractory_countdown(&self) -> &[BurstDelta<DNQ::BurstDeltaQuant>] {
        &self.neuron_refractory_countdown
    }

    #[inline]
    fn get_neuron_refractory_countdown_mut(&mut self) -> &mut [BurstDelta<DNQ::BurstDeltaQuant>] {
        &mut self.neuron_refractory_countdown
    }

    #[inline]
    fn get_neuron_consecutive_fire_countdown(&self) -> &[BurstDelta<DNQ::BurstDeltaQuant>] {
        &self.neuron_consecutive_fire_countdown
    }

    #[inline]
    fn get_neuron_consecutive_fire_countdown_mut(&mut self) -> &mut [BurstDelta<DNQ::BurstDeltaQuant>] {
        &mut self.neuron_consecutive_fire_countdown
    }

}



//endregion


//region Cortical Configuration
pub(crate) struct FeagiStandardCorticalConfigurationRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {
    cortical_flags: DimensionalNeuronCorticalFlag,
    cortical_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
    number_neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>,
    number_neurons_invalid_from_degeneration: NeuronCount<Q::NeuronIndexCountQuant>,
    excitability: NeuronExcitability<DNQ::PercentageQuant>,
    refractory_period_limit: BurstDelta<DNQ::BurstDeltaQuant>,
    fire_threshold_limit: FireThresholdLimit<DNQ::ValueQuant>,
    consecutive_fire_limit: BurstDelta<DNQ::BurstDeltaQuant>,
    _phantom_data: PhantomData<Q>
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> FeagiStandardCorticalConfigurationTrait<Q, DNQ> for FeagiStandardCorticalConfigurationRam<Q, DNQ> {
    
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalCorticalConfigurationTrait<Q, DNQ> for FeagiStandardCorticalConfigurationRam<Q, DNQ> {
    
    #[inline]
    fn get_cortical_flag(&self) -> &DimensionalNeuronCorticalFlag {
        &self.cortical_flags
    }

    #[inline]
    fn get_cortical_flag_mut(&mut self) -> &mut DimensionalNeuronCorticalFlag {
        &mut self.cortical_flags
    }

    #[inline]
    fn get_cortical_dimensions(&self) -> &NeuronVoxelDimensions<DNQ::CoordQuant> {
        &self.cortical_dimensions
    }

    #[inline]
    fn get_cortical_dimensions_mut(&mut self) -> &mut NeuronVoxelDimensions<DNQ::CoordQuant> {
        &mut self.cortical_dimensions
    }

    #[inline]
    fn get_number_neurons_per_voxel(&self) -> &NeuronCount<NumberNeuronsPerVoxel> {
        &self.number_neurons_per_voxel
    }

    #[inline]
    fn get_number_neurons_per_voxel_mut(&mut self) -> &mut NeuronCount<NumberNeuronsPerVoxel> {
        &mut self.number_neurons_per_voxel
    }

    #[inline]
    fn get_number_neurons_invalid_from_degeneration(&self) -> &NeuronCount<Q::NeuronIndexCountQuant> {
        &self.number_neurons_invalid_from_degeneration
    }

    #[inline]
    fn get_number_neurons_invalid_from_degeneration_mut(&mut self) -> &mut NeuronCount<Q::NeuronIndexCountQuant> {
        &mut self.number_neurons_invalid_from_degeneration
    }

    #[inline]
    fn get_excitability(&self) -> &NeuronExcitability<DNQ::PercentageQuant> {
        &self.excitability
    }

    #[inline]
    fn get_excitability_mut(&mut self) -> &mut NeuronExcitability<DNQ::PercentageQuant> {
        &mut self.excitability
    }

    #[inline]
    fn get_refractory_period_limit(&self) -> &BurstDelta<DNQ::BurstDeltaQuant> {
        &self.refractory_period_limit
    }

    #[inline]
    fn get_refractory_period_limit_mut(&mut self) -> &mut BurstDelta<DNQ::BurstDeltaQuant> {
        &mut self.refractory_period_limit
    }

    #[inline]
    fn get_fire_threshold_limit(&self) -> &FireThresholdLimit<DNQ::ValueQuant> {
        &self.fire_threshold_limit
    }

    #[inline]
    fn get_fire_threshold_limit_mut(&mut self) -> &mut FireThresholdLimit<DNQ::ValueQuant> {
        &mut self.fire_threshold_limit
    }

    #[inline]
    fn get_consecutive_fire_limit(&self) -> &BurstDelta<DNQ::BurstDeltaQuant> {
        &self.consecutive_fire_limit
    }

    #[inline]
    fn get_consecutive_fire_limit_mut(&mut self) -> &mut BurstDelta<DNQ::BurstDeltaQuant> {
        &mut self.consecutive_fire_limit
    }
}

//endregion



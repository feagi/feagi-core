use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::neuron::dimensional_neurons::dimensional_data_traits::{DimensionalCorticalConfigurationTrait, DimensionalNeuronDataArrayTrait, DimensionalNeuronDataVectorTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUGlobalQuantization, NPUDimensionalNeuronQuantization, NeuronExcitability};



//region Total Neuron Configuration

// TODO Array Fixed Implementation?

pub(crate) struct InterNeuronRamData<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {
    cortical_configuration:  InterNeuronRamCorticalConfiguration<Q, DNQ>,
    neuron_global_burst_index_of_last_firing: Vec<BurstGlobalIndex<Q::GlobalBurstIndexQuant>>,
    neuron_membrane_potential: Vec<NeuronMembranePotential<Q::ValueQuant>>,
    neuron_fire_threshold: Vec<FireThreshold<Q::ValueQuant>>,
    neuron_leak_coefficient: Vec<LeakCoefficient<Q::PercentageQuant>>,
    neuron_flags: Vec<NeuronFlag>,
    neuron_refractory_countdown: Vec<BurstDelta<Q::BurstDeltaQuant>>,
    neuron_consecutive_fire_countdown: Vec<BurstDelta<Q::BurstDeltaQuant>>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronDataArrayTrait<Q, DNQ> for InterNeuronRamData<Q, DNQ> {
    type DimensionalCorticalConfigurationQuant = InterNeuronRamCorticalConfiguration<Q, DNQ>;

    #[inline]
    fn get_dimensional_cortical_configuration(&self) -> &Self::DimensionalCorticalConfigurationQuant {
        &self.cortical_configuration
    }

    #[inline]
    fn get_dimensional_cortical_configuration_mut(&mut self) -> &mut Self::DimensionalCorticalConfigurationQuant {
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
    fn get_neuron_membrane_potential(&self) -> &[NeuronMembranePotential<Q::ValueQuant>] {
        &self.neuron_membrane_potential
    }

    #[inline]
    fn get_neuron_membrane_potential_mut(&mut self) -> &mut [NeuronMembranePotential<Q::ValueQuant>] {
        &mut self.neuron_membrane_potential
    }

    #[inline]
    fn get_neuron_fire_threshold(&self) -> &[FireThreshold<Q::ValueQuant>] {
        &self.neuron_fire_threshold
    }

    #[inline]
    fn get_neuron_fire_threshold_mut(&mut self) -> &mut [FireThreshold<Q::ValueQuant>] {
        &mut self.neuron_fire_threshold
    }

    #[inline]
    fn get_neuron_leak_coefficient(&self) -> &[LeakCoefficient<Q::PercentageQuant>] {
        &self.neuron_leak_coefficient
    }

    #[inline]
    fn get_neuron_leak_coefficient_mut(&mut self) -> &mut [LeakCoefficient<Q::PercentageQuant>] {
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
    fn get_neuron_refractory_countdown(&self) -> &[BurstDelta<Q::BurstDeltaQuant>] {
        &self.neuron_refractory_countdown
    }

    #[inline]
    fn get_neuron_refractory_countdown_mut(&mut self) -> &mut [BurstDelta<Q::BurstDeltaQuant>] {
        &mut self.neuron_refractory_countdown
    }

    #[inline]
    fn get_neuron_consecutive_fire_countdown(&self) -> &[BurstDelta<Q::BurstDeltaQuant>] {
        &self.neuron_consecutive_fire_countdown
    }

    #[inline]
    fn get_neuron_consecutive_fire_countdown_mut(&mut self) -> &mut [BurstDelta<Q::BurstDeltaQuant>] {
        &mut self.neuron_consecutive_fire_countdown
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronDataVectorTrait<Q, DNQ> for InterNeuronRamData<Q, DNQ> {
    fn resize_cortical_area(&mut self, dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>, density: NeuronCount<NumberNeuronsPerVoxel>) {
        todo!()
    }
}


impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> InterNeuronRamData<Q, DNQ> {

    // NOTE: No Default inits

    pub fn new_uniform(
        cortical_configuration: InterNeuronRamCorticalConfiguration<Q, DNQ>,
        global_burst_index_of_last_firing: BurstGlobalIndex<Q::GlobalBurstIndexQuant>,
        membrane_potential: NeuronMembranePotential<Q::ValueQuant>,
        fire_threshold: FireThreshold<Q::ValueQuant>,
        leak_coefficient: LeakCoefficient<Q::PercentageQuant>,
        flags: NeuronFlag,
        refractory_countdown: BurstDelta<Q::BurstDeltaQuant>,
        consecutive_fire_count: BurstDelta<Q::BurstDeltaQuant>,
    ) -> Self {
        let number_neurons: usize = cortical_configuration.cortical_dimensions.get_number_neurons(cortical_configuration.number_neurons_per_voxel).to_usize();

        Self {
            cortical_configuration,
            neuron_global_burst_index_of_last_firing: vec![global_burst_index_of_last_firing; number_neurons],
            neuron_membrane_potential: vec![membrane_potential; number_neurons],
            neuron_fire_threshold: vec![fire_threshold; number_neurons],
            neuron_leak_coefficient: vec![leak_coefficient; number_neurons],
            neuron_flags: vec![flags; number_neurons],
            neuron_refractory_countdown: vec![refractory_countdown; number_neurons],
            neuron_consecutive_fire_countdown: vec![consecutive_fire_count; number_neurons],
        }
    }
}

//endregion

// TODO Array Fixed Implementation?

//region Cortical Configuration
pub(crate) struct InterNeuronRamCorticalConfiguration<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {
    cortical_flags: DimensionalNeuronCorticalFlag,
    cortical_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
    number_neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>,
    number_neurons_invalid_from_degeneration: NeuronCount<DNQ::NeuronCountQuant>,
    excitability: NeuronExcitability<Q::PercentageQuant>,
    refractory_period_limit: BurstDelta<Q::BurstDeltaQuant>,
    fire_threshold_limit: FireThresholdLimit<Q::ValueQuant>,
    consecutive_fire_limit: BurstDelta<Q::BurstDeltaQuant>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalCorticalConfigurationTrait<Q, DNQ> for InterNeuronRamCorticalConfiguration<Q, DNQ> {

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
    fn get_number_neurons_invalid_from_degeneration(&self) -> &NeuronCount<DNQ::NeuronCountQuant> {
        &self.number_neurons_invalid_from_degeneration
    }

    #[inline]
    fn get_number_neurons_invalid_from_degeneration_mut(&mut self) -> &mut NeuronCount<DNQ::NeuronCountQuant> {
        &mut self.number_neurons_invalid_from_degeneration
    }

    #[inline]
    fn get_excitability(&self) -> &NeuronExcitability<Q::PercentageQuant> {
        &self.excitability
    }

    #[inline]
    fn get_excitability_mut(&mut self) -> &mut NeuronExcitability<Q::PercentageQuant> {
        &mut self.excitability
    }

    #[inline]
    fn get_refractory_period_limit(&self) -> &BurstDelta<Q::BurstDeltaQuant> {
        &self.refractory_period_limit
    }

    #[inline]
    fn get_refractory_period_limit_mut(&mut self) -> &mut BurstDelta<Q::BurstDeltaQuant> {
        &mut self.refractory_period_limit
    }

    #[inline]
    fn get_fire_threshold_limit(&self) -> &FireThresholdLimit<Q::ValueQuant> {
        &self.fire_threshold_limit
    }

    #[inline]
    fn get_fire_threshold_limit_mut(&mut self) -> &mut FireThresholdLimit<Q::ValueQuant> {
        &mut self.fire_threshold_limit
    }

    #[inline]
    fn get_consecutive_fire_limit(&self) -> &BurstDelta<Q::BurstDeltaQuant> {
        &self.consecutive_fire_limit
    }

    #[inline]
    fn get_consecutive_fire_limit_mut(&mut self) -> &mut BurstDelta<Q::BurstDeltaQuant> {
        &mut self.consecutive_fire_limit
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> InterNeuronRamCorticalConfiguration<Q, DNQ> {
    #[inline]
    pub fn new(
        cortical_flags: DimensionalNeuronCorticalFlag,
        cortical_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
        number_neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>,
        number_neurons_invalid_from_degeneration: NeuronCount<DNQ::NeuronCountQuant>,
        excitability: NeuronExcitability<Q::PercentageQuant>,
        refractory_period_limit: BurstDelta<Q::BurstDeltaQuant>,
        fire_threshold_limit: FireThresholdLimit<Q::ValueQuant>,
        consecutive_fire_limit: BurstDelta<Q::BurstDeltaQuant>,
    ) -> Result<Self, FeagiNPUNeuronError> {
        if number_neurons_per_voxel == NeuronCount::ZERO {
            return Err(
                FeagiNPUNeuronError::NeuronDensityCannotBeZero {
                    context: "Cannot init a new inter-neuron cortical configuration with a density of zero neurons per voxel!",
                }
            )
        }

        Ok(Self {
            cortical_flags,
            cortical_dimensions,
            number_neurons_per_voxel,
            number_neurons_invalid_from_degeneration,
            excitability,
            refractory_period_limit,
            fire_threshold_limit,
            consecutive_fire_limit,
        })
    }
}

//endregion
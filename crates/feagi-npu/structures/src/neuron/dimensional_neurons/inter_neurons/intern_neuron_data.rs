use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::neuron::dimensional_neurons::dimensional_data_traits::{DimensionalCorticalConfigurationTrait, NPUDimensionalNeuronQuantization};
use crate::neuron::flags::DimensionalNeuronCorticalFlag;
use crate::quantizables::{BurstDelta, ConsecutiveFireLimit, FireThresholdLimit, NPUDataQuantization, NeuronExcitability};

pub(crate) struct InterNeuronRamCorticalConfiguration<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization> {
    cortical_flags: DimensionalNeuronCorticalFlag,
    cortical_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
    number_neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>,
    number_neurons_invalid_from_degeneration: NeuronCount<DNQ::NeuronCountQuant>,
    excitability: NeuronExcitability<Q::PercentageQuant>,
    refractory_period_limit: BurstDelta<Q::BurstDeltaQuant>,
    fire_threshold_limit: FireThresholdLimit<Q::ValueQuant>,
    consecutive_fire_limit: ConsecutiveFireLimit<Q::ValueQuant>,
}

impl<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalCorticalConfigurationTrait<Q, DNQ> for InterNeuronRamCorticalConfiguration<Q, DNQ> {
    fn get_cortical_flag(&self) -> &DimensionalNeuronCorticalFlag {
        todo!()
    }

    fn get_cortical_flag_mut(&mut self) -> &mut DimensionalNeuronCorticalFlag {
        todo!()
    }

    fn get_cortical_dimensions(&self) -> &NeuronVoxelDimensions<DNQ::CoordQuant> {
        todo!()
    }

    fn get_cortical_dimensions_mut(&mut self) -> &mut NeuronVoxelDimensions<DNQ::CoordQuant> {
        todo!()
    }

    fn get_number_neurons_per_voxel(&self) -> &NeuronCount<NumberNeuronsPerVoxel> {
        todo!()
    }

    fn get_number_neurons_per_voxel_mut(&self) -> &NeuronCount<NumberNeuronsPerVoxel> {
        todo!()
    }

    fn get_number_neurons_invalid_from_degeneration(&self) -> &NeuronCount<DNQ::NeuronCountQuant> {
        todo!()
    }

    fn get_number_neurons_invalid_from_degeneration_mut(&mut self) -> &mut NeuronCount<DNQ::NeuronCountQuant> {
        todo!()
    }

    fn get_excitability(&self) -> &NeuronExcitability<Q::PercentageQuant> {
        todo!()
    }

    fn get_excitability_mut(&mut self) -> &mut NeuronExcitability<Q::PercentageQuant> {
        todo!()
    }

    fn get_refractory_period_limit(&self) -> &BurstDelta<Q::BurstDeltaQuant> {
        todo!()
    }

    fn get_refractory_period_limit_mut(&mut self) -> &mut BurstDelta<Q::BurstDeltaQuant> {
        todo!()
    }

    fn get_fire_threshold_limit(&self) -> &FireThresholdLimit<Q::ValueQuant> {
        todo!()
    }

    fn get_fire_threshold_limit_mut(&mut self) -> &mut FireThresholdLimit<Q::ValueQuant> {
        todo!()
    }

    fn get_consecutive_fire_limit(&self) -> &BurstDelta<Q::BurstDeltaQuant> {
        todo!()
    }

    fn get_consecutive_fire_limit_mut(&mut self) -> &mut BurstDelta<Q::BurstDeltaQuant> {
        todo!()
    }
}

impl<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization> Default for InterNeuronRamCorticalConfiguration<Q, DNQ> {
    fn default() -> Self {
        todo!()
    }
}

impl<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization> InterNeuronRamCorticalConfiguration<Q, DNQ> {
    pub fn new() -> Self {
        todo!()
    }
}



// TODO Array Fixed Implementation?

pub(crate) struct InterNeuronRamData<Q: NPUDataQuantization, DNQ: NPUDimensionalNeuronQuantization, DimensionalCorticalConfigurationQuant: DimensionalCorticalConfigurationTrait<DNQ>> {
    cortical_configuration:  InterNeuronRamCorticalConfiguration<Q, DNQ>,

}

impl
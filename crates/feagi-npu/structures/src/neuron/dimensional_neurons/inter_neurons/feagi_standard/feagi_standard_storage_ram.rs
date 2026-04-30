use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_storage_traits::{BaseNeuronCommonStorageTrait, BaseNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::inter_neurons::inter_neuron_traits::{InterNeuronCommonStorageTrait, InterNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUDimensionalNeuronQuantization, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability};
use crate::quantizables::NPUGlobalQuantization;

pub struct FeagiStandardNeuronAllocRAMStorage<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{

}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> BaseNeuronCommonStorageTrait<Q, DNQ> for FeagiStandardNeuronAllocRAMStorage<Q, DNQ> {
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<DNQ::NeuronIndexQuant> {
        todo!()
    }

    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<DNQ::NeuronIndexQuant> {
        todo!()
    }

    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<DNQ::NeuronIndexQuant> {
        todo!()
    }

    fn get_max_possible_cortical_area_index(&self) -> CorticalAreaIndex<Q::CorticalIndexQuant> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> BaseNeuronResizableStorageTrait<Q, DNQ> for FeagiStandardNeuronAllocRAMStorage<Q, DNQ> {
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<DNQ::NeuronIndexQuant>) -> NeuronCount<DNQ::NeuronIndexQuant> {
        todo!()
    }

    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<Range<NPUNeuronIndex<DNQ::NeuronIndexQuant>>, FeagiNPUNeuronError> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronCommonStorageTrait<Q, DNQ> for FeagiStandardNeuronAllocRAMStorage<Q, DNQ> {
    fn get_cortical_data(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&DimensionalNeuronCorticalData<Q>, FeagiNPUNeuronError> {
        todo!()
    }

    fn get_global_burst_index_of_last_firing(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstGlobalIndex<Q::GlobalBurstIndexQuant>], FeagiNPUNeuronError> {
        todo!()
    }

    fn get_neuron_membrane_potential(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[NPUNeuronMembranePotential<DNQ::ValueQuant>], FeagiNPUNeuronError> {
        todo!()
    }

    fn get_fire_threshold(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[FireThreshold<DNQ::ValueQuant>], FeagiNPUNeuronError> {
        todo!()
    }

    fn get_leak_coefficient(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[LeakCoefficient<DNQ::PercentageQuant>], FeagiNPUNeuronError> {
        todo!()
    }

    fn get_neuron_flags(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[NeuronFlag], FeagiNPUNeuronError> {
        todo!()
    }

    fn get_refractory_countdown(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstDelta<DNQ::BurstDeltaQuant>], FeagiNPUNeuronError> {
        todo!()
    }

    fn get_consecutive_fire_count(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstDelta<DNQ::BurstDeltaQuant>], FeagiNPUNeuronError> {
        todo!()
    }

    fn get_neuron_values_of_all_dimensional_neuron_cortical_areas_to_process(&mut self) -> DimensionalNeuronDataRefSliceAllCorticalAreas<'_, Q> {
        todo!()
    }

    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, Q>, FeagiNPUNeuronError> {
        todo!()
    }

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>, executor: &impl NeuronFireThresholdExecutor<DNQ::ValueQuant, DNQ::CoordQuant>) -> Result<(), FeagiNPUNeuronError> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronResizableStorageTrait<Q, DNQ> for FeagiStandardNeuronAllocRAMStorage<Q, DNQ> {
    fn create_cortical_area_with_uniform_neurons(&mut self, cortical_area_dimensions: NeuronVoxelDimensions<DNQ::CoordQuant>,
                                                 neurons_per_voxel: NumberNeuronsPerVoxel,
                                                 neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::GlobalBurstIndexQuant>,
                                                 neuron_membrane_potential: NPUNeuronMembranePotential<DNQ::ValueQuant>,
                                                 neuron_fire_threshold: FireThreshold<DNQ::ValueQuant>,
                                                 neuron_leak_coefficient: LeakCoefficient<DNQ::PercentageQuant>,
                                                 neuron_refractory_countdown: BurstDelta<DNQ::BurstDeltaQuant>,
                                                 neuron_consecutive_fire_count: BurstDelta<DNQ::BurstDeltaQuant>,
                                                 cortical_excitability: NeuronExcitability<DNQ::PercentageQuant>,
                                                 cortical_refractory_period_limit: BurstDelta<DNQ::BurstDeltaQuant>,
                                                 cortical_fire_threshold_limit: FireThresholdLimit<DNQ::ValueQuant>,
                                                 cortical_consecutive_fire_limit: BurstDelta<DNQ::BurstDeltaQuant>,
                                                 cortical_is_mp_charge_accumulation_enabled: bool,
                                                 cortical_is_mp_driven_psp_enabled: bool)
        -> Result<(CorticalAreaIndex<Q::CorticalIndexQuant>), FeagiNPUNeuronError> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> InterNeuronCommonStorageTrait<Q, DNQ> for FeagiStandardNeuronAllocRAMStorage<Q, DNQ>
{

}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> InterNeuronResizableStorageTrait<Q, DNQ> for FeagiStandardNeuronAllocRAMStorage<Q, DNQ>
{

}
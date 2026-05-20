
// NOTE: In alloc contexts, dont be too overspecific with quantization

use std::ops::Range;
use feagi_structures::base_feagi_types::::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxel::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::connectome::{ConnectomeAllocTrait, ConnectomeBaseTrait};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::FeagiNPUStructureError;
use crate::fire_candidate_list::{FireCandidateListRam, FireCandidateListTrait};
use crate::fire_queue::{FireQueueRam, FireQueueTrait};
use crate::neuron::base_dimension_traits::DimensionalAllocStorageTrait;
use crate::neuron::npu_storage::base_storage_traits::BaseNeuronResizableStorageTrait;
use crate::neuron::npu_storage::::core_neurons::CoreNeuronAllocRAMStorage;
use crate::neuron::npu_storage::::dimensional_storage_traits::{DimensionalNeuronResizableStorageTrait, DimensionalNeuronFixedStorageTrait};
use crate::neuron::npu_storage::::inter_neurons::InterNeuronStorageResizableRam;
use crate::neuron::npu_storage::::motor_neurons::MotorNeuronAllocRAMStorage;
use crate::neuron::npu_storage::::sensory_neurons::SensoryNeuronAllocRAMStorage;
use crate::neuron::npu_storage::::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataFromCorticalArea, DimensionalTypedCorticalIndex, DimensionalTypedNeuronIndex};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUGlobalQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability, SynapseBundleIndex, SynapseCount, NPUNeuronMembranePotential};
use crate::synapse::non_plastic_dimensional::NonplasticDimensionalSynapseAllocRAMStorage;
use crate::synapse::non_plastic_dimensional::non_plastic_traits::{NonplasticSynapseAllocStorageTrait, NonplasticSynapseBaseStorageTrait};

pub struct ConnectomeAllocRam<Q: NPUGlobalQuantization>
{
    fire_queue: FireQueueRam<Q::NeuronIndexQuant>,
    fire_candidate_list: FireCandidateListRam<Q::NeuronIndexQuant>,


    // Neurons
    core_neurons: CoreNeuronAllocRAMStorage<Q>,
    sensory_neurons: SensoryNeuronAllocRAMStorage<Q>,
    motor_neurons: MotorNeuronAllocRAMStorage<Q>,
    inter_neurons: InterNeuronStorageResizableRam<Q>,
    
    
    // Synapses
    synapse_nonplastic: NonplasticDimensionalSynapseAllocRAMStorage<Q>
}


impl<Q: NPUGlobalQuantization>
ConnectomeAllocRam<Q>
{

    pub fn new() -> ConnectomeAllocRam<Q> {
        ConnectomeAllocRam {
            fire_queue: FireQueueRam::new(0),
            fire_candidate_list: FireCandidateListRam::new(0),
            core_neurons: CoreNeuronAllocRAMStorage::new(),
            sensory_neurons: SensoryNeuronAllocRAMStorage::new(NeuronCount::ZERO, CorticalAreaIndex::ZERO),
            motor_neurons: MotorNeuronAllocRAMStorage::new(NeuronCount::ZERO, CorticalAreaIndex::ZERO),
            inter_neurons: InterNeuronStorageResizableRam::new(NeuronCount::ZERO, CorticalAreaIndex::ZERO),
            synapse_nonplastic: NonplasticDimensionalSynapseAllocRAMStorage::new(SynapseCount::ZERO),
        }
    }

    //region helpers



    //endregion


}

impl<Q: NPUGlobalQuantization>
ConnectomeAllocTrait<Q> for
ConnectomeAllocRam<Q>
{

    //region Cortical Areas

    //region Dimensional Neuron Cortical Areas

    //region Core Cortical Areas

    // You cannot create or destroy core cortical areas!

    //endregion

    //region Sensor Cortical Areas

    //endregion

    //region Motor Cortical Areas

    //endregion

    //region Interneuron Cortical Areas

    /// Create interneuron (custom) cortical area with default neuron settings spanned across the
    /// entire cortical area. Returns the cortical index of this new area.
    fn create_interneuron_area_with_default_neurons(&mut self,
                                                    cortical_area_dimensions: NeuronVoxelDimensions<Q::CoordQuantQuant>,
                                                    neurons_per_voxel: NumberNeuronsPerVoxel)
                                                    -> Result<(CorticalAreaIndex<Q::CorticalIndexCountQuant>), FeagiNPUStructureError>{
        self.inter_neurons.create_cortical_area_with_default_neurons(
            cortical_area_dimensions,
            neurons_per_voxel
        ).map_err(|err| err.into())
    }

    /// Create interneuron (custom) cortical area with given neuron settings spanned across the
    /// entire cortical area. Returns the cortical index of this new area.
    fn create_interneuron_cortical_area_with_uniform_neurons(&mut self, // TODO change other instances of spanned to uniform
                                                             cortical_area_dimensions: NeuronVoxelDimensions<Q::CoordQuantQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel,
                                                             neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::GlobalBurstIndexQuant>,
                                                             neuron_membrane_potential: NPUNeuronMembranePotential<Q::ValueQuant>,
                                                             neuron_fire_threshold: FireThreshold<Q::ValueQuant>,
                                                             neuron_leak_coefficient: LeakCoefficient<Q::PercentageQuant>,
                                                             neuron_refractory_countdown: BurstDelta<Q::BurstDeltaQuant>,
                                                             neuron_consecutive_fire_count: BurstDelta<Q::BurstDeltaQuant>,
                                                             cortical_excitability: NeuronExcitability<Q::PercentageQuant>,
                                                             cortical_refractory_period_limit: BurstDelta<Q::BurstDeltaQuant>,
                                                             cortical_fire_threshold_limit: FireThresholdLimit<Q::ValueQuant>,
                                                             cortical_consecutive_fire_limit: BurstDelta<Q::BurstDeltaQuant>,
                                                             cortical_is_mp_charge_accumulation_enabled: bool,
                                                             cortical_is_mp_driven_psp_enabled: bool)
                                                             -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUStructureError> {
       let cortical_index = self.inter_neurons.create_cortical_area_with_uniform_neurons(
            cortical_area_dimensions,
            neurons_per_voxel,
            neuron_global_burst_index_of_last_firing,
            neuron_membrane_potential,
            neuron_fire_threshold,
            neuron_leak_coefficient,
            neuron_refractory_countdown,
            neuron_consecutive_fire_count,
            cortical_excitability,
            cortical_refractory_period_limit,
            cortical_fire_threshold_limit,
            cortical_consecutive_fire_limit,
            cortical_is_mp_charge_accumulation_enabled,
            cortical_is_mp_driven_psp_enabled)?;
        Ok(cortical_index)
    }

    /// Create interneuron (custom) cortical area with given per neuron values
    fn create_interneuron_cortical_area_with_individualized_neurons(&mut self,
                                                                    cortical_area_dimensions: NeuronVoxelDimensions<Q::CoordQuantQuant>,
                                                                    neurons_per_voxel: NumberNeuronsPerVoxel,
                                                                    neuron_data: DimensionalNeuronDataFromCorticalArea<Q>)
                                                                    -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUStructureError>
    {
        todo!()
    }


    /// Resizes an interneuron neuron cortical area toa  new dimension and or density. Attempts to maintain
    /// cortical area level values, but per neuron values will be reset! First disconnects
    /// all existing synapses, saves the cortical level values, deletes the cortical area,
    /// recreates the cortical area with
    /// the new dimensions and cortical level settings and then reestablishes the synapses
    /// anew with given mappers (should be the same as what was had before)
    fn resize_interneuron_cortical_area<'a>(&mut self,
                                            cortical_area_dimensions: NeuronVoxelDimensions<Q::CoordQuantQuant>,
                                            neurons_per_voxel: NumberNeuronsPerVoxel,
                                            cortical_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                            presynaptic_nonplastic_dimensional_mappings: &Vec<(
                                                CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                DimensionCorticalAreaType, &'a impl NonPlasticCorticalMappingDefinitionExecutor<Q>)>,
                                            postsynaptic_nonplastic_dimensional_mappings: &Vec<(
                                                CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                DimensionCorticalAreaType, &'a impl NonPlasticCorticalMappingDefinitionExecutor<Q>)>, )

                                            -> Result<(), FeagiNPUStructureError>
    {
        todo!()
        //TODO Right now, the resize function is broken, fix it before implementing here!
    }


    /// First deletes any synaptic connections to / from this area, then deletes the interneuron
    /// cortical area
    fn delete_interneuron_cortical_area(&mut self,
                                        cortical_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>)
                                        -> Result<(), FeagiNPUStructureError> {
        _ = self.inter_neurons.delete_cortical_area(cortical_index)?;
        // TODO delete mappings!
        Ok(())
    }

    //endregion


    //endregion

    //endregion

    //region Synapses

    //region dimensional area to dimensional area

    fn add_nonplastic_connection_from_dimensional_area_to_dimensional_area(&mut self,
                                                                           source_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                                           source_area_dimension_type: DimensionCorticalAreaType,
                                                                           destination_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                                           destination_area_dimension_type: DimensionCorticalAreaType,
                                                                           neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<Q>)
                                                                           -> Result<SynapseBundleIndex<Q::SynapseBundleIndexQuant>, FeagiNPUStructureError> {


        let (source_cortical_data, source_neuron_flags) = match &source_area_dimension_type {
            DimensionCorticalAreaType::Sensor => {
                let source_cortical_data= self.sensory_neurons.get_cortical_data(source_area_index)?;
                let source_neuron_flags = self.sensory_neurons.get_neuron_flags(source_area_index)?;
                (source_cortical_data, source_neuron_flags)
            }
            DimensionCorticalAreaType::Motor => {
                let source_cortical_data= self.motor_neurons.get_cortical_data(source_area_index)?;
                let source_neuron_flags = self.motor_neurons.get_neuron_flags(source_area_index)?;
                (source_cortical_data, source_neuron_flags)
            }
            DimensionCorticalAreaType::Core => {
                let source_cortical_data= self.core_neurons.get_cortical_data(source_area_index)?;
                let source_neuron_flags = self.core_neurons.get_neuron_flags(source_area_index)?;
                (source_cortical_data, source_neuron_flags)
            }
            DimensionCorticalAreaType::Custom => {
                let source_cortical_data= self.sensory_neurons.get_cortical_data(source_area_index)?;
                let source_neuron_flags = self.sensory_neurons.get_neuron_flags(source_area_index)?;
                (source_cortical_data, source_neuron_flags)
            }
        };

        let (destination_cortical_data, destination_neuron_flags) = match &destination_area_dimension_type {
            DimensionCorticalAreaType::Sensor => {
                let destination_cortical_data= self.sensory_neurons.get_cortical_data(destination_area_index)?;
                let destination_neuron_flags = self.sensory_neurons.get_neuron_flags(destination_area_index)?;
                (destination_cortical_data, destination_neuron_flags)
            }
            DimensionCorticalAreaType::Motor => {
                let destination_cortical_data= self.motor_neurons.get_cortical_data(destination_area_index)?;
                let destination_neuron_flags = self.motor_neurons.get_neuron_flags(destination_area_index)?;
                (destination_cortical_data, destination_neuron_flags)
            }
            DimensionCorticalAreaType::Core => {
                let destination_cortical_data= self.core_neurons.get_cortical_data(destination_area_index)?;
                let destination_neuron_flags = self.core_neurons.get_neuron_flags(destination_area_index)?;
                (destination_cortical_data, destination_neuron_flags)
            }
            DimensionCorticalAreaType::Custom => {
                let destination_cortical_data= self.sensory_neurons.get_cortical_data(destination_area_index)?;
                let destination_neuron_flags = self.sensory_neurons.get_neuron_flags(destination_area_index)?;
                (destination_cortical_data, destination_neuron_flags)
            }
        };

        let source_area_index = DimensionalTypedCorticalIndex {
            index: source_area_index,
            dimensional_type: source_area_dimension_type,
        };
        let destination_area_index = DimensionalTypedCorticalIndex {
            index: destination_area_index,
            dimensional_type: destination_area_dimension_type,
        };


        let synapse_bundle_index = self.synapse_nonplastic.add_synapses_mapping_between_cortical_areas(
            source_area_index,
            source_cortical_data,
            source_neuron_flags,
            destination_area_index,
            destination_cortical_data,
            destination_neuron_flags,
            neuron_mapping_executor).map_err(|err| FeagiNPUStructureError::NeuronError { error: FeagiNPUNeuronError::InternalError { context: "TODO" } })?;

        Ok(synapse_bundle_index)
    }

    fn disconnect_all_synapses_from_dimensional_area_to_dimensional_area(&mut self,
                                                                         source_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                                         destination_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>)
                                                                         -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    //endregion

    //endregion

    //region Housekeeping

    fn free_unused_neuron_capacity(&mut self) {
        todo!()
    }

    fn free_unused_synapse_capacity(&mut self) {
        todo!()
    }

    fn free_unused_cortical_area_capacity(&mut self) {
        todo!()
    }
    //endregion
}

impl<Q: NPUGlobalQuantization>
ConnectomeBaseTrait<Q> for
ConnectomeAllocRam<Q>
{
    fn process_burst(&mut self, current_burst_index: &BurstGlobalIndex<<Q as NPUGlobalQuantization>::GlobalBurstIndexQuant>) -> Result<(), FeagiNPUStructureError> {

        // Reset Fire Candidate List to prep for burst
        self.fire_candidate_list.clear();

        // Iterate through fire queue
        // TODO Rayon

        for firing_neuron_index in self.fire_queue.get_core_neuron_indexes_slice() {
            let (downstream_synapses_iterator, neuron_count) = self.synapse_nonplastic.get_nonplastic_synapse_data_from_source_neuron_index(
                DimensionalTypedNeuronIndex {
                    index: firing_neuron_index.clone(),
                    dimensional_type: DimensionCorticalAreaType::Core
                }
            )?;
            // TODO mp_charge_accumulation ? do we do it at the start of the burst?


            for downstream_synapse in downstream_synapses_iterator {
                let downstream_neuron_index = downstream_synapse.destination_neuron_index;




            }


        }









        // reset fire queue
        self.fire_candidate_list.clear(); // Also sets pwr


        // TODO rayon feature swapper

        // NOTE I dont think its a good idea to have injectors run in here, they should be done before
        // maybe motor / sensor stuff should be injected to areas directly

        // TODO many errors here can only occur if something went very wrong. We should map them appropriately to be clear

        for firing_neuron_index in self.fire_queue.get_core_neuron_indexes_slice() {
            let (downstream_synapses_iterator, neuron_count) = self.synapse_nonplastic.get_nonplastic_synapse_data_from_source_neuron_index(
                DimensionalTypedNeuronIndex {
                    index: firing_neuron_index.clone(),
                    dimensional_type: DimensionCorticalAreaType::Core
                }
            )?;
            // TODO mp_charge_accumulation ? do we do it at the start of the burst?


            for downstream_synapse in downstream_synapses_iterator {
                let downstream_neuron_index = downstream_synapse.destination_neuron_index;


                
                
            }


        }



        Ok(())
        // TODO safer increment?
        //burst_index++; // TODO it may be better to increment this outside

    }

    //region Set Neuron Properties
    fn set_dimensional_neuron_fire_thresholds(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>, executor: &impl NeuronFireThresholdExecutor<Q::ValueQuant, Q::CoordQuantQuant>) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn set_dimensional_neuron_leak_coefficients(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>, executor: &impl NeuronFireThresholdExecutor<Q::ValueQuant, Q::CoordQuantQuant>) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }
    //endregion

    //region Utility and HouseKeeping
    fn prune_dead_synapses_and_neurons(&mut self) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn defragment_connectome(&mut self) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }
    //endregion
}

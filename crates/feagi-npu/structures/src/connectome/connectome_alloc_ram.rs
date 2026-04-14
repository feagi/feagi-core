
// NOTE: In alloc contexts, dont be too overspecific with quantization

use std::ops::Range;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::connectome::{ConnectomeAllocTrait, ConnectomeBaseTrait};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::FeagiNPUStructureError;
use crate::fire_candidate_list::{FireCandidateListRam, FireCandidateListTrait};
use crate::fire_queue::{FireQueueRam, FireQueueTrait};
use crate::neuron::base_dimension_traits::DimensionalAllocStorageTrait;
use crate::neuron::dimensional_neurons::DimensionalNeuronAllocRAMStorage;
use crate::neuron::dimensional_neurons::shared_funcs_and_structs::DimensionalNeuronDataFromCorticalArea;
use crate::quantizables::{NPUQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability, SynapseBundleIndex, SynapseCount};
use crate::synapse::non_plastic_dimensional::NonplasticDimensionalSynapseAllocRAMStorage;
use crate::synapse::non_plastic_dimensional::traits::{NonplasticSynapseAllocStorageTrait, NonplasticSynapseBaseStorageTrait};

pub struct ConnectomeAllocRam<Q: NPUQuantization>
{
    fire_queue: FireQueueRam<Q::NeuronIndex>,
    fire_candidate_list_ram: FireCandidateListRam<Q::NeuronIndex>,


    // Neurons
    neurons_dimensional: DimensionalNeuronAllocRAMStorage<Q>,
    
    // Synapses
    synapse_nonplastic: NonplasticDimensionalSynapseAllocRAMStorage<Q>
}


impl<Q: NPUQuantization>
ConnectomeAllocRam<Q>
{
    pub fn new(
        preallocated_dimensional_neuron_count: NeuronCount<Q::NeuronIndex>,
        preallocated_nonplastic_dimensional_synapse_count: SynapseCount<Q::SynapseIndex>
    ) -> Self {
        Self {
            fire_queue: FireQueueRam::new(0),
            fire_candidate_list_ram: FireCandidateListRam::new(0),
            neurons_dimensional: DimensionalNeuronAllocRAMStorage::new(preallocated_dimensional_neuron_count, CorticalAreaIndex::ZERO), // TODO prellocate cortical areas?
            synapse_nonplastic: NonplasticDimensionalSynapseAllocRAMStorage::new(preallocated_nonplastic_dimensional_synapse_count),
        }
    }
}

impl<Q: NPUQuantization>
ConnectomeAllocTrait<Q> for
ConnectomeAllocRam<Q>
{

    //region Cortical Areas

    //region Dimensional Neuron Cortical Areas

    fn create_dimensional_neuron_cortical_area_with_default_neurons(&mut self,
                                                                    cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                                    neurons_per_voxel: NumberNeuronsPerVoxel)
        -> Result<(CorticalAreaIndex<Q::CorticalIndex>, Range<NPUNeuronIndex<Q::NeuronIndex>>), FeagiNPUStructureError> {
        self.neurons_dimensional.create_cortical_area_with_default_neurons(
            cortical_area_dimensions,
            neurons_per_voxel
        ).map_err(|err| err.into())
    }

    fn create_dimensional_neuron_cortical_area_with_uniform_neurons(&mut self,
                                                                    cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
                                                                    neurons_per_voxel: NumberNeuronsPerVoxel,
                                                                    neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::BurstIndex>,
                                                                    neuron_membrane_potential: NPUNeuronMembranePotential<Q::Value>,
                                                                    neuron_fire_threshold: FireThreshold<Q::Value>,
                                                                    neuron_leak_coefficient: LeakCoefficient<Q::Percentage>,
                                                                    neuron_refractory_countdown: BurstDelta<Q::BurstDelta>,
                                                                    neuron_consecutive_fire_count: BurstDelta<Q::BurstDelta>,
                                                                    cortical_excitability: NeuronExcitability<Q::Percentage>,
                                                                    cortical_refractory_period_limit: BurstDelta<Q::BurstDelta>,
                                                                    cortical_fire_threshold_limit: FireThresholdLimit<Q::Value>,
                                                                    cortical_consecutive_fire_limit: BurstDelta<Q::BurstDelta>,
                                                                    cortical_is_mp_charge_accumulation_enabled: bool,
                                                                    cortical_is_mp_driven_psp_enabled: bool)
        -> Result<CorticalAreaIndex<Q::CorticalIndex>, FeagiNPUStructureError>
    {
        todo!()
    }

    fn create_dimensional_neuron_cortical_area_with_individualized_neurons(&mut self, cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>, neurons_per_voxel: NumberNeuronsPerVoxel, neuron_data: DimensionalNeuronDataFromCorticalArea<Q>) -> Result<CorticalAreaIndex<Q::CorticalIndex>, FeagiNPUStructureError> {
        todo!()
    }

    fn resize_dimensional_neuron_cortical_area<'a>(&mut self, cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>, neurons_per_voxel: NumberNeuronsPerVoxel, cortical_index: CorticalAreaIndex<Q::CorticalIndex>, presynaptic_dimensional_mappings: &Vec<(CorticalAreaIndex<Q::CorticalIndex>, DimensionCorticalAreaType, &'a impl NonPlasticCorticalMappingDefinitionExecutor<Q::NeuronIndex, Q::SynapseIndex, Q::Coord, Q::CorticalIndex, Q::BurstDelta, Q::Value>)>, postsynaptic_dimensional_mappings: &Vec<(CorticalAreaIndex<Q::CorticalIndex>, DimensionCorticalAreaType, &'a impl NonPlasticCorticalMappingDefinitionExecutor<Q::NeuronIndex, Q::SynapseIndex, Q::Coord, Q::CorticalIndex, Q::BurstDelta, Q::Value>)>) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn delete_dimensional_neuron_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndex>) -> Result<Range<NPUNeuronIndex<Q::NeuronIndex>>, FeagiNPUStructureError> {
        todo!()
    }

    //endregion

    //endregion

    //region Synapses

    //region dimensional area to dimensional area

    fn disconnect_all_synapses_from_dimensional_area_to_dimensional_area(&mut self, source_index: CorticalAreaIndex<Q::CorticalIndex>, destination_index: CorticalAreaIndex<Q::CorticalIndex>) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn add_nonplastic_connection_from_dimensional_area_to_dimensional_area(&mut self, source_area_index: CorticalAreaIndex<Q::CorticalIndex>, destination_area_index: CorticalAreaIndex<Q::CorticalIndex>, neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<Q::NeuronIndex, Q::SynapseIndex, Q::Coord, Q::CorticalIndex, Q::BurstDelta, Q::Value>) -> Result<SynapseBundleIndex<Q::SynapseBundleIndex>, FeagiNPUStructureError> {
        let synapse_bundle_index = self.synapse_nonplastic.add_synapses_mapping_between_cortical_areas(
            source_area_index,
            source_neuron_indexes,
            source_neuron_flags,
            source_cortical_dimensions,
            source_neuron_density,
            destination_area_index,
            destination_neuron_indexes,
            destination_neuron_flags: &[NeuronFlag],
            destination_cortical_dimensions,
            destination_neuron_density,
            neuron_mapping_executor).map_err(|err| err.into())?;
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

impl<Q: NPUQuantization>
ConnectomeBaseTrait<Q> for
ConnectomeAllocRam<Q>
{
    fn process_burst(&mut self, burst_index: &BurstGlobalIndex<<Q as NPUQuantization>::BurstIndex>) {

        self.fire_candidate_list_ram.clear();

        // TODO rayon feature swapper

        // NOTE I dont think its a good idea to have injectors run in here, they should be done before
        // maybe motor / sensor stuff should be injected to areas directly

        // TODO many errors here can only occur if something went very wrong. We should map them appropriately to be clear

        for firing_neuron_index in &self.fire_queue.get_dimensional_neuron_indexes_slice() {
            let downstream_synapses = self.synapse_nonplastic.get_nonplastic_synapse_data_from_source_neuron_index(firing_neuron_index)?;
            // TODO mp_charge_accumulation ? do we do it at the start of the burst?
            
            
            for downstream_synapse in downstream_synapses {
                let downstream_neuron_index = downstream_synapse.destination_neuron;
                self.neurons_dimensional.membrane_potential(downstream_neuron_index)
                    .update_threshold_nonplastic(
                        downstream_synapse.weight,
                        self.neurons_dimensional.membrane_potential(firing_neuron_index)
                    )
                
                
            }


        }




        // TODO safer increment?
        //burst_index++; // TODO it may be better to increment this outside

    }

    //region Set Neuron Properties
    fn set_dimensional_neuron_fire_thresholds(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndex>, executor: &impl NeuronFireThresholdExecutor<Q::Value, Q::Coord>) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn set_dimensional_neuron_leak_coefficients(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndex>, executor: &impl NeuronFireThresholdExecutor<Q::Value, Q::Coord>) -> Result<(), FeagiNPUStructureError> {
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

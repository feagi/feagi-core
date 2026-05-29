
// we may need to make this an enum wrapping a type, with runtime checks to prevent invalid calls

use std::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxel::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NumberNeuronsPerVoxel};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::FeagiNPUStructureError;

use crate::quantizables::{NPUGlobalQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability, SynapseBundleIndex, NPUNeuronMembranePotential};

pub trait ConnectomeBaseTrait<Q: NPUGlobalQuantization>
{

    fn process_burst(&mut self, burst_index: &BurstGlobalIndex<Q::GlobalBurstIndexQuant>) -> Result<(), FeagiNPUStructureError>;


    //region Set Neuron Properties

    fn set_dimensional_neuron_fire_thresholds(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                              executor: &impl NeuronFireThresholdExecutor<Q::ValueQuant, Q::CoordQuantQuant>)
                                              -> Result<(), FeagiNPUStructureError>;

    fn set_dimensional_neuron_leak_coefficients(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                executor: &impl NeuronFireThresholdExecutor<Q::ValueQuant, Q::CoordQuantQuant>)
                                                -> Result<(), FeagiNPUStructureError>;

    //endregion


    //region Utility and housekeeping

    //fn compute_minimum_possible_quantization_of_all_types(&self); // TODO what do we return here, a struct of enums???

    /// Where applicable, removes dead synapses, then dead neurons. Does not free memory
    fn prune_dead_synapses_and_neurons(&mut self) -> Result<(), FeagiNPUStructureError>;
    // TODO RISK OF BUG: we must ensure we prune all connected synapses to a dead neuron first to avoid unintended connections!
    // TODO seperate marking as invalid, and deleting
    // TODO clean up a list of cortical areas to cleanup
    // todo specify synapses with key input cortical area output cortical area
    // todo get voxel definitition, get data of each neuron inside seperately

    // NOTE: neuron and synapse defragging is paired as they are are interwoven
    /// Sort stored data across neurons and synapses for more optimal data reads and to allow freeing
    /// of unused memory.
    fn defragment_connectome(&mut self) -> Result<(), FeagiNPUStructureError>; // TODO not embedded?

    // TODO Limits / statistics

    //endregion

}

/// Connectome functions ONLY for static implementations
pub trait ConnectomeStaticTrait<Q: NPUGlobalQuantization> // TODO const sizes
{
    // TODO
}

/// Connectome functions ONLY for alloc capable implementations
pub trait ConnectomeAllocTrait<Q: NPUGlobalQuantization>
{
    // NOTE: We will not store mapping definitions in the connectome since that takes space and is
    // the job of the genome. We do not want to replicate cached data and maintain it!
    
    // TODO discuss insane idea for storing data for sensors / motors

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
                                                    -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUStructureError>;

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
                                                             -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUStructureError>;

    /// Create interneuron (custom) cortical area with given per neuron values
    fn create_interneuron_cortical_area_with_individualized_neurons(&mut self,
                                                                    cortical_area_dimensions: NeuronVoxelDimensions<Q::CoordQuantQuant>,
                                                                    neurons_per_voxel: NumberNeuronsPerVoxel,
                                                                    neuron_data: DimensionalNeuronDataFromCorticalArea<Q>)
                                                                    -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUStructureError>;
    

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

                                            -> Result<(), FeagiNPUStructureError>;


    /// First deletes any synaptic connections to / from this area, then deletes the interneuron
    /// cortical area
    fn delete_interneuron_cortical_area(&mut self,
                                               cortical_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>)
                                               -> Result<(), FeagiNPUStructureError>;

    //endregion


    //endregion

    //endregion

    //region Synapses

    //region dimensional area to dimensional area

    // NOTE: These functions exist under alloc since in static contexts we will not be
    // dynamically creating / destroying synapses between dimensional areas

    /// Adds synapse mappings between 2 cortical areas as defined by a given neuron mapping executor.
    /// Returns the synapse bundle index of the mapping
    fn add_nonplastic_connection_from_dimensional_area_to_dimensional_area(&mut self,
                                                                           source_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                                           source_area_dimension_type: DimensionCorticalAreaType,
                                                                           destination_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                                           destination_area_dimension_type: DimensionCorticalAreaType,
                                                                           neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<Q>)
                                                                           -> Result<SynapseBundleIndex<Q::SynapseBundleIndexQuant>, FeagiNPUStructureError>;
    
    /// Disconnects all synapses between 2 dimensional cortical areas
    fn disconnect_all_synapses_from_dimensional_area_to_dimensional_area(&mut self,
                                                                         source_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,
                                                                         destination_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>,)
                                                                         -> Result<(), FeagiNPUStructureError>;
    

    /*
    /// Disconnects just the synapses of the given synapse bundle index between
    /// 2 dimensional cortical areas
    fn disconnect_specific_nonplastic_synapse_bundle_from_dimensional_area_to_dimensional_area(&mut self,
                                                                                     source_index: CorticalAreaIndex<Q::CorticalIndex>,
                                                                                     destination_index: CorticalAreaIndex<Q::CorticalIndex>,
                                                                                     synapse_bundle_index: SynapseBundleIndex<Q::SynapseBundleIndex>)
        -> Result<(), FeagiNPUStructureError>;

     */


    /*
    // TODO this may belong under base
    fn connect_dimensional_area_to_dimensional_area_plastic(&mut self,
                                                               source_index: CorticalAreaIndex<Q::CorticalIndex>,
                                                               destination_index: CorticalAreaIndex<Q::CorticalIndex>, TODO);

     */




    //endregion

    //endregion

    //region Housekeeping
    fn free_unused_neuron_capacity(&mut self); // TODO take spare capacity number

    fn free_unused_synapse_capacity(&mut self);

    fn free_unused_cortical_area_capacity(&mut self); // TODO may not be needed?

    //endregion



}

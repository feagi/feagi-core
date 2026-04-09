
// we may need to make this an enum wrapping a type, with runtime checks to prevent invalid calls

use std::ops::Range;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::FeagiNPUStructureError;
use crate::neuron::dimensional_neurons::shared_funcs_and_structs::DimensionalNeuronDataFromCorticalArea;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability};

pub trait ConnectomeBaseTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    fn process_burst(&mut self, burst_index: BurstGlobalIndex<BurstIndexQuant>);


    //region Set Neuron Properties

    fn set_dimensional_neuron_fire_thresholds(&mut self, cortical_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                       executor: &impl NeuronFireThresholdExecutor<ValueQuant, CoordQuant>)
                                                     -> Result<(), FeagiNPUStructureError>;

    fn set_dimensional_neuron_leak_coefficients(&mut self, cortical_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                         executor: &impl NeuronFireThresholdExecutor<ValueQuant, CoordQuant>)
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
pub trait ConnectomeStaticTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> // TODO const sizes
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    // TODO
}

/// Connectome functions ONLY for alloc capable implementations
pub trait ConnectomeAllocTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    fn free_unused_neuron_capacity(&mut self); // TODO take spare capacity number

    fn free_unused_synapse_capacity(&mut self);

    fn free_unused_cortical_area_capacity(&mut self); // TODO may not be needed?

    //region Cortical Areas

    //region DimensionalNeuron Cortical Areas

    /// Create dimensional_neuron (custom) cortical area with default neuron settings spanned across the
    /// entire cortical area. Returns the cortical index of this new area.
    fn create_dimensional_neuron_cortical_area_with_default_neurons(&mut self,
                                                             cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel)
                                                             ->  Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUStructureError>;

    /// Create dimensional_neuron (custom) cortical area with given neuron settings spanned across the
    /// entire cortical area. Returns the cortical index of this new area.
    fn create_dimensional_neuron_cortical_area_with_uniform_neurons(&mut self, // TODO change other instances of spanned to uniform
                                                             cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel,
                                                             neuron_global_burst_index_of_last_firing: BurstGlobalIndex<BurstIndexQuant>,
                                                             neuron_membrane_potential: NeuronMembranePotential<ValueQuant>,
                                                             neuron_fire_threshold: FireThreshold<ValueQuant>,
                                                             neuron_leak_coefficient: LeakCoefficient<PercentageQuant>,
                                                             neuron_refractory_countdown: BurstDelta<BurstDeltaQuant>,
                                                             neuron_consecutive_fire_count: BurstDelta<BurstDeltaQuant>,
                                                             cortical_excitability: NeuronExcitability<PercentageQuant>,
                                                             cortical_refractory_period_limit: BurstDelta<BurstDeltaQuant>,
                                                             cortical_fire_threshold_limit: FireThresholdLimit<ValueQuant>,
                                                             cortical_consecutive_fire_limit: BurstDelta<BurstDeltaQuant>,
                                                             cortical_is_mp_charge_accumulation_enabled: bool,
                                                             cortical_is_mp_driven_psp_enabled: bool)
                                                             -> Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUStructureError>;

    /// Create dimensional_neuron (custom) cortical area with given per neuron values
    fn create_dimensional_neuron_cortical_area_with_individualized_neurons(&mut self,
                                                                    cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                                    neurons_per_voxel: NumberNeuronsPerVoxel,
                                                                    neuron_data: DimensionalNeuronDataFromCorticalArea<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>)
                                                                    -> Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUStructureError>;


    // NOTE: We will not store mapping definitions in the connectome since that takes space and is
    // the job of the genome. We do not want to replicate cached data and maintain it!

    /// Resizes an dimensional neuron cortical area toa  new dimension and or density. Attempts to maintain
    /// cortical area level values, but per neuron values will be reset! First disconnects
    /// all existing synapses, saves the cortical level values, deletes the cortical area,
    /// recreates the cortical area with
    /// the new dimensions and cortical level settings and then reestablishes the synapses
    /// anew with given mappers (should be the same as what was had before)
    fn resize_dimensional_neuron_cortical_area<'a>(&mut self,
                                                   cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                   neurons_per_voxel: NumberNeuronsPerVoxel,
                                                   cortical_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   presynaptic_dimensional_mappings: &Vec<(
                                                CorticalAreaIndex<CorticalIndexQuant>,
                                                DimensionCorticalAreaType, &'a impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>)>,
                                                   postsynaptic_dimensional_mappings: &Vec<(
                                                CorticalAreaIndex<CorticalIndexQuant>,
                                                DimensionCorticalAreaType, &'a impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>)>, )

                                                   -> Result<(), FeagiNPUStructureError>;


    /// First deletes any synaptic connections to / from this area, then deletes the dimensional_neuron
    /// cortical area
    fn delete_dimensional_neuron_cortical_area(&mut self,
                                        cortical_index: CorticalAreaIndex<CorticalIndexQuant>)
                                        -> Result<Range<NPUNeuronIndex<NeuronIndexQuant>>, FeagiNPUStructureError>;

    //endregion

    //endregion

    //region Synapses

    //region dimensional area to dimensional area

    // NOTE: These functions exist under alloc since in static contexts we will not be
    // dynamically creating / destroying nonplastic synapses

    fn disconnect_all_synapses_from_dimensional_area_to_dimensional_area(&mut self,
                                                                         source_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                                         source_dimensional_type: DimensionCorticalAreaType,
                                                                         destination_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                                         destination_dimensional_type: DimensionCorticalAreaType)
        -> Result<(), FeagiNPUStructureError>;

    fn add_nonplastic_connection_from_dimensional_area_to_dimensional_area(&mut self,
                                                               source_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                               destination_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                               neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>)
        -> Result<(), FeagiNPUStructureError>;

    // TODO setting mappings from a vector -> possibly complicated since we need to clear synapses, and then go through multiple vectors


    /*
    // TODO this may belong under base
    fn connect_dimensional_area_to_dimensional_area_plastic(&mut self,
                                                               source_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                               destination_index: CorticalAreaIndex<CorticalIndexQuant>, TODO);

     */




    //endregion

    //endregion



}
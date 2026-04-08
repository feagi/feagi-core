


// we may need to make this an enum wrapping a type, with runtime checks to prevent invalid calls

use std::ops::Range;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::executors::synapse_mapper_executors::SynapseNeuronMapperDim2DimExecutor;
use crate::FeagiNPUStructureError;
use crate::neuron::interneuron::shared_funcs_and_structs::InterneuronDataFromCorticalArea;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability};

pub trait ConnectomeBaseTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    fn process_burst(&mut self);

    //fn get_fire_candidate_list_ref(&self) -> &impl FireCandidateList;

    //fn get_fire_candidate_list_ref_mut(&self) -> &mut impl FireCandidateList;


    //region Set Neuron Properties

    fn set_interneuron_fire_thresholds(&mut self, cortical_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                       executor: &impl NeuronFireThresholdExecutor<PotentialQuant, CoordQuant>)
                                                     -> Result<(), FeagiNPUStructureError>;

    fn set_interneuron_leak_coefficients(&mut self, cortical_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                         executor: &impl NeuronFireThresholdExecutor<PotentialQuant, CoordQuant>)
                                       -> Result<(), FeagiNPUStructureError>;

    //endregion


    //region Utility and housekeeping

    fn compute_minimum_possible_quantization_of_all_types(&self); // TODO what do we return here, a struct of enums???

    // NOTE: neuron and synapse defragging is paired as they are are interwoven
    /// Sort stored data across neurons and synapses for more optimal data reads and to allow freeing
    /// of unused memory.
    fn defragment_connectome(&mut self) -> Result<(), FeagiNPUStructureError>;

    // TODO Limits / statistics

    //endregion

}

/// Connectome functions ONLY for static implementations
pub trait ConnectomeStaticTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant> // TODO const sizes
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    // TODO
}


pub trait ConnectomeAllocTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    PotentialQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    fn free_unused_neuron_capacity(&mut self);

    fn free_unused_synapse_capacity(&mut self);

    fn free_unused_cortical_area_capacity(&mut self); // TODO may not be needed?

    //region Synapses

    //region dimensional area to dimensional area

    // NOTE: These functions exist under alloc since in static contexts we will not be
    // dynamically creating / destroying nonplastic synapses

    fn connect_dimensional_area_to_dimensional_area_nonplastic(&mut self,
                                                               source_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                               source_dimensional_type: DimensionCorticalAreaType,
                                                               destination_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                               destination_dimensional_type: DimensionCorticalAreaType,
                                                               neuron_mapping_executor: &impl SynapseNeuronMapperDim2DimExecutor<NeuronIndexQuant, CoordQuant>)
        -> Result<(), FeagiNPUStructureError>; // TODO parameters

    fn disconnect_all_synapses_from_dimensional_area_to_dimensional_area_nonplastic(&mut self,
                                                                  source_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                                  source_dimensional_type: DimensionCorticalAreaType,
                                                                  destination_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                                  destination_dimensional_type: DimensionCorticalAreaType)
        -> Result<(), FeagiNPUStructureError>;






    /*
    // TODO this may belong under base
    fn connect_dimensional_area_to_dimensional_area_plastic(&mut self,
                                                               source_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                               destination_index: CorticalAreaIndex<CorticalIndexQuant>, TODO);

     */




    //endregion

    //endregion

    //region Cortical Areas
    
    //region Interneuron Cortical Areas

    /// Create interneuron (custom) cortical area with default neuron settings spanned across the
    /// entire cortical area. Returns the cortical index of this new area.
    fn create_interneuron_cortical_area_with_default_neurons(&mut self,
                                                             cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel)
        ->  Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUStructureError>;

    /// Create interneuron (custom) cortical area with given neuron settings spanned across the
    /// entire cortical area. Returns the cortical index of this new area.
    fn create_interneuron_cortical_area_with_uniform_neurons(&mut self, // TODO change other instances of spanned to uniform
                                                             cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel,
                                                             neuron_global_burst_index_of_last_firing: BurstGlobalIndex<BurstIndexQuant>,
                                                             neuron_membrane_potential: NeuronMembranePotential<PotentialQuant>,
                                                             neuron_fire_threshold: FireThreshold<PotentialQuant>,
                                                             neuron_leak_coefficient: LeakCoefficient<PercentageQuant>,
                                                             neuron_refractory_countdown: BurstDelta<BurstDeltaQuant>,
                                                             neuron_consecutive_fire_count: BurstDelta<BurstDeltaQuant>,
                                                             cortical_excitability: NeuronExcitability<PercentageQuant>,
                                                             cortical_refractory_period_limit: BurstDelta<BurstDeltaQuant>,
                                                             cortical_fire_threshold_limit: FireThresholdLimit<PotentialQuant>,
                                                             cortical_consecutive_fire_limit: BurstDelta<BurstDeltaQuant>,
                                                             cortical_is_mp_charge_accumulation_enabled: bool,
                                                             cortical_is_mp_driven_psp_enabled: bool)
        -> Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUStructureError>;

    /// Create interneuron (custom) cortical area with given per neuron values
    fn create_interneuron_cortical_area_with_individualized_neurons(&mut self,
                                                                cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                                neurons_per_voxel: NumberNeuronsPerVoxel,
                                                                neuron_data: InterneuronDataFromCorticalArea<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>)
        -> Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUStructureError>;


    // NOTE: We will not store mapping definitions in the connectome since that takes space and is
    // the job of the genome. We do not want to replicate cached data and maintain it!

    // TODO take other types of synaptic mappings too
    // TODO parameters!

    /// Resizes an interneuron cortical area toa  new dimension and or density. Attempts to maintain
    /// cortical area level settings, but per neuron settings will be reset! Also, first disconnects
    /// all existing synapses, saves the cortical level settings, recreates the cortical area with
    /// the new dimensions and cortical level settings and then reestablishes the synapses
    /// anew with given mappers (should be the same as what was had before)
    fn resize_interneuron_cortical_area(&mut self,
                                        cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                        neurons_per_voxel: NumberNeuronsPerVoxel,
                                        cortical_index: CorticalAreaIndex<CorticalIndexQuant>,
                                        presynaptic_dimensional_mappings: Vec<(
                                            CorticalAreaIndex<CorticalIndexQuant>,
                                            DimensionCorticalAreaType, &impl SynapseNeuronMapperDim2DimExecutor<NeuronIndexQuant, CoordQuant>)>,
                                        postsynaptic_dimensional_mappings: Vec<(
                                            CorticalAreaIndex<CorticalIndexQuant>,
                                            DimensionCorticalAreaType, &impl SynapseNeuronMapperDim2DimExecutor<NeuronIndexQuant, CoordQuant>)>, )

        -> Result<(Range<NPUNeuronIndex<NeuronIndexQuant>>, Range<NPUNeuronIndex<NeuronIndexQuant>>), FeagiNPUStructureError>;


    /// First deletes any synaptic connections to / from this area, then deletes the interneuron
    /// cortical area
    fn delete_interneuron_cortical_area(&mut self,
                                        cortical_index: CorticalAreaIndex<CorticalIndexQuant>)
        -> Result<Range<NPUNeuronIndex<NeuronIndexQuant>>, FeagiNPUStructureError>;

    //endregion

    //endregion


}
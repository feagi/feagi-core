



// we may need to make this an enum wrapping a type, with runtime checks to prevent invalid calls

pub trait ConnectomeBaseTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt,
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{

    fn process_burst(&mut self, fire_queue: &mut FireQueue, fire_candidate_list: &mut FireCandidateList); // TODO pass through types of FCL, FQ as mutable references

    //region Utility and housekeeping

    // NOTE: neuron and synapse defragging is paired as they are are interwoven
    /// Sort stored data across neurons and synapses for more optimal data reads and to allow freeing
    /// of unused memory.
    fn defragment_connectome(&mut self);

    fn compute_minimum_possible_quantization_of_all_types(&self); // TODO what do we return here, a struct of enums???

    // TODO Limits / statistics

    //endregion

    //region Set Neuron Properties

    fn set_interneuron_fire_thresholds(&mut self, cortical_area_index: CorticalIndexQuant,
                                       executor: &Impl<NeuronFireThresholdExecutor>)
                                                     -> Result<(), FeagiNPUDataError>;

    fn set_interneuron_leak_coefficients(&mut self, cortical_area_index: CorticalIndexQuant,
                                         executor: &Impl<NeuronLeakCoefficientExecutor>)
                                       -> Result<(), FeagiNPUDataError>;

    //endregion



}

/// Connectome functions ONLY for static implementations
pub trait ConnectomeStaticTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> // TODO const sizes
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{
    // TODO
}



pub trait ConnectomeAllocTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant> where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{

    fn free_unused_neuron_capacity(&mut self);

    fn free_unused_synapse_capacity(&mut self);

    fn free_unused_cortical_area_capacity(&mut self); // TODO may not be needed?

    //region Synapses
    // NOTE: These connection functions exist under alloc since in static contexts we will not be
    // dynamically creating / destroying them

    fn connect_dimensional_area_to_dimensional_area_nonplastic(&mut self,
                                                               source_index: CorticalIndexQuant,
                                                               destination_index: CorticalIndexQuant, TODO);

    fn connect_dimensional_area_to_dimensional_area_plastic(&mut self,
                                                               source_index: CorticalIndexQuant,
                                                               destination_index: CorticalIndexQuant, TODO);

    //endregion

    //region Interneuron Cortical Areas


    fn create_interneuron_cortical_area_with_default_neurons(&mut self,
                                                             cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel)
        ->  Result<CorticalIndexQuant, FeagiNPUDataError>;

    fn create_interneuron_cortical_area_with_uniform_neurons(&mut self, // TODO change other instances of spanned to uniform
                                                             cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel,
                                                             neuron_global_burst_index_of_last_firing: BurstIndexQuant,
                                                             neuron_membrane_potential: PotentialQuant,
                                                             neuron_fire_threshold: PotentialQuant,
                                                             neuron_leak_coefficient: PercentageQuant,
                                                             neuron_refractory_countdown: BurstDeltaQuant,
                                                             neuron_consecutive_fire_count: BurstDeltaQuant,
                                                             cortical_excitability: PercentageQuant,
                                                             cortical_refractory_period_limit: BurstDeltaQuant,
                                                             cortical_fire_threshold_limit: PotentialQuant,
                                                             cortical_consecutive_fire_limit: PotentialQuant,
                                                             cortical_is_mp_charge_accumulation_enabled: bool,
                                                             cortical_is_mp_driven_psp_enabled: bool)
        -> Result<CorticalIndexQuant, FeagiNPUDataError>;

    // TODO ask about passing structs or arrays?
    /*
    fn create_interneuron_cortical_area_with_configured_neurons(&mut self, cortical_area_data: InterneuronCorticalData,
                                                                neuron_data: InterneuronDataFromCorticalArea)
                                                                -> Result<(CorticalIndexQuant, Range<NeuronIndexQuant>), FeagiNPUDataError>)
     */



    fn resize_interneuron_cortical_area_with_default_neurons(&mut self,
                                                             cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel,
                                                             cortical_index: CorticalIndexQuant,
                                                             TODO)
        -> Result<(), FeagiNPUDataError>;

    // TODO resize with spanned?

    fn delete_interneuron_cortical_area(&mut self,
                                        cortical_index: CorticalIndexQuant)
        -> Result<(), FeagiNPUDataError>;

    //endregion



}
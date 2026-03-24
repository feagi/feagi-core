
// TODO extended traits with plastic support?

// TODO &ConnectivityFunction we cant have one universal type right? Different types on connectivity functions for different types of synapses
// we may need to make this an enum wrapping a type, with runtime checks to prevent invalid calls

pub trait ConnectomeBaseTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstQuant, PotentialQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstQuant: BurstDeltaCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{

    fn defragment_connectome(&mut self);

    fn process_burst(&mut self, fire_queue: &mut FireQueue, fire_candidate_list: &mut FireCandidateList); // TODO pass through types of FCL, FQ as mutable references

    // TODO Memory Specific interactions

    // TODO get connectome properties

    // TODO Limits / statistics


    fn compute_minimum_possible_quantization_of_all_types(&self); // TODO what do we return here, a struct of enums???

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

    // TODO above note about &ConnectivityFunction
    fn connect_nonplastic_synapse(&mut self, source_cortical_type: NPUNeuronType,
                                  source_cortical_index: CorticalIndexQuant,
                                  destination_cortical_type: NPUNeuronType,
                                  destination_cortical_index: CorticalIndexQuant,
                                  connectivity_function: &ConnectivityFunction);

    //endregion

    //region Interneuron Cortical Areas

    fn create_interneuron_cortical_area_with_default_neurons(&mut self,
                                                             cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>,
                                                             neurons_per_voxel: NumberNeuronsPerVoxel)
        ->  Result<CorticalIndexQuant, FeagiNPUDataError>;

    fn create_interneuron_cortical_area_with_spanned_neurons(&mut self,
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
                                                             connectivity_function: &ConnectivityFunction)
        -> Result<(), FeagiNPUDataError>;

    // TODO resize with spanned?

    fn delete_interneuron_cortical_area(&mut self,
                                        cortical_index: CorticalIndexQuant)
        -> Result<(), FeagiNPUDataError>;

    //endregion



}
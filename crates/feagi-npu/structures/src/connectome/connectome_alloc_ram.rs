
// NOTE: In alloc contexts, dont be too overspecific with quantization


pub struct ConnectomeAllocRam<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, SynapseIndexQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    SynapseIndexQuant: SynapseIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstDeltaQuant: BurstCount,
    BurstIndexQuant: BurstCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,

{
    // Neurons
    neuron_interneuron: InterneuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, PercentageQuant>,
    
    // Synapses
    synapse_nonplastic: NonplasticSynapseAllocRAMStorage<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant>
}


impl ConnectomeAllocRam<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, SynapseIndexQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    SynapseIndexQuant: SynapseIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstDeltaQuant: BurstCount,
    BurstIndexQuant: BurstCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{

    pub fn new(number_neurons_to_preallocate_space_for: NeuronIndexQuant, number_synapses_to_preallocate: SynapseIndexQuant, number_cortical_areas_to_preallocate_space_for: CorticalIndexQuant) -> Result<Self, FeagiNPUDataError> {
        Ok(Self {
            InterneuronAllocRAMStorage::new(number_neurons_to_preallocate_space_for, number_cortical_areas_to_preallocate_space_for),
            NonplasticSynapseAllocRAMStorage::new(number_synapses_to_preallocate),
        })
    }






}

impl ConnectomeBaseTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, SynapseIndexQuant, PercentageQuant> for ConnectomeAllocRam<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, SynapseIndexQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    SynapseIndexQuant: SynapseIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstDeltaQuant: BurstCount,
    BurstIndexQuant: BurstCount,
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

    // TODO get connectome properties

    // TODO Limits / statistics

    //endregion

    //region Set Neuron Properties

    // TODO there are faster ways if we just want to set a uniform type!

    fn set_interneuron_fire_thresholds(&mut self, cortical_area_index: CorticalIndexQuant,
                                       executor: &Impl<NeuronFireThresholdExecutor>)
                                       -> Result<(), FeagiNPUDataError> {
        &mut self.neuron_interneuron.set_fire_thresholds(cortical_area_index, increment_function)
    }

    fn set_interneuron_leak_coefficients(&mut self, cortical_area_index: CorticalIndexQuant,
                                         executor: &Impl<NeuronLeakCoefficientExecutor>)
                                         -> Result<(), FeagiNPUDataError> {
        &mut self.neuron_interneuron.set_leak_coefficient(cortical_area_index, increment_function)
    }

    //endregion

}


impl ConnectomeAllocTrait<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, SynapseIndexQuant, PercentageQuant> for ConnectomeAllocRam<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, PotentialQuant, SynapseIndexQuant, PercentageQuant>
where
    NeuronIndexQuant: InterneuronIndex,
    CorticalIndexQuant: CorticalAreaIndex,
    SynapseIndexQuant: SynapseIndex,
    CoordQuant: QuantizableUInt, // Using this here as we may be using coords or dimensions
    BurstDeltaQuant: BurstCount,
    BurstIndexQuant: BurstCount,
    PotentialQuant: PotentialUnit,
    PercentageQuant: PercentageScale,
{

    fn free_unused_neuron_capacity(&mut self, percent_to_keep: f32) -> Result<(), FeagiNPUDataError> {

        //self.neuron_interneuron.free_unused_neuron_capacity(number_interneurons_to_preserve);
        Ok(())
    }

    fn free_unused_synapse_capacity(&mut self, percent_to_keep: f32) {
        Ok(())
    }

    //region Synapses

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
                                                             connectivity_function: &ConnectivityFunction)
                                                             -> Result<(), FeagiNPUDataError>;

    // TODO resize with spanned?

    fn delete_interneuron_cortical_area(&mut self,
                                        cortical_index: CorticalIndexQuant)
                                        -> Result<(), FeagiNPUDataError>;



}




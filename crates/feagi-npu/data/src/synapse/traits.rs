

pub trait SynapseData<SynapseIndexAndSize, NeuronIndex, Weight, Potential>
where
    SynapseIndexAndSize: QuantizableUInt,
    NeuronIndex: QuantizableUInt,
    Weight: QuantizableValue,
    Potential: QuantizableUInt,
{

    fn number_of_active_synapses(&self) -> SynapseIndexAndSize;

    //fn current_synapse_capacity(&self) -> SynapseIndexAndSize;
    //fn set_current_synapse_capacity(&mut self, capacity: SynapseIndexAndSize); // TODO does this make sense?

    fn get_source_neuron(&self, synapse_index: SynapseIndexAndSize) -> Result<NeuronIndex, FeagiNPUDataError>;
    fn set_source_neuron(&mut self, synapse_index: SynapseIndexAndSize, source_neuron: NeuronIndex) -> Result<(), FeagiNPUDataError>;

    fn get_destination_neuron(&self, synapse_index: SynapseIndexAndSize) -> Result<NeuronIndex, FeagiNPUDataError>;
    fn set_destination_neuron(&mut self, synapse_index: SynapseIndexAndSize, destination_neuron: NeuronIndex) -> Result<(), FeagiNPUDataError>;

    fn get_weight(&self, synapse_index: SynapseIndexAndSize) -> Result<Weight, FeagiNPUDataError>;
    fn set_weight(&mut self, synapse_index: SynapseIndexAndSize, weight: Weight) -> Result<(), FeagiNPUDataError>;

    fn get_post_synaptic_potential(&self, synapse_index: SynapseIndexAndSize)  -> Result<Potential, FeagiNPUDataError>;
    fn set_post_synaptic_potential(&mut self, synapse_index: SynapseIndexAndSize, post_synaptic_potential: Potential) -> Result<(), FeagiNPUDataError>;

    fn get_synapse_flag(&self, synapse_index: SynapseIndexAndSize)  -> Result<SynapseFlag, FeagiNPUDataError>;
    fn set_synapse_flag(&mut self, synapse_index: SynapseIndexAndSize, synapse_flag: SynapseFlag) -> Result<(), FeagiNPUDataError>;
}


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


    // TODO out mut!
    fn get_source_neurons(&self, synapse_indexes: &[SynapseIndexAndSize]) -> Result<NeuronIndex, FeagiNPUDataError>;
    fn set_source_neurons(&mut self, synapse_indexes: &[SynapseIndexAndSize], source_neuron: NeuronIndex) -> Result<(), FeagiNPUDataError>;

    fn get_destination_neurons(&self, synapse_indexes: &[SynapseIndexAndSize]) -> Result<NeuronIndex, FeagiNPUDataError>;
    fn set_destination_neurons(&mut self, synapse_indexes: &[SynapseIndexAndSize], destination_neuron: NeuronIndex) -> Result<(), FeagiNPUDataError>;
1
    fn get_weights(&self, synapse_indexes: &[SynapseIndexAndSize]) -> Result<Weight, FeagiNPUDataError>;
    fn set_weights(&mut self, synapse_indexes: &[SynapseIndexAndSize], weight: Weight) -> Result<(), FeagiNPUDataError>;

    fn get_post_synaptic_potentials(&self, synapse_indexes: &[SynapseIndexAndSize])  -> Result<Potential, FeagiNPUDataError>;
    fn set_post_synaptic_potentials(&mut self, synapse_indexes: &[SynapseIndexAndSize], post_synaptic_potential: Potential) -> Result<(), FeagiNPUDataError>;

    fn get_synapse_flags(&self, synapse_indexes: &[SynapseIndexAndSize])  -> Result<SynapseFlag, FeagiNPUDataError>;
    fn set_synapse_flags(&mut self, synapse_indexes: &[SynapseIndexAndSize], synapse_flag: SynapseFlag) -> Result<(), FeagiNPUDataError>;
}
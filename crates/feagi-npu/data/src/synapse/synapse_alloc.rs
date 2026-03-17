


pub struct SynapseDataAlloc<SynapseIndexAndSize, NeuronIndex, Weight, Potential> {
    number_active_synapses: SynapseIndexAndSize, // Not the same as length! Some synapses may be dead! // TODO apply this for neurons too
    source_neurons: Vec<NeuronIndex>,
    destination_neurons: Vec<NeuronIndex>,
    weights: Vec<Weight>,
    postsynaptic_potentials: Vec<Potential>,
    synapse_flags: Vec<SynapseFlag>,
    source_2_destinations: AHashMap<NeuronIndex, Vec<Potential>>,
}

impl<SynapseIndexAndSize, NeuronIndex, Weight, Potential> SynapseData<SynapseIndexAndSize, NeuronIndex, Weight, Potential> for SynapseDataAlloc<SynapseIndexAndSize, NeuronIndex, Weight, Potential> {

    fn number_of_active_synapses(&self) -> SynapseIndexAndSize {
        self.number_active_synapses
    }

    //fn current_synapse_capacity(&self) -> SynapseIndexAndSize;
    //fn set_current_synapse_capacity(&mut self, capacity: SynapseIndexAndSize);

    fn get_source_neuron(&self, synapse_index: SynapseIndexAndSize) -> Result<NeuronIndex, FeagiNPUDataError> {
        Ok(
            self.source_neurons.get(synapse_index as usize).ok_or(FeagiNPUDataError::SynapseIndexNotFound)?
        )
    }
    fn set_source_neuron(&mut self, synapse_index: SynapseIndexAndSize, source_neuron: NeuronIndex) -> Result<(), FeagiNPUDataError> {
        if let Some(existing_source_neuron) = self.source_neurons.get_mut(synapse_index as usize) {
            *existing_source_neuron = source_neuron;
            return Ok(());
        }
        Err(FeagiNPUDataError::SynapseIndexNotFound)
    }

    fn get_destination_neuron(&self, synapse_index: SynapseIndexAndSize) -> Result<NeuronIndex, FeagiNPUDataError> {
        Ok(
            self.destination_neurons.get(synapse_index as usize).ok_or(FeagiNPUDataError::SynapseIndexNotFound)?
        )
    }
    fn set_destination_neuron(&mut self, synapse_index: SynapseIndexAndSize, destination_neuron: NeuronIndex) -> Result<(), FeagiNPUDataError> {
        if let Some(existing_destination_neuron) = self.destination_neurons.get_mut(synapse_index as usize) {
            *existing_destination_neuron = destination_neuron;
            return Ok(());
        }
        Err(FeagiNPUDataError::SynapseIndexNotFound)
    }

    fn get_weight(&self, synapse_index: SynapseIndexAndSize) -> Result<Weight, FeagiNPUDataError> {
        Ok(
            self.weights.get(synapse_index as usize).ok_or(FeagiNPUDataError::SynapseIndexNotFound)?
        )
    }
    fn set_weight(&mut self, synapse_index: SynapseIndexAndSize, weight: Weight) -> Result<(), FeagiNPUDataError> {
        if let Some(existing_weight) = self.weights.get_mut(synapse_index as usize) {
            *existing_weight = weight;
            return Ok(());
        }
        Err(FeagiNPUDataError::SynapseIndexNotFound)
    }

    fn get_post_synaptic_potential(&self, synapse_index: SynapseIndexAndSize)  -> Result<Potential, FeagiNPUDataError> {
        Ok(
            self.postsynaptic_potentials.get(synapse_index as usize).ok_or(FeagiNPUDataError::SynapseIndexNotFound)?
        )
    }
    fn set_post_synaptic_potential(&mut self, synapse_index: SynapseIndexAndSize, post_synaptic_potential: Potential) -> Result<(), FeagiNPUDataError> {
        if let Some(existing_post_synaptic_potential) = self.postsynaptic_potentials.get_mut(synapse_index as usize) {
            *existing_post_synaptic_potential = post_synaptic_potential;
            return Ok(());
        }
        Err(FeagiNPUDataError::SynapseIndexNotFound)
    }

    fn get_synapse_flag(&self, synapse_index: SynapseIndexAndSize)  -> Result<SynapseFlag, FeagiNPUDataError> {
        Ok(
            self.synapse_flags.get(synapse_index as usize).ok_or(FeagiNPUDataError::SynapseIndexNotFound)?
        )
    }
    fn set_synapse_flag(&mut self, synapse_index: SynapseIndexAndSize, synapse_flag: SynapseFlag) -> Result<(), FeagiNPUDataError> {
        if let Some(existing_synapse_flag) = self.synapse_flags.get_mut(synapse_index as usize) {
            *existing_synapse_flag = synapse_flag;
            return Ok(());
        }
        Err(FeagiNPUDataError::SynapseIndexNotFound)
    }
}
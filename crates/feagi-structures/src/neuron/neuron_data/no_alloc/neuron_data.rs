#[!no_std]

// TODO into error from indexer

pub struct NeuronDataStatic<P: NeuralPotentialValue> {
    potentials: Vec<P>,
    thresholds: Vec<P>,
}

impl NeuronData for NeuronDataStatic<NeuralPotentialValue> {
    fn get_total_number_of_neurons(&self) -> u32 {
        self.potentials.len() as u32
    }

    fn get_neuron_membrane_potential(&self, neuron_id: NeuronID) -> Result<P, > {
        self.potentials.get(*neuron_id)
            .map_err()
    }

    fn set_neuron_membrane_potential(&mut self, neuron_id: NeuronID, potential: P) -> Result<(), > {
        self.potentials.set(*neuron_id, potential)
            .map_err()
    }

    fn get_neuron_threshold(&self, neuron_id: NeuronID) -> Result<P, > {
        self.thresholds.get(*neuron_id)
            .map_err()
    }

    fn set_neuron_threshold(&mut self, neuron_id: NeuronID, threshold: P) -> Result<(), >{
        self.thresholds.get(*neuron_id)
            .map_err()
    }
}
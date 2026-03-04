

pub trait NeuronData<P: NeuralPotentialValue> {
    fn get_total_number_of_neurons(&self) -> u32;

    fn get_neuron_membrane_potential(&self, neuron_id: NeuronID) -> Result<P, >;

    fn set_neuron_membrane_potential(&mut self, neuron_id: NeuronID, potential: P) -> Result<(), >;

    fn get_neuron_threshold(&self, neuron_id: NeuronID) -> Result<P, >;

    fn set_neuron_threshold(&mut self, neuron_id: NeuronID, potential: P) -> Result<(), >;

}
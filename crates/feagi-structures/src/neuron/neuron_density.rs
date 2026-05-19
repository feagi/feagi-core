
/// Standard method of defining how many neurons are in some singular unit. Should not be 0/
pub trait NeuronDensity {
    fn number_of_neurons_per_unit(&self) -> u8;
    fn is_single_neuron(&self) -> bool {
        self.number_of_neurons_per_unit() == 1
    }

    fn is_multi_neuron(&self) -> bool {
        self.number_of_neurons_per_unit() != 1
    }

    fn is_invalid(&self) -> bool {
        self.number_of_neurons_per_unit() == 0
    }
}
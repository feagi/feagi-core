use feagi_structures_quantization::quantizable_base::QuantizedIndexCountTrait;

/// Standard method of defining how many neurons are in some singular unit. Should not be 0!
pub trait NeuronDensityTrait<QuantLinear: QuantizedIndexCountTrait> {
    fn number_of_neurons_per_unit(&self) -> QuantLinear;
    
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
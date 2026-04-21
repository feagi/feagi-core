use feagi_structures::define_bit_packed_u8_flags;
use crate::neuron::npu_neuron_type::NPUDimensionalNeuronType;

/// Stores various synapse boolean flags under a single byte. Meant to be used for all
/// synapse types
define_bit_packed_u8_flags! {
    pub struct SynapseFlag(
        valid,
        inhibitory,
        neuron_type_a,
        neuron_type_b,
        neuron_type_c,
        neuron_type_d,
        reserved_6,
        reserved_7,
    )
}

// TODO this can be optimized

impl SynapseFlag {
    pub fn get_source_npu_dimensional_neuron_type(&self) -> NPUDimensionalNeuronType {
        match (self.is_neuron_type_a(), self.is_neuron_type_b()) {
            (true, true) => NPUDimensionalNeuronType::Core,
            (true, false) => NPUDimensionalNeuronType::Sensory,
            (false, true) => NPUDimensionalNeuronType::Motor,
            (false, false) => NPUDimensionalNeuronType::Interneuron,
        }
    }

    pub fn get_destination_npu_dimensional_neuron_type(&self) -> NPUDimensionalNeuronType {
        match (self.is_neuron_type_c(), self.is_neuron_type_d()) {
            (true, true) => NPUDimensionalNeuronType::Core,
            (true, false) => NPUDimensionalNeuronType::Sensory,
            (false, true) => NPUDimensionalNeuronType::Motor,
            (false, false) => NPUDimensionalNeuronType::Interneuron,
        }
    }
    
    pub fn set_source_npu_dimension_neuron_type(&mut self, npu_dimension_neuron_type: &NPUDimensionalNeuronType) {
        match(npu_dimension_neuron_type) {
            NPUDimensionalNeuronType::Core => {
                self.set_neuron_type_a(true);
                self.set_neuron_type_b(true);
            }
            NPUDimensionalNeuronType::Sensory => {
                self.set_neuron_type_a(true);
                self.set_neuron_type_b(false);
            }
            NPUDimensionalNeuronType::Motor => {
                self.set_neuron_type_a(false);
                self.set_neuron_type_b(true);
            }
            NPUDimensionalNeuronType::Interneuron => {
                self.set_neuron_type_a(false);
                self.set_neuron_type_b(false);
            }
        }
    }

    pub fn set_destination_npu_dimension_neuron_type(&mut self, npu_dimension_neuron_type: &NPUDimensionalNeuronType) {
        match(npu_dimension_neuron_type) {
            NPUDimensionalNeuronType::Core => {
                self.set_neuron_type_c(true);
                self.set_neuron_type_d(true);
            }
            NPUDimensionalNeuronType::Sensory => {
                self.set_neuron_type_c(true);
                self.set_neuron_type_d(false);
            }
            NPUDimensionalNeuronType::Motor => {
                self.set_neuron_type_c(false);
                self.set_neuron_type_d(true);
            }
            NPUDimensionalNeuronType::Interneuron => {
                self.set_neuron_type_c(false);
                self.set_neuron_type_d(false);
            }
        }
    }
}




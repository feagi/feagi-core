use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::NeuronMembranePotential;
use crate::dynamics::neuron::shared::iteration::{PackedLinearIteration, PackedLinearIterationMut};
use crate::dynamics::neuron::shared::neurons::{NeuronDataRef, NeuronDataRefMut, NeuronModelParametersTrait};


/// Defines all the fields for a slice of all neurons as an immutable reference
pub struct NeuronModelSlice<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    /// Potential of neuron
    pub neuron_potentials: &'a [NeuronMembranePotential<CANQ::NeuronValueQuant>],

    /// All other parameters of neurons
    pub get_model_parameters: &'a [NMP]
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> PackedLinearIteration<CANQ, NMP> for NeuronModelSlice<'a, CANQ, NMP> {
    fn linear_neuron_iter(&self) -> impl Iterator<Item=NeuronDataRef<'_, CANQ, NMP>> {
        self.neuron_potentials
            .iter()
            .zip(self.get_model_parameters.iter())
            .map(|(potential, model_parameters)| NeuronDataRef::new(potential, model_parameters))
    }
}

/// Defines all the fields for a slice of all neurons as a mutable reference
pub struct NeuronModelMutSlice<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    /// Potential of neuron
    pub neuron_potentials: &'a mut [NeuronMembranePotential<CANQ::NeuronValueQuant>],

    /// All other parameters of neurons
    pub get_model_parameters: &'a mut [NMP]
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> PackedLinearIteration<CANQ, NMP> for NeuronModelMutSlice<'a, CANQ, NMP> {
    fn linear_neuron_iter(&self) -> impl Iterator<Item=NeuronDataRef<'_, CANQ, NMP>> {
        self.neuron_potentials
            .iter()
            .zip(self.get_model_parameters.iter())
            .map(|(potential, model_parameters)| NeuronDataRef::new(potential, model_parameters))
    }
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> PackedLinearIterationMut<CANQ, NMP> for NeuronModelMutSlice<'a, CANQ, NMP> {

    fn linear_neuron_iter_mut(&mut self) -> impl Iterator<Item=NeuronDataRefMut<'_, CANQ, NMP>> {
        self.neuron_potentials
            .iter_mut()
            .zip(self.get_model_parameters.iter_mut())
            .map(|(potential, model_parameters)| NeuronDataRefMut::new(potential, model_parameters))
    }
}
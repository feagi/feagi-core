use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::NeuronMembranePotential;

pub trait NeuronFiringResultTrait<'a, CANQ: CorticalAreaNeuronQuantization> {
    /// Is the neuron going to fire
    fn is_going_to_fire(&self) -> bool;

    /// What the final potential of the neuron is, expected to be a reference right from the
    /// potential collection
    fn final_potential(&self) -> &'a NeuronMembranePotential<CANQ::NeuronValueQuant>;
}


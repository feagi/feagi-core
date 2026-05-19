//! Traits for all base neurons, neuron refs, and neuron mutable refs

use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::NeuronMembranePotential;


#[derive(Clone)]
pub struct NeuronData<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    potential: NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: NMP,
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronData<CANQ, CANQ>
{
    pub(crate) fn new(potential: NeuronMembranePotential<CANQ::NeuronValueQuant>,
                      model_parameters: NMP) -> NeuronData<CANQ, CANQ>
    {
        NeuronData {
            potential,
            model_parameters
        }
    }
}

pub struct NeuronDataRef<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    potential: &'a NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: &'a NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataRef<CANQ, CANQ>
{
    pub(crate) fn new(potential: &'a NeuronMembranePotential<CANQ::NeuronValueQuant>,
                      model_parameters: &'a NMP) -> NeuronDataRef<'a, CANQ, CANQ>
    {
        NeuronDataRef {
            potential,
            model_parameters
        }
    }
}

pub struct NeuronDataRefMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    potential: &'a mut NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: &'a mut NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataRefMut<CANQ, CANQ>
{
    pub(crate) fn new(potential: &'a mut NeuronMembranePotential<CANQ::NeuronValueQuant>,
                      model_parameters: &'a mut NMP) -> NeuronDataRefMut<'a, CANQ, CANQ>
    {
        NeuronDataRefMut {
            potential,
            model_parameters
        }
    }
}


/// Defines any parameters you want per neuron for a given neuron model.
/// This is what models shall expand on!
pub trait NeuronModelParametersTrait<CANQ: CorticalAreaNeuronQuantization>:
Clone
{}


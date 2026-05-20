use feagi_structures::neuron::NeuronMembranePotential;
use feagi_structures::quantization::CorticalAreaNeuronQuantization;
use crate::neuron_models::neuron_models::NeuronModelParametersTrait;

pub trait NeuronDataRefTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> 
{
    fn get_potential(&self) -> &NeuronMembranePotential<CANQ::NeuronDecimalQuant>;
    fn get_model_parameters(&self) -> &NMP<CANQ>;
}

pub trait NeuronDataMutRefTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronDataRefTrait<CANQ, NMP>
{
    fn get_potential_mut(&mut self) -> &mut NeuronMembranePotential<CANQ::NeuronDecimalQuant>;
    fn get_model_parameters_mut(&mut self) -> &mut NMP<CANQ>;
}
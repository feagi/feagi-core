use feagi_structures::neuron::NeuronMembranePotential;
use feagi_structures::quantization::CorticalAreaNeuronQuantization;
use crate::external_models_make_me_my_own_crate::shared_traits_and_structs::NeuronModelParametersTrait;

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
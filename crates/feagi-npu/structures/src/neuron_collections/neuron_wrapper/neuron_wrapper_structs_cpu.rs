use feagi_structures::neuron::NeuronMembranePotential;
use feagi_structures::quantization::CorticalAreaNeuronQuantization;
use crate::neuron_collections::neuron_wrapper::neuron_wrapper_trait::{NeuronDataMutRefTrait, NeuronDataRefTrait};
use crate::neuron_models::neuron_models::NeuronModelParametersTrait;


//region Independent Neuron Struct
#[derive(Clone)]
pub struct NeuronDataCPU<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    potential: NeuronMembranePotential<CANQ::NeuronDecimalQuant>,
    model_parameters: NMP,
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataCPU<CANQ, NMP>
{
    pub(crate) fn new(potential: NeuronMembranePotential<CANQ::NeuronDecimalQuant>,
                      model_parameters: NMP) -> NeuronDataCPU<CANQ, NMP>
    {
        NeuronDataCPU {
            potential,
            model_parameters
        }
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataRefTrait<CANQ, NMP> for NeuronDataCPU<CANQ, NMP>{
    fn get_potential(&self) -> &NeuronMembranePotential<CANQ::NeuronDecimalQuant> {
        &self.potential
    }

    fn get_model_parameters(&self) -> &NMP<CANQ> {
        &self.model_parameters
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataMutRefTrait<CANQ, NMP> for NeuronDataCPU<CANQ, NMP>{
    fn get_potential_mut(&mut self) -> &mut NeuronMembranePotential<CANQ::NeuronDecimalQuant> {
        &mut self.potential
    }

    fn get_model_parameters_mut(&mut self) -> &mut NMP<CANQ> {
        &mut self.model_parameters
    }
}
//endregion


//region Immutable Neuron Reference
pub struct NeuronDataCPURef<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    potential: &'a NeuronMembranePotential<CANQ::NeuronDecimalQuant>,
    model_parameters: &'a NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataCPURef<'a, CANQ, NMP>
{
    pub(crate) fn new(potential: &'a NeuronMembranePotential<CANQ::NeuronDecimalQuant>,
                      model_parameters: &'a NMP) -> NeuronDataCPURef<'a, CANQ, NMP>
    {
        NeuronDataCPURef {
            potential,
            model_parameters
        }
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataRefTrait<CANQ, NMP> for NeuronDataCPURef<CANQ, NMP>{
    fn get_potential(&self) -> &NeuronMembranePotential<CANQ::NeuronDecimalQuant> {
        &self.potential
    }

    fn get_model_parameters(&self) -> &NMP<CANQ> {
        &self.model_parameters
    }
}
//endregion

//region Mutable Neuron Reference
pub struct NeuronDataCPURefMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    potential: &'a mut NeuronMembranePotential<CANQ::NeuronDecimalQuant>,
    model_parameters: &'a mut NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataCPURefMut<'a, CANQ, NMP>
{
    pub(crate) fn new(potential: &'a mut NeuronMembranePotential<CANQ::NeuronDecimalQuant>,
                      model_parameters: &'a mut NMP) -> NeuronDataCPURefMut<'a, CANQ, NMP>
    {
        NeuronDataCPURefMut {
            potential,
            model_parameters
        }
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataRefTrait<CANQ, NMP> for NeuronDataCPURefMut<CANQ, NMP>{
    fn get_potential(&self) -> &NeuronMembranePotential<CANQ::NeuronDecimalQuant> {
        &self.potential
    }

    fn get_model_parameters(&self) -> &NMP<CANQ> {
        &self.model_parameters
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronDataMutRefTrait<CANQ, NMP> for NeuronDataCPURefMut<CANQ, NMP>{
    fn get_potential_mut(&mut self) -> &mut NeuronMembranePotential<CANQ::NeuronDecimalQuant> {
        &mut self.potential
    }

    fn get_model_parameters_mut(&mut self) -> &mut NMP<CANQ> {
        &mut self.model_parameters
    }
}

//endregion
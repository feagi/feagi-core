use feagi_structures::base_feagi_types::quantizable_types::QuantizableUIntType;
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{LinearNeuronIndexCount, NeuronMembranePotential};
use crate::dynamics::neuron::shared::neuron_slices::{NeuronModelMutSlice, NeuronModelSlice};
use crate::dynamics::neuron::shared::neurons::{NeuronDataRef, NeuronDataRefMut, NeuronModelParametersTrait};


//region Linear Packed
pub trait PackedLinearIteration<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> {
    fn linear_neuron_iter(&self) -> impl Iterator<Item = NeuronDataRef<'_, CANQ, NMP>>;
}

pub trait PackedLinearIterationMut<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
PackedLinearIteration<CANQ, NMP>
{
    fn linear_neuron_iter_mut(&mut self) -> impl Iterator<Item = NeuronDataRefMut<'_, CANQ, NMP>>;
}

//endregion

//region Enumerated
pub struct EnumeratedLinearNeuron<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    potential: &'a NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: &'a NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> EnumeratedLinearNeuron<'a, CANQ, NMP> {
    pub(crate) fn new(
        linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
        potential: &'a NeuronMembranePotential<CANQ::NeuronValueQuant>,
        model_parameters: &'a NMP,
    ) -> Self {
        Self {
            linear_neuron_index,
            potential,
            model_parameters,
        }
    }

    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> NeuronDataRef<'a, CANQ, NMP> {
        NeuronDataRef::new(self.potential, self.model_parameters)
    }
}

pub struct EnumeratedLinearNeuronMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    potential: &'a mut NeuronMembranePotential<CANQ::NeuronValueQuant>,
    model_parameters: &'a mut NMP,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> EnumeratedLinearNeuronMut<'a, CANQ, NMP> {
    pub(crate) fn new(
        linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
        potential: &'a mut NeuronMembranePotential<CANQ::NeuronValueQuant>,
        model_parameters: &'a mut NMP,
    ) -> Self {
        Self {
            linear_neuron_index,
            potential,
            model_parameters,
        }
    }

    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> NeuronDataRef<'a, CANQ, NMP> {
        NeuronDataRef::new(self.potential, self.model_parameters)
    }

    pub fn neuron_ref_mut(&mut self) -> NeuronDataRefMut<'_, CANQ, NMP> {
        NeuronDataRefMut::new(self.potential, self.model_parameters)
    }
}




pub struct EnumeratedLinearSetNeuron<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> {
    neuron_set_index: NeuronSetIndexTrait,
    potentials: &'a [NeuronMembranePotential<CANQ::NeuronValueQuant>],
    model_parameters: &'a [NMP],
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> EnumeratedLinearSetNeuron<'a, CANQ, NMP, NeuronSetIndexTrait> {
    pub fn get_set_index(&self) -> &NeuronSetIndexTrait {
        &self.neuron_set_index
    }

    pub fn neuron_ref(&self) -> NeuronModelSlice<'a, CANQ, NMP> {
        NeuronModelSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }
}

pub struct EnumeratedLinearSetNeuronMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> {
    neuron_set_index: NeuronSetIndexTrait,
    potentials: &'a mut [NeuronMembranePotential<CANQ::NeuronValueQuant>],
    model_parameters: &'a mut [NMP],
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> EnumeratedLinearSetNeuronMut<'a, CANQ, NMP, NeuronSetIndexTrait> {
    pub fn get_set_index(&self) -> &NeuronSetIndexTrait {
        &self.neuron_set_index
    }

    pub fn neuron_ref(&self) -> NeuronModelSlice<'a, CANQ, NMP> {
        NeuronModelSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }

    pub fn neuron_ref_mut(&mut self) -> NeuronModelMutSlice<'a, CANQ, NMP> {
        NeuronModelMutSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }
}

//endregion
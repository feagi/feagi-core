use crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronRef;
use crate::neuron_dynamics::code_definitions::neurons::common_neuron_structs::LinearNeuronIndexCount;
use crate::quantization_level::CorticalAreaNeuronQuantization;


//region Linear
pub struct EnumeratedNeuronLinearReference<'a, CANQ: CorticalAreaNeuronQuantization, NMF: NeuronModelNeuronRef<'a, CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a NMF
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMF: NeuronModelNeuronRef<'a, CANQ>> EnumeratedNeuronLinearReference<'a, CANQ, NMF> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> &'a NMF {
        self.neuron_collection_ref
    }
}

pub struct EnumeratedNeuronLinearReferenceMut<'a, CANQ: CorticalAreaNeuronQuantization, NMF: NeuronModelNeuronRef<'a, CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a mut NMF
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMF: NeuronModelNeuronRef<'a, CANQ>> EnumeratedNeuronLinearReferenceMut<'a, CANQ, NMF> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> &'a NMF {
        self.neuron_collection_ref
    }

    pub fn neuron_ref_mut(&mut self) -> &'a mut NMF {
        self.neuron_collection_ref
    }
}
//endregion
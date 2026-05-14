use crate::neuron::model_specifications::base_specifications::{BaseNeuronCollectionSharedTrait, LinearNeuronIndexCount};
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub struct EnumeratedBaseNeuronReference<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: BaseNeuronCollectionSharedTrait<CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a BNMC
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: BaseNeuronCollectionSharedTrait<CANQ>> EnumeratedBaseNeuronReference<'a, CANQ, BNMC> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref(&self) -> &'a BNMC::SingleNeuronReference {
        self.neuron_collection_ref.try_get_neuron_data_ref(self.linear_neuron_index).unwrap() // Assuming this is correct
    }
}



pub struct EnumeratedBaseNeuronReferenceMut<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: BaseNeuronCollectionSharedTrait<CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a mut BNMC
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: BaseNeuronCollectionSharedTrait<CANQ>> EnumeratedBaseNeuronReference<'a, CANQ, BNMC> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn neuron_ref_mut(&mut self) -> &'a BNMC::SingleNeuronReference {
        self.neuron_collection_ref.try_get_neuron_data_ref_mut(self.linear_neuron_index).unwrap() // Assuming this is correct
    }
}
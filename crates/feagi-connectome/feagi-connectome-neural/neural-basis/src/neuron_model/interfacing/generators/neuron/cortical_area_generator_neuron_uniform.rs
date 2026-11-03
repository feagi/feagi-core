use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::neuron_model::interfacing::generators::neuron::CorticalAreaGeneratorNeuron;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct CorticalAreaGeneratorNeuronUniform<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    neuron_layout: NeuronLayoutOf<NM, FIQ>,

    default_neuron_data_internal: NM::NeuronDataInternal,
    default_neuron_data_scratch: NM::NeuronDataScratch,
}

impl<FIQ, NM> CorticalAreaGeneratorNeuronUniform<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    pub fn new(neuron_layout: NeuronLayoutOf<NM, FIQ>, default_neuron_data_internal: NM::NeuronDataInternal, default_neuron_data_scratch: NM::NeuronDataScratch) -> Self {
        Self { neuron_layout, default_neuron_data_internal, default_neuron_data_scratch }
    }
}

unsafe impl<FIQ, NM> Send for CorticalAreaGeneratorNeuronUniform<FIQ, NM> where FIQ: FeagiIndexQuantization, NM: NeuronModel, {}

impl<FIQ, NM> CorticalAreaGeneratorNeuron<FIQ, NM> for CorticalAreaGeneratorNeuronUniform<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    fn get_neuron_layout(&self) -> &NeuronLayoutOf<NM, FIQ> {
        &self.neuron_layout
    }

    fn generate_neuron_data_at(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, variation: &NM::CorticalFunctionContext) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        (
            self.default_neuron_data_internal.clone(), 
            self.default_neuron_data_scratch.clone()
        )

        // TODO function context
    }
}
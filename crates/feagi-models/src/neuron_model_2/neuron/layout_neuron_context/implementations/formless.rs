use feagi_data::neurons::potentials::neuron::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_model::neuron::layout_neuron_context::layout_neuron_context::LayoutNeuronContext;

pub struct FormlessLayoutNeuronContext<FIQ: FeagiIndexQuantization> {
    pub index: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> LayoutNeuronContext<FIQ> for FormlessLayoutNeuronContext<FIQ> {
    
}
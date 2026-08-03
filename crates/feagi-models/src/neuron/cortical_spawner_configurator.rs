use crate::neuron::model_generated::cortical_layout::CorticalAreaLayoutNested;
use crate::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron::neuron_model_quantization::NeuronModelQuantization;

pub enum NeuronModelCorticalConstructorConfigurator<NMQ: NeuronModelQuantization, NMCD: NeuronModelCorticalData<NMQ>, NMND: NeuronModelNeuronData<NMQ>> {
    Raw
    {
        _p: core::marker::PhantomData<NMQ>,
        cortical_data: NMCD,
        neuron_data: Vec<NMND>,
        neuron_layout: CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>,
    }
}
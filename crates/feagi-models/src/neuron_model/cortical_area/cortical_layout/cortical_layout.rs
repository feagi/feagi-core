use crate::neuron_model::neuron::layout_neuron_context::layout_neuron_context::LayoutNeuronContext;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Defines how the neurons of a cortical area are arranged internally
pub trait CorticalLayout<FIQ: FeagiIndexQuantization> {
    /// What layout context a neuron has
    type CorticalLayoutNeuronContext: LayoutNeuronContext<FIQ>;

    /// How many neurons the cortical area may contain (usually this value but may be less. NEVER MORE)
    fn get_total_number_neurons_possible(&self) -> FIQ::NeuronIndexQuant; // TODO move to count when we get the wrapper update!

    // Any other cortical level layout context data goes here
}

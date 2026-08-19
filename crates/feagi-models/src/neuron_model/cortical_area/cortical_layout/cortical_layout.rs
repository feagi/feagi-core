use feagi_data::neurons::NeuronCorticalLocalIndex;
use crate::neuron_model::neuron::layout_neuron_context::layout_neuron_context::LayoutNeuronContext;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_model::cortical_area::cortical_layout::cortical_layout_enum::CorticalLayoutTypeEnum;

/// Defines how the neurons of a cortical area are arranged internally
pub trait CorticalLayout<FIQ: FeagiIndexQuantization> {
    
    const CORTICAL_LAYOUT: CorticalLayoutTypeEnum;
    
    /// What layout context a neuron has
    type CorticalLayoutNeuronContext: LayoutNeuronContext<FIQ>;

    /// How many neurons the cortical area may contain (usually this value but may be less. NEVER MORE)
    fn get_total_number_neurons_possible(&self) -> FIQ::NeuronIndexQuant; // TODO move to count when we get the wrapper update!
    
    fn get_neuron_layout_context_from_linear(&self, neuron_index: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>) -> Self::CorticalLayoutNeuronContext;

    // Any other cortical level layout context data goes here
}

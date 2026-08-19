use crate::neuron_model::cortical_area::cortical_layout::cortical_layout::CorticalLayout;
use crate::neuron_model::neuron::layout_neuron_context::implementations::formless::FormlessLayoutNeuronContext;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Defines the dimensions of a cortical area 
pub struct FormlessLayout<FIQ: FeagiIndexQuantization> {
    pub neuron_count: FIQ::NeuronIndexQuant,
}

impl<FIQ: FeagiIndexQuantization> CorticalLayout<FIQ> for FormlessLayout<FIQ>
{
    type CorticalLayoutNeuronContext = FormlessLayoutNeuronContext<FIQ>;

    fn get_total_number_neurons_possible(&self) -> FIQ::NeuronIndexQuant {
        self.neuron_count
    }
}
use feagi_data::neurons::potentials::neuron::NeuronCorticalLocalIndex;
use crate::neuron_model::cortical_area::cortical_layout::cortical_layout::CorticalLayout;
use crate::neuron_model::neuron::layout_neuron_context::implementations::formless::FormlessLayoutNeuronContext;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_model::cortical_area::cortical_layout::cortical_layout_enum::CorticalLayoutTypeEnum;

/// Defines the dimensions of a cortical area 
pub struct FormlessLayout<FIQ: FeagiIndexQuantization> {
    pub neuron_count: FIQ::NeuronIndexQuant,
}

impl<FIQ: FeagiIndexQuantization> CorticalLayout<FIQ> for FormlessLayout<FIQ>
{
    const CORTICAL_LAYOUT: CorticalLayoutTypeEnum = CorticalLayoutTypeEnum::Formless;
    
    type CorticalLayoutNeuronContext = FormlessLayoutNeuronContext<FIQ>;

    fn get_total_number_neurons_possible(&self) -> FIQ::NeuronIndexQuant {
        self.neuron_count
    }

    fn get_neuron_layout_context_from_linear(&self, neuron_index: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>) -> Self::CorticalLayoutNeuronContext {
        FormlessLayoutNeuronContext {
            index: neuron_index,
        }
    }
}
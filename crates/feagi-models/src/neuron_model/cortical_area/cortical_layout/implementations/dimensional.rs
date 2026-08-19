use crate::neuron_model::cortical_area::cortical_layout::cortical_layout::CorticalLayout;
use crate::neuron_model::neuron::layout_neuron_context::implementations::dimensional::DimensionalLayoutNeuronContext;
use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Defines the dimensions of a cortical area
pub struct DimensionalLayout<FIQ: FeagiIndexQuantization> {
    pub dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant>,
}


impl<FIQ: FeagiIndexQuantization> CorticalLayout<FIQ> for DimensionalLayout<FIQ>
{
    type CorticalLayoutNeuronContext = DimensionalLayoutNeuronContext<FIQ>;

    fn get_total_number_neurons_possible(&self) -> FIQ::NeuronIndexQuant {
        self.get_total_number_neurons_possible()
    }
}
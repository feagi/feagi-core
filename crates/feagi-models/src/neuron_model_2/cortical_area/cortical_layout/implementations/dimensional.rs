use crate::neuron_model::cortical_area::cortical_layout::cortical_layout::CorticalLayout;
use crate::neuron_model::neuron::layout_neuron_context::implementations::dimensional::DimensionalLayoutNeuronContext;
use feagi_data::neurons::potentials::neuron::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_model::cortical_area::cortical_layout::cortical_layout_enum::CorticalLayoutTypeEnum;

/// Defines the dimensions of a cortical area
pub struct DimensionalLayout<FIQ: FeagiIndexQuantization> {
    pub dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant>,
}


impl<FIQ: FeagiIndexQuantization> CorticalLayout<FIQ> for DimensionalLayout<FIQ>
{
    const CORTICAL_LAYOUT: CorticalLayoutTypeEnum = CorticalLayoutTypeEnum::Dimensional; 
    
    type CorticalLayoutNeuronContext = DimensionalLayoutNeuronContext<FIQ>;

    fn get_total_number_neurons_possible(&self) -> FIQ::NeuronIndexQuant {
        self.get_total_number_neurons_possible()
    }

    fn get_neuron_layout_context_from_linear(&self, neuron_index: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>) -> Self::CorticalLayoutNeuronContext {
        DimensionalLayoutNeuronContext {
            coordinate: self.dimensions.linear_index_to_coordinate_unchecked(neuron_index)
        }
    }
}
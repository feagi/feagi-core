use feagi_data::neurons::DimensionalCorticalArea4DCoordinate;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_model::neuron::layout_neuron_context::layout_neuron_context::LayoutNeuronContext;

/// Defines the coordinate of a neuron in the dimensional layout
pub struct DimensionalLayoutNeuronContext<FIQ: FeagiIndexQuantization> {
    pub coordinate: DimensionalCorticalArea4DCoordinate<FIQ::NeuronIndexQuant>
}

impl<FIQ: FeagiIndexQuantization> LayoutNeuronContext<FIQ> for DimensionalLayoutNeuronContext<FIQ> {

}
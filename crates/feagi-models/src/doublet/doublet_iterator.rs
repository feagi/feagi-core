use crate::neuron::cortical_area_layout::CorticalAreaLayout;
use feagi_data::neurons::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;



/// Used to map source and destination neurons for a cortical mapping entry.
pub trait DoubletIterator<FIQ, SourceLayout, DestinationLayout>:
    Iterator<
        Item = (
            NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
            NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        ),
    >
where
    FIQ: FeagiIndexQuantization,
    SourceLayout: CorticalAreaLayout<FIQ>,
    DestinationLayout: CorticalAreaLayout<FIQ>,
{
    /// How many synapses will need to be made with the given cortical pairings. This is the
    /// total for the pairing, and does not shrink as the iterator is consumed. Use
    /// [`Iterator::size_hint`] for the number of pairs still to come.
    fn get_number_of_synapses(&self) -> FIQ::NeuronIndexQuant;

}

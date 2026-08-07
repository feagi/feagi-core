
use crate::cortical_area::components::cortical_area_layout::CorticalAreaLayout;
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
+ ExactSizeIterator
+ Clone
+ PartialEq
+ core::hash::Hash
where
    FIQ: FeagiIndexQuantization,
    SourceLayout: CorticalAreaLayout<FIQ>,
    DestinationLayout: CorticalAreaLayout<FIQ>,
{
    /// since we can chain simple doublet iterators to create more complex ones, we may make some
    /// simple doublets for the purpose of being building blocks, but they themselves may not
    /// be useful for genome developers. Set this flag to true to instruct UIs to not
    /// show this as a UI option.
    const ALWAYS_HIDDEN: bool = false;
    
    /// If true, the doublet can be recomputed to remap synapses if either cortical area resizes.
    /// If false, this would mean this doublet would need to be replaced with another for the
    /// given cortical mapping entry if a cortical area is being resized. This should be true
    /// in most cases as having it be false is restrictive!
    const CAN_BE_RECOMPUTED_FOR_CORTICAL_RESIZING: bool;

    /// How many synapses will need to be made with the given cortical pairings. This is the
    /// total for the pairing, and does not shrink as the iterator is consumed. Use
    /// [`Iterator::size_hint`] for the number of pairs still to come.
    fn get_number_of_synapses(&self) -> FIQ::NeuronIndexQuant;

    // TODO function that recomputes, IE takes in another Self to replace itself, but returns also
    // the previous number of doublets that this doublet iterator had
}

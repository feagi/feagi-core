use crate::neuron_model::cortical_area::cortical_area_layout::cortical_area_layout::CorticalAreaLayout;
use crate::cortical_mapping_entry::components::doublet::doublet_iterator::DoubletIterator;
use core::marker::PhantomData;
use feagi_data::neurons::neuron::neuron::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedUnsignedIntegerTrait;

/// Effectively reads a list of existing neuron index pairs into memory. Cannot be modified
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DoubleIteratorManualDefinition<FIQ, SourceLayout, DestinationLayout>
where
    FIQ: FeagiIndexQuantization,
    SourceLayout: CorticalAreaLayout<FIQ>,
    DestinationLayout: CorticalAreaLayout<FIQ>,
{
    stored_pairs: Vec<(
        NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    )>,
    cursor: usize,
    _p: PhantomData<(SourceLayout, DestinationLayout)>,
}

impl<FIQ, SourceLayout, DestinationLayout> DoubleIteratorManualDefinition<FIQ, SourceLayout, DestinationLayout>
where
    FIQ: FeagiIndexQuantization,
    SourceLayout: CorticalAreaLayout<FIQ>,
    DestinationLayout: CorticalAreaLayout<FIQ>,
{
    /// Loads in the given vector of pairs, first filtering out all pairs of neurons that do not fit
    pub fn new(
        mut pairs_to_validate: Vec<(
            NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
            NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        )>,
        source_layout: &SourceLayout,
        destination_layout: &DestinationLayout,
    ) -> DoubleIteratorManualDefinition<FIQ, SourceLayout, DestinationLayout> {
        // TODO swap for result, error if the number of neurons is greater that quantization allows!

        let mut i = 0;
        while i < pairs_to_validate.len() {
            let pair = pairs_to_validate[i];
            if source_layout.contains_given_neuron_index(pair.0) && destination_layout.contains_given_neuron_index(pair.1) {
                i += 1;
            } else {
                pairs_to_validate.swap_remove(i);
            }
        }

        Self {
            stored_pairs: pairs_to_validate,
            cursor: 0,
            _p: Default::default(),
        }
    }
}

impl<FIQ, SourceLayout, DestinationLayout> Iterator for DoubleIteratorManualDefinition<FIQ, SourceLayout, DestinationLayout>
where
    FIQ: FeagiIndexQuantization,
    SourceLayout: CorticalAreaLayout<FIQ>,
    DestinationLayout: CorticalAreaLayout<FIQ>,
{
    type Item = (
        NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.stored_pairs.len() {
            return None;
        }
        let pair = self.stored_pairs[self.cursor];
        self.cursor += 1;
        Some(pair)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.stored_pairs.len() - self.cursor;
        (remaining, Some(remaining))
    }
}

impl<FIQ, SourceLayout, DestinationLayout> ExactSizeIterator for DoubleIteratorManualDefinition<FIQ, SourceLayout, DestinationLayout>
where
    FIQ: FeagiIndexQuantization,
    SourceLayout: CorticalAreaLayout<FIQ>,
    DestinationLayout: CorticalAreaLayout<FIQ>,
{
}

impl<FIQ, SourceLayout, DestinationLayout> DoubletIterator<FIQ, SourceLayout, DestinationLayout>
    for DoubleIteratorManualDefinition<FIQ, SourceLayout, DestinationLayout>
where
    FIQ: FeagiIndexQuantization,
    SourceLayout: CorticalAreaLayout<FIQ>,
    DestinationLayout: CorticalAreaLayout<FIQ>,
{
    /// Since this is a static collection of pairs, we have no means of recomputing!
    const CAN_BE_RECOMPUTED_FOR_CORTICAL_RESIZING: bool = false;

    fn get_number_of_synapses(&self) -> FIQ::SynapseIndexCountQuant {
        // We verified length beforehand
        FIQ::SynapseIndexCountQuant::quant_from_usize_unchecked(self.stored_pairs.len())
    }
}

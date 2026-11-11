use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutTag};
use crate::neuron_model::neuron_pattern::NeuronPattern;
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

// TODO this may not be correct for all layouts?

/// Iterates over all neurons of a cortical area's layout
pub struct NeuronPatternAll<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    layout: <NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>
}

impl<FIQ, NM> NeuronPatternAll<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    pub fn new(layout: <NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>) -> Self {
        Self { layout }
    }
}

impl<FIQ, NM> NeuronPattern<FIQ, NM> for NeuronPatternAll<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    fn get_contained_neuron_layout(&self) -> &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ> {
        &self.layout
    }

    fn iterate_neurons_indexes_to_edit(&self) -> impl Iterator<Item=CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>> {
        let number_neurons = self.layout.get_number_neurons_to_allocate_for().quant_to_usize();
        let range = 0..number_neurons;
        range.map(move |i | {
            let index= &CorticalAreaNeuronLocalIndex::<FIQ::NeuronIndexQuant>::quant_from_usize_unchecked(i);
            *index
        })
    }
}


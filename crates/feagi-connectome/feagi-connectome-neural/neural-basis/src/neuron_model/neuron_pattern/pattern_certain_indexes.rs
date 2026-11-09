use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::neuron_model::neuron_layout::NeuronLayoutTag;
use crate::neuron_model::neuron_pattern::{NeuronPattern, NeuronPatternError};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

pub struct NeuronPatternCertainIndexes<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    layout: <NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
    indexes: Vec<CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>>
}

impl<FIQ, NM> NeuronPatternCertainIndexes<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    pub fn new(
        layout: <NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        indexes: Vec<CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>>) -> Result<Self, NeuronPatternError> {
        // TODO check first that no index is out of range!
        Ok(Self { layout, indexes })
    }
}

impl<FIQ, NM> NeuronPattern<FIQ, NM> for NeuronPatternCertainIndexes<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    fn get_contained_neuron_layout(&self) -> &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ> {
        &self.layout
    }

    fn iterate_neurons_indexes_to_edit(&self) -> impl Iterator<Item=CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>> {
        self.indexes.iter().copied()
    }
}
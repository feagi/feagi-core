use crate::neuron_model::interfacing::generators::neuron::CorticalAreaGeneratorNeuron;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::quant_indexed::quant_indexed::{QuantContiguousStoreBackend, QuantIndexedBackend};
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;

pub struct CorticalAreaGeneratorNeuronFromVector<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    neuron_layout: NeuronLayoutOf<NM, FIQ>,

    neuron_data_internal: Vec<NM::NeuronDataInternal>,
    neuron_data_scratch: Vec<NM::NeuronDataScratch>,
}

impl<FIQ, NM> CorticalAreaGeneratorNeuronFromVector<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    pub fn new( // TODO length checks with layout!
        neuron_layout: NeuronLayoutOf<NM, FIQ>,
        neuron_data_internal: Vec<NM::NeuronDataInternal>,
        neuron_data_scratch: Vec<NM::NeuronDataScratch>,
    ) -> Self {
        Self {
            neuron_layout,
            neuron_data_internal,
            neuron_data_scratch,
        }
    }
}

impl<FIQ, NM> CorticalAreaGeneratorNeuron<FIQ, NM> for CorticalAreaGeneratorNeuronFromVector<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    // TODO Error Checking on Collection Length

    fn get_neuron_layout(&self) -> &NeuronLayoutOf<NM, FIQ> {
        &self.neuron_layout
    }

    fn generate_neuron_data_at(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        variation: &NM::CorticalFunctionContext,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        (
            self.neuron_data_internal[index.quant_to_usize()].clone(),
            self.neuron_data_scratch[index.quant_to_usize()].clone(),
        )

        // TODO function context
    }

    // TODO overwrite default trait implementations for slices, we can just move the whole slice over
}

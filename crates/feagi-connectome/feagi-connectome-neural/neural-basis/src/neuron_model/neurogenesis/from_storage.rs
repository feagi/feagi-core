use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neurogenesis::{NeurogenesisDefaults, NeurogenesisMethod};
use crate::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutTag};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;

#[derive(Clone, Debug)]
pub struct NeurogenesisFromStorage<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    layout: <NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
    cortical_universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    cortical_properties: NM::CorticalDataProperties,
    cortical_internal: NM::CorticalDataInternal,
    cortical_variance: NM::CorticalDataVariance,
    neuron_internal: Vec<NM::NeuronDataInternal>, // TODO dont use vec!
}

impl<FIQ, NM> NeurogenesisFromStorage<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    pub fn new(
        layout: <NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        cortical_universal_properties: NeuronModelUniversalCorticalProperties<NM>,
        cortical_properties: NM::CorticalDataProperties,
        cortical_internal: NM::CorticalDataInternal,
        cortical_variance: NM::CorticalDataVariance,
        neuron_internal: Vec<NM::NeuronDataInternal>,
    ) -> Result<Self, NeurogenesisStorageError> {

        if layout.get_number_neurons_to_allocate_for().quant_to_usize() != neuron_internal.len() {
            return Err(NeurogenesisStorageError::InvalidSize)
        }
        Ok(
            Self {
                layout,
                cortical_universal_properties,
                cortical_properties,
                cortical_internal,
                cortical_variance,
                neuron_internal,
            }
        )
    }
}

impl<FIQ, NM> NeurogenesisMethod<FIQ, NM> for NeurogenesisFromStorage<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    fn get_layout(&self) -> &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ> {
        &self.layout
    }

    fn get_universal_cortical_properties(&self) -> NeuronModelUniversalCorticalProperties<NM> {
        self.cortical_universal_properties
    }

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        (self.cortical_properties, self.cortical_internal)
    }

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        &self.cortical_variance
    }

    fn spawn_neuron_data_at(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        let index_u = index.quant_to_usize();
        let internal = self.neuron_internal[index_u].clone();
        let scratch = NM::NeurogenesisDefaults::static_get_neuron_scratch(index, &self.layout, &internal);
        (internal, scratch)
    }
}

#[derive(Clone, Copy, Hash, Debug, thiserror::Error)]
pub enum NeurogenesisStorageError {
    #[error("Given collection size of NeurogenesisFromStorage does not meet size defined by layout!")]
    InvalidSize
}
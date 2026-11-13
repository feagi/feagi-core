use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayout, NeuronLayoutOf};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;



/// The saved neuron data of a cortical area, spawning it as it was. Taken from a running area or a
/// file, never from defaults. The size is checked during validation
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
// Every field is serializable through the traits of FIQ and NM, not the bounds serde infers
#[serde(bound = "")]
pub struct NeurogenesisByStorage<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    cortical_layout_context: CorticalLayoutContextOf<FIQ, NM>,
    cortical_universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    cortical_properties: NM::CorticalDataProperties,
    cortical_internal: NM::CorticalDataInternal,
    cortical_variance: NM::CorticalDataVariance,
    neuron_internal: Vec<NM::NeuronDataInternal>, // TODO dont use vec!
}

impl<FIQ, NM> NeurogenesisByStorage<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    pub fn new(
        cortical_layout_context: CorticalLayoutContextOf<FIQ, NM>,
        cortical_universal_properties: NeuronModelUniversalCorticalProperties<NM>,
        cortical_properties: NM::CorticalDataProperties,
        cortical_internal: NM::CorticalDataInternal,
        cortical_variance: NM::CorticalDataVariance,
        neuron_internal: Vec<NM::NeuronDataInternal>,
    ) -> Self {
        Self {
            cortical_layout_context,
            cortical_universal_properties,
            cortical_properties,
            cortical_internal,
            cortical_variance,
            neuron_internal,
        }
    }
}

impl<FIQ, NM> NeurogenesisMethod<FIQ, NM> for NeurogenesisByStorage<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    fn get_cortical_layout_context(&self) -> &CorticalLayoutContextOf<FIQ, NM> {
        &self.cortical_layout_context
    }

    fn get_universal_cortical_properties(&self) -> NeuronModelUniversalCorticalProperties<NM> {
        self.cortical_universal_properties
    }

    fn get_model_cortical_properties(&self) -> &NM::CorticalDataProperties {
        &self.cortical_properties
    }

    fn get_model_cortical_internal(&self) -> &NM::CorticalDataInternal {
        &self.cortical_internal
    }

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        &self.cortical_variance
    }

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        (self.cortical_properties, self.cortical_internal)
    }

    fn spawn_neuron_data_at(
        &self,
        layout: &NeuronLayoutOf<FIQ, NM>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        let internal = self.neuron_internal[index.quant_to_usize()].clone();
        let scratch = NM::generate_neuron_scratch(index, layout, &internal);
        (internal, scratch)
    }
}



use crate::neuron_model::interfacing::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::NeuronModel;

/// For editing the genome properties in place, the most common type of editing
pub struct CorticalAreaEditorCorticalGenomeProperties<NM>
where
    NM: NeuronModel,
{
    updated_universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    updated_cortical_data_properties: NM::CorticalDataProperties,
}

impl<NM> CorticalAreaEditorCorticalGenomeProperties<NM>
where
    NM: NeuronModel,
{
    pub fn new(
        updated_universal_properties: NeuronModelUniversalCorticalProperties<NM>,
        updated_cortical_data_properties: NM::CorticalDataProperties,
    ) -> Self {
        Self {
            updated_universal_properties,
            updated_cortical_data_properties,
        }
    }

}

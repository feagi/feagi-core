use crate::neuron_model::interfacing::generators::cortical::CorticalAreaGeneratorCortical;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use crate::neuron_model::interfacing::shared::NeuronModelUniversalCorticalProperties;

pub struct CorticalAreaGeneratorCorticalFromStored<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    neuron_layout: NeuronLayoutOf<NM, FIQ>,

    universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    
    default_cortical_data_properties: NM::CorticalDataProperties,
    default_cortical_data_internal: NM::CorticalDataInternal,
    default_cortical_data_scratch: NM::CorticalDataScratch,
}

impl<FIQ, NM> CorticalAreaGeneratorCorticalFromStored<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    
    /*
    pub fn new_from_default(default: CorticalDef) {
        
    }
     */
    
    pub fn new_manual(
        neuron_layout: NeuronLayoutOf<NM, FIQ>,
        universal_properties: NeuronModelUniversalCorticalProperties<NM>,
        default_cortical_data_properties: NM::CorticalDataProperties,
        default_cortical_data_internal: NM::CorticalDataInternal,
        default_cortical_data_scratch: NM::CorticalDataScratch,
    ) -> Self {
        Self {
            neuron_layout,
            universal_properties,
            default_cortical_data_properties,
            default_cortical_data_internal,
            default_cortical_data_scratch,
        }
    }
}

impl<FIQ, NM> CorticalAreaGeneratorCortical<FIQ, NM> for CorticalAreaGeneratorCorticalFromStored<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    fn get_neuron_layout(&self) -> &NeuronLayoutOf<NM, FIQ> {
        &self.neuron_layout
    }

    fn get_cortical_universal_properties(&self) -> &NeuronModelUniversalCorticalProperties<NM> {
        &self.universal_properties
    }


    fn get_cortical_level_data(&self) -> (&NM::CorticalDataProperties, &NM::CorticalDataInternal, &NM::CorticalDataScratch) {
        (
            &self.default_cortical_data_properties,
            &self.default_cortical_data_internal,
            &self.default_cortical_data_scratch,
        )
    }
}

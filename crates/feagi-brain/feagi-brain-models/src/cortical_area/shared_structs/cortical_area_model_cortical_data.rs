use feagi_basis::feagi_neuron::wrapped_types::CorticalAreaNeuronPotential;
use crate::cortical_area::implemented_components::cortical_area_quantization::CorticalAreaQuantization;
use crate::cortical_area::implemented_components::cortical_model_data_field::CorticalModelDataField;
use crate::cortical_area::shared_structs::inverse_outgoing_connection_count::InverseOutgoingConnectionCount;

pub struct CorticalAreaModelCorticalData<CAQ, CorticalDataProperties, CorticalDataInternal, CorticalDataScratch>
where
    CAQ: CorticalAreaQuantization,
    CorticalDataProperties: CorticalModelDataField<CAQ>,
    CorticalDataInternal: CorticalModelDataField<CAQ>,
    CorticalDataScratch: CorticalModelDataField<CAQ>,

{
    /// The cortical level data that should be exposed to genome developers. Not mutable during
    /// cortical processing
    pub cortical_data_properties: CorticalDataProperties,
    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    pub cortical_data_internal: CorticalDataInternal,
    /// The cortical level data that is used for runtime processing, is not exposed to
    /// genome developers nor is it saved (starts clean with every init)
    pub cortical_data_scratch: CorticalDataScratch,
    /// If PSP is cortical driven, use this value
    pub cortical_driven_psp: CorticalAreaNeuronPotential<CAQ::MembranePotentialQuant>,
    /// The 1 / float count of outgoing synapses
    pub inverse_outgoing_connection_count: InverseOutgoingConnectionCount<CAQ::MembranePotentialQuant>
}

impl<CAQ, CorticalDataProperties, CorticalDataInternal, CorticalDataScratch>
CorticalAreaModelCorticalData<CAQ, CorticalDataProperties, CorticalDataInternal, CorticalDataScratch>
where
    CAQ: CorticalAreaQuantization,
    CorticalDataProperties: CorticalModelDataField<CAQ>,
    CorticalDataInternal: CorticalModelDataField<CAQ>,
    CorticalDataScratch: CorticalModelDataField<CAQ>,
{
    pub fn get_parameters_for_cortical_dynamics<'a>(&mut self) -> (&'a CorticalDataProperties, &'a mut CorticalDataInternal, &'a mut CorticalDataScratch)
    {
        (self.cortical_data_properties, self.cortical_data_internal, self.cortical_data_scratch)
    }

    pub fn get_parameters_for_neuron_dynamics<'a>(&self) -> (&'a CorticalDataProperties, &'a CorticalDataInternal, &'a CorticalDataScratch)
    {
        (self.cortical_data_properties, self.cortical_data_internal, self.cortical_data_scratch)
    }
}

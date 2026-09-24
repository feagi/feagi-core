use feagi_basis::feagi_neuron::single_area_collections::neurons::CorticalAreaNeuronPotential;
use crate::cortical_area::data_structs::cortical_area_flags::CorticalAreaFlags;
use crate::cortical_area::extendable_components::cortical_area_quantization::CorticalAreaQuantization;
use crate::cortical_area::extendable_components::cortical_model_data_field::CorticalModelDataField;

pub struct CorticalAreaModelConnectomeCorticalData<CAQ, CorticalDataProperties, CorticalDataInternal, CorticalDataScratch>
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
    /// If PSP is cortical area driven (not membrane potential driven), use this value
    pub cortical_level_psp: CorticalAreaNeuronPotential<CAQ::MembranePotentialQuant>,
    /// Boolean flags used by all cortical areas
    pub cortical_area_flags: CorticalAreaFlags,
}

impl<CAQ, CorticalDataProperties, CorticalDataInternal, CorticalDataScratch>
CorticalAreaModelConnectomeCorticalData<CAQ, CorticalDataProperties, CorticalDataInternal, CorticalDataScratch>
where
    CAQ: CorticalAreaQuantization,
    CorticalDataProperties: CorticalModelDataField<CAQ>,
    CorticalDataInternal: CorticalModelDataField<CAQ>,
    CorticalDataScratch: CorticalModelDataField<CAQ>,
{
    /// Get the parameters needed for cortical dynamics. Inner / scratch states are mutable, but
    /// properties is not mutable since that is intended to be set by the genome developer
    /// externally only!
    pub fn get_parameters_for_cortical_dynamics(&mut self) -> (&CorticalDataProperties, &mut CorticalDataInternal, &mut CorticalDataScratch)
    {
        (&self.cortical_data_properties, &mut self.cortical_data_internal, &mut self.cortical_data_scratch)
    }

    /// Get parameters used for neuron dynamics. Since many neurons read this state in parallel, the
    /// given data must be an immut ref!
    pub fn get_parameters_for_neuron_dynamics(&self) -> (&CorticalDataProperties, &CorticalDataInternal, &CorticalDataScratch)
    {
        ( &self.cortical_data_properties, &self.cortical_data_internal, &self.cortical_data_scratch)
    }
}

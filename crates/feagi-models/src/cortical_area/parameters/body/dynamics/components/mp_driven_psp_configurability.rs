use feagi_data::neurons::wrapped_types::CorticalNeuronPotential;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

/// A marker trait that defines if the PSP can be configured to be / not be on the membrane
/// potentials. Note that in the NPU, this state is stored in the neuron flags. This is just
/// for easy export reasons
pub trait MPDrivenPSPConfigurability {
    fn get_if_psp_is_mp_driven(&self) -> bool;
}

/// For if the cortical area can 
pub trait MPDrivenPSPPossible<MP: MembranePotentialQuantization>: MPDrivenPSPConfigurability  {
    fn get_cortical_driven_psp_value(&self) -> CorticalNeuronPotential<MP::MembranePotentialQuant>;
}


/// MP Driven PSP is forced on, cannot be set to cortical level at all
pub struct MPDrivenPSPForcedOn;
impl MPDrivenPSPConfigurability for MPDrivenPSPForcedOn {
    fn get_if_psp_is_mp_driven(&self) -> bool {
        true
    }
}

pub struct MPDrivenPSPForcedOff<MP: MembranePotentialQuantization> {
    pub cortical_psp: CorticalNeuronPotential<MP::MembranePotentialQuant>
}

impl<MP: MembranePotentialQuantization> MPDrivenPSPConfigurability for MPDrivenPSPForcedOff<MP> {
    fn get_if_psp_is_mp_driven(&self) -> bool {
        false
    }
}

impl<MP: MembranePotentialQuantization> MPDrivenPSPPossible<MP> for MPDrivenPSPForcedOff<MP> {
    fn get_cortical_driven_psp_value(&self) -> CorticalNeuronPotential<MP::MembranePotentialQuant>
    {
        self.cortical_psp
    }
}


pub struct MPDrivenPSPConfigurable<MP: MembranePotentialQuantization> {
    pub cortical_psp: CorticalNeuronPotential<MP::MembranePotentialQuant>,
    pub psp_is_mp_driven: bool // Not actually a data point in cortical area, its per all neurons uniformly
}

impl<MP: MembranePotentialQuantization> MPDrivenPSPConfigurability for MPDrivenPSPConfigurable<MP> {
    fn get_if_psp_is_mp_driven(&self) -> bool {
        self.psp_is_mp_driven
    }
}

impl<MP: MembranePotentialQuantization> MPDrivenPSPPossible<MP> for MPDrivenPSPConfigurable<MP> {
    fn get_cortical_driven_psp_value(&self) -> CorticalNeuronPotential<MP::MembranePotentialQuant>
    {
        self.cortical_psp
    }
}
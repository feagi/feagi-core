use crate::models::cortical_area::components::cortical_area_dynamics::components::post_synaptic_potential::MPDrivenPSPConfigurability;

/// MP Driven PSP is forced on, cannot be set to cortical level at all
pub struct MPDrivenPSPForcedOn;

impl PostSynapticPotential for MPDrivenPSPForcedOn {
    fn get_if_psp_is_mp_driven(&self) -> bool {
        true
    }
}
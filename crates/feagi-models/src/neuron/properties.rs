use feagi_data::quantization_levels::membrane_potential_quantization::{CorticalMembranePotentialQuantizationGenomic, MembranePotentialQuantization};

type GenomeMPQuant = <CorticalMembranePotentialQuantizationGenomic as MembranePotentialQuantization>::MembranePotentialQuant;

/// Cortical level properties that all cortical areas have
#[derive(Clone, Copy, Debug)]
pub struct CorticalAreaProperties {
    pub non_mp_psp: GenomeMPQuant,
    pub probe_cortical_area_input_disabled: bool,
    pub probe_cortical_area_output_disabled: bool,
    pub is_psp_uniform: bool,
    pub is_psp_mp_driven: bool,
}

/// Per neuron properties that all neurons have that can be configured
#[derive(Clone, Copy, Debug)]
pub struct NeuronProperties {
    pub probe_force_disabled: bool,
    pub probe_force_firing: bool,
}

impl Default for NeuronProperties {
    fn default() -> Self {
        NeuronProperties {
            probe_force_disabled: false,
            probe_force_firing: false,
        }
    }
}
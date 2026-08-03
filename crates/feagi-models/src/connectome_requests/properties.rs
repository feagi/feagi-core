//! Defines any universal properties using the universal quantization state

/// Cortical level properties of all cortical areas that can be configured
pub struct UniversalCorticalAreaProperties {
    pub non_mp_psp: f64, // will have to be requantized to mp
    pub probe_cortical_area_input_disabled: bool,
    pub probe_cortical_area_output_disabled: bool,
    pub is_psp_uniform: bool,
    pub is_psp_mp_driven: bool,
}

/// Per neuron properties of all neurons that can be configured
pub struct UniversalNeuronProperties {
    pub probe_force_disabled: bool,
    pub probe_force_firing: bool,
}
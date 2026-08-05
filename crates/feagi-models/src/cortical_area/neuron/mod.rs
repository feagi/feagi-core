pub mod neuron_model;

pub mod neuron_history;

use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

/// Per neuron properties that all neurons have that can be configured
#[derive(Clone, Copy, Debug)]
pub struct Neuron {
    pub probe_force_disabled: bool,
    pub probe_force_firing: bool,
}



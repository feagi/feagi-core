//! The `NeuronModel` family, laid out as its model family template expects: the base trait and
//! every generated item of the family, with a sub module per model holding the model trait and
//! the marker struct of each of its quantization levels

pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::NeuronModel;
pub use super::family::*;

/// The `FoundationPower` model
pub mod foundation_power {
    pub use crate::neuron_models::foundation_power::data::*;
}

/// The `FeagiAdvanced` model
pub mod feagi_advanced {
    pub use crate::neuron_models::feagi_advanced::data::*;
}

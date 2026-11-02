

pub mod neuron_models;

pub use feagi_connectome_neural::neural_basis::quantization_levels;

pub mod cortical_area {
    // TODO check whats needed
    pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout;
    pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::{NeuronLayoutType, NeuronModel};
    pub use feagi_connectome_neural::neural_basis::neuron_model::interfacing::generators::{
        CorticalAreaGenerator, CorticalAreaGeneratorFromDefaults, CorticalModelDefaultsSpawnData,
    };
    pub use feagi_connectome_neural::neural_basis::neuron_model::interfacing::shared::NeuronModelUniversalCorticalProperties;
}

/// Macros for defining families of models, and families of cortical area types using them
pub mod family_macros {
    pub use feagi_connectome_proc::{
        define_cortical_type_family_template, define_model_family_template, expand_cortical_type_family, expand_model_family,
    };
}

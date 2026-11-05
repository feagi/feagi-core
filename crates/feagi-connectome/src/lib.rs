

pub mod neuron_models;
pub mod neural_processors;

pub use feagi_connectome_neural::neural_basis::quantization_levels;

pub use feagi_connectome_neural::neural_basis::visualization;

pub mod cortical_area {
    // TODO check whats needed
    pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout;
    pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutTag;
    pub use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
    pub use feagi_connectome_neural::neural_basis::neuron_model::CorticalAreaEngineIndex;
    pub use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_spec::{
        CorticalAreaSpec, CorticalAreaSpecError, NeuronInit, NeuronInitializer, NoCustomNeuronInit,
    };
}

/// Macros for defining families of models, and families of cortical area types using them
pub mod family_macros {
    pub use feagi_connectome_proc::{
        define_cortical_type_family_template, define_model_family_template, expand_cortical_area_map, expand_cortical_identifier, expand_cortical_type_family,
        expand_model_family,
    };
}


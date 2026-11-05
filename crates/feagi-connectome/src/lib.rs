

pub mod neuron_models;
pub mod neural_processors;

/// Core data structures and traits needed for interfacing with the connectome
pub mod connectome_basis {
    pub use feagi_connectome_neural::neural_basis::quantization_levels;
    pub use feagi_connectome_neural::neural_basis::visualization;

    pub mod neuron_model {
        pub use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
        pub use feagi_connectome_neural::neural_basis::neuron_model::wrapped_indexing;
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutTag;
        pub use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_spec::{
            CorticalAreaSpec, CorticalAreaSpecError, NeuronInit, NeuronInitializer, NoCustomNeuronInit,
        };
        pub use feagi_connectome_proc::{
            define_cortical_type_family_template, define_model_family_template, expand_cortical_area_map, expand_cortical_identifier, expand_cortical_type_family,
            expand_model_family,
        };
    }
    
    pub mod synapse_model {
        
    }
}

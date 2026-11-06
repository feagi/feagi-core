

mod neuron_models;
pub mod neural_processors;

/// Core data structures and traits needed for interfacing with the connectome
pub mod connectome_basis {
    pub use feagi_connectome_neural::neural_basis::quantization_levels;
    pub use feagi_connectome_neural::neural_basis::visualization;

    pub mod engine {
        pub use feagi_connectome_neural::neural_basis::engine::BurstEngineIndex;
    }

    pub mod neuron_model {
        pub use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
        pub use feagi_connectome_neural::neural_basis::neuron_model::NeuronModelTag;
        /// The generated `NeuronModel` family, which the cortical type family macros reach as `neuron_model`
        pub use crate::neuron_models::neuron_model::*;
        pub use crate::neuron_models::init_dispatch::{CorticalAreaInitContainer, CorticalAreaInitEnum};
        pub use feagi_connectome_neural::neural_basis::neuron_model::wrapped_indexing;
        pub use feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutTag};
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutTagVoxel, NeuronLayoutVoxel};
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::single_stepped_dynamic::{
            NeuronLayoutSingleSteppedDynamic, NeuronLayoutTagSingleSteppedDynamic,
        };
        pub use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_init::{
            CorticalAreaInit, CorticalAreaInitError, NeuronInit, NeuronInitializer, NoCustomNeuronInit,
        };
        pub use feagi_connectome_proc::{
            define_cortical_type_family_template, define_model_family_template, expand_cortical_area_map, expand_cortical_identifier, expand_cortical_type_family,
            expand_model_family,
        };
    }

    pub mod synapse_model {
    }
}

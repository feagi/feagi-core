

mod neuron_models;
pub mod neural_processors;

/// Core data structures and traits needed for interfacing with the connectome
pub mod connectome_basis {
    pub use feagi_connectome_neural::neural_basis::quantization_levels;
    pub use feagi_connectome_neural::neural_basis::visualization;

    pub mod engine {
        pub use feagi_connectome_neural::neural_basis::engine::BurstEngineIndex;
        pub use feagi_connectome_neural::neural_basis::engine::engine_change_command::EngineChangeCommandOtherError;
        /// The burst engine running on the CPU, multithreaded through rayon
        #[cfg(feature = "burst-engine-cpu")]
        pub use feagi_connectome_neural::burst_engine_cpu::BurstEngineCPURayon;
        #[cfg(feature = "burst-engine-cpu")]
        pub use feagi_connectome_neural::burst_engine_cpu::BurstEngineCPURayonInitError;
    }

    /// The edit steps sent to the connectome, and what each returns
    #[cfg(feature = "std")]
    pub mod connectome_editing {
        pub use crate::neuron_models::init_dispatch::ConnectomeDispatch;
        pub use feagi_connectome_neural::neural_basis::connectome_editing::{
            ConnectomeEditError, ConnectomeEditErrorCorticalAreaAdd, ConnectomeEditResultantCorticalAreaAdd, ConnectomeEditResultantEnum,
            ConnectomeEditStepCorticalAreaAdd, ConnectomeEditStepEnum,
        };
    }

    pub mod neuron_model {
        pub use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
        pub use feagi_connectome_neural::neural_basis::neuron_model::NeuronModelTag;
        /// The generated `NeuronModel` family, which the cortical type family macros reach as `neuron_model`
        pub use crate::neuron_models::neuron_model::*;
        pub use crate::neuron_models::init_dispatch::{CorticalAreaInitContainer, CorticalAreaInitEnum};
        pub use feagi_connectome_neural::neural_basis::neuron_model::wrapped_indexing;
        pub use feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutOf, NeuronLayoutTag};
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutTagVoxel, NeuronLayoutVoxel};
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::single_stepped_dynamic::{
            NeuronLayoutSingleSteppedDynamic, NeuronLayoutTagSingleSteppedDynamic,
        };
        pub use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_init::{
            CorticalAreaInit, CorticalAreaInitGenerationError, NeuronInitErrorOf, NeuronInitializer,
        };
        /// The neuron initializers commonly listed by neuron models
        pub use crate::neuron_models::initializers::UniformNeuronInit;
        #[cfg(feature = "alloc")]
        pub use crate::neuron_models::initializers::{StoredNeuronInit, StoredNeuronInitError};
        pub use feagi_connectome_proc::{
            define_cortical_type_family_template, define_model_family_template, expand_cortical_area_map, expand_cortical_identifier, expand_cortical_type_family,
            expand_model_family,
        };
    }

    pub mod synapse_model {
    }
}

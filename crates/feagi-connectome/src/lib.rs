

mod neuron_models;
pub mod neural_processors;
pub mod common_systems;

/// Core data structures and traits needed for interfacing with the connectome
pub mod connectome_basis {
    pub use feagi_connectome_neural::neural_basis::quantization_levels;
    #[cfg(feature = "alloc")]
    pub use feagi_connectome_neural::neural_basis::visualization;

    #[cfg(feature = "alloc")]
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
        pub use crate::neuron_models::neurogenesis_dispatch::ConnectomeDispatch;
        pub use feagi_connectome_neural::neural_basis::connectome_editing::{
            ConnectomeEditError, ConnectomeEditErrorCorticalAreaAdd, ConnectomeEditResultantCorticalAreaAdd, ConnectomeEditResultantEnum,
            ConnectomeEditStepCorticalAreaAdd, ConnectomeEditStepEnum,
        };
    }

    pub mod neuron_model {
        pub use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
        /// The `NeuronModel` family, which the cortical type family macros reach as `neuron_model`,
        /// and the dispatch macros `with_nm!` as `$crate::connectome_basis::neuron_model`
        pub use crate::neuron_models::family::*;
        /// A module per neuron model, with its quantization items and `BurstEngineDynamics`
        pub use crate::neuron_models::{};
        pub use crate::neuron_models::neurogenesis_dispatch::{NeurogenesisFromStorageOrByGeneratorContainer, NeurogenesisFromStorageOrByGeneratorEnum, NeurogenesisFromStorageOrByGeneratorOf};
        pub use feagi_connectome_neural::neural_basis::neuron_model::wrapped_indexing;
        pub use feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutOf, NeuronLayoutTag};
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutTagVoxel, NeuronLayoutVoxel};
        pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::single_stepped_dynamic::{
            NeuronLayoutSingleSteppedDynamic, NeuronLayoutTagSingleSteppedDynamic,
        };
        /// How the neurons of a cortical area are spawned
        pub use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::{
            by_generator::NeurogenesisByGenerator, from_storage::{NeurogenesisFromStorage, NeurogenesisStorageError},
            from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator, NeurogenesisMethod, NeurogenesisScratchVarianceGenerator,
        };
        /// The generator starting every neuron of an area from the same values, for cortical types to wrap
        pub use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
        pub use feagi_connectome_proc::{
            define_cortical_type_family_template, define_model_family_template, expand_cortical_area_map, expand_cortical_identifier, expand_cortical_type_family,
            expand_model_family,
        };
        /// A module per neuron model, with its quantization items and `BurstEngineDynamics`
        pub use crate::neuron_models::models::feagi_advanced;
        /// A module per neuron model, with its quantization items and `BurstEngineDynamics`
        pub use crate::neuron_models::models::feagi_simple;
        /// A module per neuron model, with its quantization items and `BurstEngineDynamics`
        pub use crate::neuron_models::models::foundation_death;
        /// A module per neuron model, with its quantization items and `BurstEngineDynamics`
        pub use crate::neuron_models::models::foundation_power;
        /// A module per neuron model, with its quantization items and `BurstEngineDynamics`
        pub use crate::neuron_models::models::memory_fire_hashing;
    }

    pub mod synapse_model {
    }
}

//! The `NeuronModel` model family, of every neuron model at each of its quantization levels. Its
//! items are reached through [`super::neuron_model`]

pub use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::NeuronModel;
use feagi_connectome_proc::{define_model_family_template, expand_model_family};



define_model_family_template! {
    /// Every neuron model, each at any of its quantization levels
    Family: NeuronModel,
    Error: NeuronModelSelectionError,

    Models: {
        /// Always fires
        FoundationPower: {
            Float16,
            BFloat16,
            Float32,
            Float64,
        }
        
        /// Kills the brain
        FoundationDeath: {
            Float16,
            BFloat16,
            Float32,
            Float64,
        }

        /// Feagi Variant of Leaky integrate and fire, suitable for most needs
        FeagiSimple: {
            DefaultQuant,
        }

        /// Feagi Variant of Leaky integrate and fire, with additional advanced parameters at a computational and memory cost
        FeagiAdvanced: {
            DefaultQuant,
        }
        
        /// Gains Memories by the hash of incoming fire patterns
        MemoryFireHashing: {
            DefaultQuant,
        }
        
    }
}

neuron_model_family_template!(expand_model_family);

//! The `CorticalAreaType` cortical type family, of every cortical area type at each member of its
//! neuron model. Its items are reached through [`super::cortical_area_type`]

use super::cortical_area_type;
use feagi_connectome::family_macros::{define_cortical_type_family_template, expand_cortical_type_family};
use feagi_connectome::neuron_models::neuron_model;

define_cortical_type_family_template! {
    /// Every cortical area type, each using a single neuron model
    Family: CorticalAreaType,
    Error: CorticalAreaTypeSelectionError,
    ModelFamily: NeuronModel,
    ModelFamilyError: NeuronModelSelectionError,

    Types: {
        /// Cortical area types with unique properties within the genome
        Foundation: {
            /// Power management processing
            Power: FoundationPower,
        }

        /// Connects cortical areas to each other
        Interconnect: FeagiAdvanced,
    }
}

cortical_area_type_family_template!(expand_cortical_type_family);

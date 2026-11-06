//! Every cortical area type. The template below lists each one by its super type, with the neuron
//! models it allows, while the details of a type live with its super type in the module of that
//! name: its marker struct, [`CorticalAreaType`] implementation, and a `Uses{Model}` implementation
//! per neuron model it allows. Generated items are also reached through `cortical_area_type`

use feagi_connectome::connectome_basis::neuron_model::{
    self, define_cortical_type_family_template, expand_cortical_identifier, expand_cortical_type_family, CorticalAreaInit, CorticalAreaInitContainer,
    NeuronLayoutOf, NeuronLayoutTag, NeuronModel, NeuronModelContainer, NeuronModelMember, NeuronModelUniversalCorticalProperties,
};
use feagi_connectome::{
    with_neuron_model_feagi_advanced, with_neuron_model_feagi_simple, with_neuron_model_foundation_death, with_neuron_model_foundation_power,
    with_neuron_model_memory_fire_hashing,
};
#[cfg(feature = "std")]
use feagi_connectome::connectome_basis::neuron_model::expand_cortical_area_map;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;

pub mod cortical_identifier_base64;
pub mod foundation;
pub mod motor;
pub mod processing;
pub mod sensor;
pub mod tunable;

/// The last 6 bytes of a cortical ID, which each cortical type gives its own meaning. Converting to
/// bytes and back must return the exact same bytes, or two different byte patterns could mean the
/// same cortical area
pub trait CorticalAreaInnerID: Copy + Eq + core::hash::Hash + core::fmt::Debug + Into<[u8; 6]> + TryFrom<[u8; 6]> {}

impl CorticalAreaInnerID for [u8; 6] {}

/// Describes a specific cortical area type, as a unit struct. A cortical type could use various
/// neuron models (and their quantizations), but it must stick to a certain neuron layout. Which
/// models it allows is listed in the template of this module, and what it does with each is defined
/// by implementing their `Uses{Model}` traits
pub trait CorticalAreaType {
    /// What type of neuron layout this type uses
    type NeuronLayout: NeuronLayoutTag;

    /// What the last 6 bytes of the cortical ID of this type hold
    type Inner: CorticalAreaInnerID;
}

define_cortical_type_family_template! {
    /// The `CorticalAreaType` family: every cortical area type, each allowing one or more neuron models
    Family: CorticalAreaType,
    Error: CorticalAreaTypeSelectionError,
    ModelFamily: NeuronModel,
    ModelFamilyError: NeuronModelSelectionError,
    Identifier: CorticalIdentifier,
    IdentifierError: CorticalIdentifierParseError,

    Types: {
        /// Cortical areas fundamental to the running of the brain
        Foundation = 1: {
            /// Power management processing
            Power = 1: [FoundationPower],
            /// Termination of the brain
            Death = 2: [FoundationDeath],
        }

        /// Cortical areas that process activity within the brain
        Processing = 2: {
            /// Connects cortical areas to each other
            Interconnect = 1: [FeagiAdvanced, FeagiSimple],
            /// Stores information and grows dynamically
            Memory = 2: [MemoryFireHashing],
        }

        /// Cortical areas receiving input from outside of the brain
        Sensor = 3: {
            /// Image input
            Camera = 1: [FeagiAdvanced, FeagiSimple],
        }

        /// Cortical areas sending output outside of the brain
        Motor = 4: {
            /// Light output
            Led = 1: [FeagiAdvanced, FeagiSimple],
        }

        /// Outputs that affect the inputs
        Tunable = 5: {
            /// Adjusts the brightness of an input
            BrightnessAdjust = 1: [FeagiAdvanced, FeagiSimple],
        }
    }
}

cortical_area_type_family_template!(expand_cortical_type_family);
cortical_area_type_family_template!(expand_cortical_identifier);
#[cfg(feature = "std")]
cortical_area_type_family_template!(expand_cortical_area_map);

/// The Genomic Data For a given Cortical Area
pub type CorticalGenomicDataEnum<FIQ> = CorticalAreaTypeEnum<CorticalAreaInitContainer<FIQ>>;

/// Something that wants the data of a `CorticalAreaTypeEnum` at its concrete cortical type `T` and
/// model member `NM`
#[cfg(feature = "std")]
pub(crate) trait CorticalAreaTypeVisitor<C: NeuronModelContainer> {
    type Output;

    fn visit<T, NM>(self, data: C::Of<NM>) -> Self::Output
    where
        T: CorticalAreaTypeMemberAt<NM> + CorticalAreaTypeMapMember + CorticalAreaType<NeuronLayout = <NM as NeuronModel>::LayoutTag> + 'static,
        NM: NeuronModelMember + 'static;
}

#[cfg(feature = "std")]
impl<C: NeuronModelContainer> CorticalAreaTypeEnum<C> {
    /// Gives the data held to `visitor`, along with the cortical type and model member it is held for
    pub(crate) fn visit<V: CorticalAreaTypeVisitor<C>>(self, visitor: V) -> V::Output {
        with_cortical_area_type!(self, <T, NM> |data| visitor.visit::<T, NM>(data))
    }
}

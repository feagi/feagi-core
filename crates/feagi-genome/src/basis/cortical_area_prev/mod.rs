//! Every cortical area type. The template below lists each one by its super type, with the neuron
//! models it allows, while the details of a type live with its super type in the module of that
//! name: its marker struct, [`CorticalAreaType`] implementation, and per neuron model it allows, a
//! generator wrapper holding its defaults (see `cortical_type_generator!`). Generated items are
//! also reached through `cortical_area_type`



/// Declares the generator wrapper of a cortical type for one neuron model it allows:
/// `$wrapper<FIQ, NM>`, holding a [`NeurogenesisGeneratorUniformDefault`] as its public field, with
/// `into_model_generator`, and the `$uses_trait` implementation of the cortical type, which builds
/// the wrapper. The wrapper's `new(layout) -> Result<Self, DataValueQuantizationError>`, holding
/// the defaults of the cortical type for that model, is written next to the cortical type
macro_rules! cortical_type_generator {
    (
        $(#[$meta:meta])*
        $wrapper:ident for $cortical_type:ty, $uses_trait:ident, $model_trait:ident
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $wrapper<
            FIQ: ::feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization,
            NM: $model_trait,
        >(pub ::feagi_connectome::connectome_basis::neuron_model::NeurogenesisGeneratorUniformDefault<FIQ, NM>);

        impl<FIQ, NM> $wrapper<FIQ, NM>
        where
            FIQ: ::feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization,
            NM: $model_trait,
        {
            /// The generator, as the generator of the neuron model. Only available for a model
            /// whose generator can be made from a `NeurogenesisDefaultsGenerator`
            pub fn into_model_generator(self) -> NM::NeurogenesisGenerator<FIQ>
            where
                NM::NeurogenesisGenerator<FIQ>: ::core::convert::From<::feagi_connectome::connectome_basis::neuron_model::NeurogenesisGeneratorUniformDefault<FIQ, NM>>,
            {
                ::core::convert::Into::into(self.0)
            }
        }

        impl $uses_trait for $cortical_type {
            fn new_defaults_generator<
                FIQ: ::feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization,
                NM: $model_trait,
            >(
                layout: ::feagi_connectome::connectome_basis::neuron_model::NeuronLayoutOf<NM, FIQ>,
            ) -> ::core::result::Result<
                ::feagi_connectome::connectome_basis::neuron_model::NeurogenesisGeneratorUniformDefault<FIQ, NM>,
                ::feagi_basis::quantization::quantizable::DataValueQuantizationError,
            > {
                $wrapper::<FIQ, NM>::new(layout).map(|wrapper| wrapper.0)
            }
        }
    };
}

pub mod cortical_identifier_base64;
pub mod foundation;
pub mod motor;
pub mod processing;
pub mod sensor;
pub mod tunable;
pub mod faux_cortical_identifier;

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

//region test

pub trait CorticalTypeNeuronModelMarker {

    type NeuronModel: NeuronModel;

    
}

/// What Cortical Supertype are we using
pub enum CTEnumKind {
    Foundation(CTFoundationEnumKind),
    Processing(),
    Sensor(),
    Motor(),
    Tunable(),
}

/// Describes the cortical type under the Foundation SuperType
pub enum CTFoundationEnumKind {
    Power(CTFoundationPowerEnumKind),
    Death(),
}

/// Defines the type of neuron model Power uses
pub enum CTFoundationPowerEnumKind {
    NMFoundationPower
}

pub struct CTFFoundationPowerNMFoundationPowerMarker;



//endregion



define_cortical_type_family_template! {
    /// The `CorticalAreaType` family: every cortical area type, each allowing one or more neuron models
    Family: CorticalAreaType,
    Error: CorticalAreaTypeSelectionError,
    ModelFamily: NeuronModel,
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

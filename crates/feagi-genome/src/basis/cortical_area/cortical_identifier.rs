use crate::basis::cortical_area::cortical_id_legacy::CorticalIDLegacy;
use crate::basis::cortical_area::cortical_identifier::foundation::{FoundationCorticalIdentifier, PowerCorticalIdentifier};
use crate::basis::cortical_area::cortical_identifier::interconnect::InterconnectCorticalIdentifier;
use crate::basis::cortical_area::cortical_identifier::interface::{InputCorticalIdentifier, InterfaceCorticalIdentifier, OutputCorticalIdentifier};
use crate::basis::cortical_area::cortical_identifier::memory::MemoryCorticalIdentifier;
use crate::basis::specifications::genome_identifier_error::GenomeIdentifierError;
use crate::cortical_area_types::cortical_area_type::{CorticalAreaTypeKind, FoundationSuperTypeKind};
use crate::match_bytes_by_cortical_type;
use feagi_basis::feagi_id::feagi_id::FeagiID;

const LEGACY_PREFIX_INTERCONNECT: u8 = b'c';
const LEGACY_PREFIX_MEMORY: u8 = b'm';
const LEGACY_PREFIX_INPUT: u8 = b'i';
const LEGACY_PREFIX_OUTPUT: u8 = b'o';
const LEGACY_PREFIX_TUNABLE: u8 = b't';

const _: () = assert!(size_of::<CorticalIdentifier>() <= 8, "CorticalIdentifier must fit in 8 bytes");

/// Identifies a cortical area within the running program, keeping cortical types separated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorticalIdentifier {
    Foundation(FoundationCorticalIdentifier),
    Memory(MemoryCorticalIdentifier),
    Interconnect(InterconnectCorticalIdentifier),
    Interface(InterfaceCorticalIdentifier),
}

/// Implemented by every layer of [`CorticalIdentifier`], wrapping it up to the top level enum
pub trait ToCorticalIdentifier: Copy {
    fn to_cortical_identifier(self) -> CorticalIdentifier;
}

impl ToCorticalIdentifier for CorticalIdentifier {
    fn to_cortical_identifier(self) -> CorticalIdentifier {
        self
    }
}

impl CorticalIdentifier {
    /// The kind of cortical area this identifies
    pub const fn kind(&self) -> CorticalAreaTypeKind {
        match self {
            Self::Foundation(f) => match f {
                foundation::FoundationCorticalIdentifier::Power(_) =>
                    CorticalAreaTypeKind::Foundation(FoundationSuperTypeKind::Power)
            }
            Self::Memory(m) => todo!(), // CorticalAreaTypeKind::Memory,
            Self::Interconnect(i) => CorticalAreaTypeKind::Interconnect,
            Self::Interface(i) => todo!()
        }
    }
}



pub mod foundation {
    use super::{CorticalIdentifier, ToCorticalIdentifier};


    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum FoundationCorticalIdentifier {
        Power(PowerCorticalIdentifier),
    }



    impl ToCorticalIdentifier for FoundationCorticalIdentifier {
        fn to_cortical_identifier(self) -> CorticalIdentifier {
            CorticalIdentifier::Foundation(self)
        }
    }

    /// Identifies a brain power area
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PowerCorticalIdentifier([u8; 7]);

    impl ToCorticalIdentifier for PowerCorticalIdentifier {
        fn to_cortical_identifier(self) -> CorticalIdentifier {
            FoundationCorticalIdentifier::Power(self).to_cortical_identifier()
        }
    }

    impl PowerCorticalIdentifier {
        pub(in crate::basis) const fn new(payload: [u8; 7]) -> Self {
            Self(payload)
        }

        pub(in crate::basis) const fn payload(&self) -> [u8; 7] {
            self.0
        }
    }
}

pub mod memory {
    use super::{CorticalIdentifier, ToCorticalIdentifier};

    /// Identifies a memory area
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MemoryCorticalIdentifier([u8; 7]);

    impl ToCorticalIdentifier for MemoryCorticalIdentifier {
        fn to_cortical_identifier(self) -> CorticalIdentifier {
            CorticalIdentifier::Memory(self)
        }
    }

    impl MemoryCorticalIdentifier {
        pub(in crate::basis) const fn new(payload: [u8; 7]) -> Self {
            Self(payload)
        }

        pub(in crate::basis) const fn payload(&self) -> [u8; 7] {
            self.0
        }
    }
}

pub mod interconnect {
    use super::{CorticalIdentifier, ToCorticalIdentifier};

    /// Identifies an interconnect area
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct InterconnectCorticalIdentifier([u8; 7]);

    impl ToCorticalIdentifier for InterconnectCorticalIdentifier {
        fn to_cortical_identifier(self) -> CorticalIdentifier {
            CorticalIdentifier::Interconnect(self)
        }
    }

    impl InterconnectCorticalIdentifier {
        pub(in crate::basis) const fn new(payload: [u8; 7]) -> Self {
            Self(payload)
        }

        pub(in crate::basis) const fn payload(&self) -> [u8; 7] {
            self.0
        }
    }
}

pub mod interface {
    use super::{CorticalIdentifier, ToCorticalIdentifier};

    /// Identifies an area that is an interface to the outside of the brain
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum InterfaceCorticalIdentifier {
        Input(InputCorticalIdentifier),
        Output(OutputCorticalIdentifier),
        Tunable(TunableCorticalIdentifier),
    }

    impl ToCorticalIdentifier for InterfaceCorticalIdentifier {
        fn to_cortical_identifier(self) -> CorticalIdentifier {
            CorticalIdentifier::Interface(self)
        }
    }

    impl InterfaceCorticalIdentifier {
        /// The kind of interface area this identifies
        pub const fn kind(&self) -> InterfaceCorticalIdentifier {
            match self {
                Self::Input(i) => InterfaceCorticalIdentifier::Input(*i),
                Self::Output(o) => InterfaceCorticalIdentifier::Output(*o),
                Self::Tunable(t) => InterfaceCorticalIdentifier::Tunable(*t),
            }
        }
    }

    // TODO these are wrong, we have another layer

    /// Identifies a brain input area
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct InputCorticalIdentifier([u8; 7]);

    impl ToCorticalIdentifier for InputCorticalIdentifier {
        fn to_cortical_identifier(self) -> CorticalIdentifier {
            InterfaceCorticalIdentifier::Input(self).to_cortical_identifier()
        }
    }

    impl InputCorticalIdentifier {
        pub(in crate::basis) const fn new(payload: [u8; 7]) -> Self {
            Self(payload)
        }

        pub(in crate::basis) const fn payload(&self) -> [u8; 7] {
            self.0
        }
    }

    /// Identifies a brain output area
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OutputCorticalIdentifier([u8; 7]);

    impl ToCorticalIdentifier for OutputCorticalIdentifier {
        fn to_cortical_identifier(self) -> CorticalIdentifier {
            InterfaceCorticalIdentifier::Output(self).to_cortical_identifier()
        }
    }

    impl OutputCorticalIdentifier {
        pub(in crate::basis) const fn new(payload: [u8; 7]) -> Self {
            Self(payload)
        }

        pub(in crate::basis) const fn payload(&self) -> [u8; 7] {
            self.0
        }
    }

    /// Identifies a brain tunable area (an output that affects the input)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TunableCorticalIdentifier([u8; 7]);

    impl ToCorticalIdentifier for TunableCorticalIdentifier {
        fn to_cortical_identifier(self) -> CorticalIdentifier {
            InterfaceCorticalIdentifier::Tunable(self).to_cortical_identifier()
        }
    }

    impl TunableCorticalIdentifier {
        pub(in crate::basis) const fn new(payload: [u8; 7]) -> Self {
            Self(payload)
        }

        pub(in crate::basis) const fn payload(&self) -> [u8; 7] {
            self.0
        }
    }
}







/// Splits the prefix byte off of legacy cortical ID bytes
const fn legacy_payload(bytes: &[u8; 8]) -> [u8; 7] {
    let [_, payload @ ..] = *bytes;
    payload
}

/// Joins a prefix byte and a payload into legacy cortical ID bytes
const fn legacy_bytes(prefix: u8, payload: [u8; 7]) -> [u8; 8] {
    let [a, b, c, d, e, f, g] = payload;
    [prefix, a, b, c, d, e, f, g]
}

impl TryFrom<CorticalIDLegacy> for CorticalIdentifier {
    type Error = GenomeIdentifierError;

    /// Fails if the legacy ID is a core area that names no known foundation kind
    fn try_from(legacy: CorticalIDLegacy) -> Result<Self, Self::Error> {
        let bytes = legacy.get_id_bytes();
        let payload = legacy_payload(bytes);
        match_bytes_by_cortical_type!(bytes,
            custom => { Ok(Self::Interconnect(InterconnectCorticalIdentifier::new(payload))) },
            memory => { Ok(Self::Memory(MemoryCorticalIdentifier::new(payload))) },
            core => {
                FoundationSuperTypeKind::try_from_cortical_id_bytes_type_unchecked(bytes).map(|kind| match kind {
                    FoundationSuperTypeKind::Power => Self::Foundation(FoundationCorticalIdentifier::Power(PowerCorticalIdentifier::new(payload))),
                })
            },
            brain_input => { Ok(Self::Interface(InterfaceCorticalIdentifier::Input(InputCorticalIdentifier::new(payload)))) },
            brain_output => { Ok(Self::Interface(InterfaceCorticalIdentifier::Output(OutputCorticalIdentifier::new(payload)))) },
            invalid => { Err(GenomeIdentifierError::CannotParseFromBytes("CorticalIdentifier")) },
        )
    }
}

impl TryFrom<CorticalIdentifier> for CorticalIDLegacy {
    type Error = GenomeIdentifierError;

    /// Fails if the identifier has no legacy form. Legacy foundation areas were unique, so only the
    /// one identifier matching that fixed ID converts. Legacy IDs also have no tunable prefix
    fn try_from(identifier: CorticalIdentifier) -> Result<Self, Self::Error> {
        let bytes = match identifier {
            CorticalIdentifier::Foundation(FoundationCorticalIdentifier::Power(power)) => {
                let legacy = FoundationSuperTypeKind::Power.to_cortical_id();
                return match legacy_payload(legacy.get_id_bytes()) == power.payload() {
                    true => Ok(legacy),
                    false => Err(GenomeIdentifierError::CannotParseFromBytes("CorticalIDLegacy")),
                };
            }
            CorticalIdentifier::Memory(memory) => legacy_bytes(LEGACY_PREFIX_MEMORY, memory.payload()),
            CorticalIdentifier::Interconnect(interconnect) => legacy_bytes(LEGACY_PREFIX_INTERCONNECT, interconnect.payload()),
            CorticalIdentifier::Interface(InterfaceCorticalIdentifier::Input(input)) => legacy_bytes(LEGACY_PREFIX_INPUT, input.payload()),
            CorticalIdentifier::Interface(InterfaceCorticalIdentifier::Output(output)) => legacy_bytes(LEGACY_PREFIX_OUTPUT, output.payload()),
            CorticalIdentifier::Interface(InterfaceCorticalIdentifier::Tunable(tunable)) => legacy_bytes(LEGACY_PREFIX_TUNABLE, tunable.payload()),
        };
        CorticalIDLegacy::try_from_id_bytes(bytes)
    }
}

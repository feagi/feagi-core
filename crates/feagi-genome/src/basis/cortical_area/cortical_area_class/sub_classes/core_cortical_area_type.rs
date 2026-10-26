use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::identifiers::genome_identifier_error::GenomeIdentifierError;

/// Core cortical area classes for fundamental / specific brain functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CoreCorticalClass {
    /// Termination/death signal processing
    Death,
    /// Power management processing
    Power,
    /// Brain fatigue indicator
    Fatigue,
    /// Pain signal processing
    Pain,
    /// Pleasure signal processing
    Pleasure,
    /// Fear signal processing
    Fear,
    /// Hope signal processing
    Hope,
}

impl CoreCorticalClass {
    pub(crate) fn try_from_cortical_id_bytes_type_unchecked(cortical_id_bytes: &[u8; 8]) -> Result<CoreCorticalClass, GenomeIdentifierError> {
        match cortical_id_bytes {
            b"___death" => Ok(CoreCorticalClass::Death),
            b"___power" => Ok(CoreCorticalClass::Power),
            b"___fatig" => Ok(CoreCorticalClass::Fatigue),
            b"___pain_" => Ok(CoreCorticalClass::Pain),
            b"___pleas" => Ok(CoreCorticalClass::Pleasure),
            b"___fear_" => Ok(CoreCorticalClass::Fear),
            b"___hope_" => Ok(CoreCorticalClass::Hope),
            _ => Err(GenomeIdentifierError::CannotParseFromBytes("CorticalID")),
        }
    }

    pub fn to_cortical_id(&self) -> CorticalID {
        match self {
            Self::Death => CorticalID::from_bytes_const(*b"___death"),
            Self::Power => CorticalID::from_bytes_const(*b"___power"),
            Self::Fatigue => CorticalID::from_bytes_const(*b"___fatig"),
            Self::Pain => CorticalID::from_bytes_const(*b"___pain_"),
            Self::Pleasure => CorticalID::from_bytes_const(*b"___pleas"),
            Self::Fear => CorticalID::from_bytes_const(*b"___fear_"),
            Self::Hope => CorticalID::from_bytes_const(*b"___hope_"),
        }
    }
}

impl core::fmt::Display for CoreCorticalClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ch = match self {
            CoreCorticalClass::Death => "Death",
            CoreCorticalClass::Power => "Power",
            CoreCorticalClass::Fatigue => "Fatigue",
            CoreCorticalClass::Pain => "Pain",
            CoreCorticalClass::Pleasure => "Pleasure",
            CoreCorticalClass::Fear => "Fear",
            CoreCorticalClass::Hope => "Hope",
        };
        write!(f, "CoreCorticalType({})", ch)
    }
}

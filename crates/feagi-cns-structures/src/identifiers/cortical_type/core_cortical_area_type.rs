use crate::identifiers::cortical_id::CorticalID;
use crate::identifiers::genome_identifier_error::GenomeIdentifierError;

/// Core cortical_area area types for fundamental brain functions.
///
/// Represents essential processing regions that manage the agent's power,
/// termination states, and fatigue monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CoreCorticalType {
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

impl CoreCorticalType {
    pub(crate) fn try_from_cortical_id_bytes_type_unchecked(
        cortical_id_bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH],
    ) -> Result<CoreCorticalType, GenomeIdentifierError> {
        match cortical_id_bytes {
            b"___death" => Ok(CoreCorticalType::Death),
            b"___power" => Ok(CoreCorticalType::Power),
            b"___fatig" => Ok(CoreCorticalType::Fatigue),
            b"___pain_" => Ok(CoreCorticalType::Pain),
            b"___pleas" => Ok(CoreCorticalType::Pleasure),
            b"___fear_" => Ok(CoreCorticalType::Fear),
            b"___hope_" => Ok(CoreCorticalType::Hope),
            _ => Err(GenomeIdentifierError::CannotParseFromBytes("CorticalID")),
        }
    }

    pub fn to_cortical_id(&self) -> CorticalID {
        match self {
            Self::Death => CorticalID::from_bytes_const(*b"___death" ),
            Self::Power => CorticalID::from_bytes_const(*b"___power" ),
            Self::Fatigue => CorticalID::from_bytes_const(*b"___fatig" ),
            Self::Pain => CorticalID::from_bytes_const(*b"___pain_" ),
            Self::Pleasure => CorticalID::from_bytes_const(*b"___pleas"),
            Self::Fear => CorticalID::from_bytes_const(*b"___fear_" ),
            Self::Hope => CorticalID::from_bytes_const(*b"___hope_" ),
        }
    }
}

impl core::fmt::Display for CoreCorticalType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ch = match self {
            CoreCorticalType::Death => "Death",
            CoreCorticalType::Power => "Power",
            CoreCorticalType::Fatigue => "Fatigue",
            CoreCorticalType::Pain => "Pain",
            CoreCorticalType::Pleasure => "Pleasure",
            CoreCorticalType::Fear => "Fear",
            CoreCorticalType::Hope => "Hope",
        };
        write!(f, "CoreCorticalType({})", ch)
    }
}
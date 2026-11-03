use crate::basis::cortical_area::cortical_id_legacy::CorticalIDLegacy;
use crate::basis::specifications::genome_identifier_error::GenomeIdentifierError;
use crate::cortical_area_types::cortical_area_type::FoundationSuperTypeKind;

// NOTE: Future foundation types, once added to the family template
//     /// Termination/death signal processing
//     Death,
//     /// Brain fatigue indicator
//     Fatigue,
//     /// Pain signal processing
//     Pain,
//     /// Pleasure signal processing
//     Pleasure,
//     /// Fear signal processing
//     Fear,
//     /// Hope signal processing
//     Hope,

impl FoundationSuperTypeKind {
    pub(crate) fn try_from_cortical_id_bytes_type_unchecked(cortical_id_bytes: &[u8; 8]) -> Result<FoundationSuperTypeKind, GenomeIdentifierError> {
        match cortical_id_bytes {
            //b"___death" => Ok(FoundationSuperTypeKind::Death),
            b"___power" => Ok(FoundationSuperTypeKind::Power),
            //b"___fatig" => Ok(FoundationSuperTypeKind::Fatigue),
            //b"___pain_" => Ok(FoundationSuperTypeKind::Pain),
            //b"___pleas" => Ok(FoundationSuperTypeKind::Pleasure),
            //b"___fear_" => Ok(FoundationSuperTypeKind::Fear),
            //b"___hope_" => Ok(FoundationSuperTypeKind::Hope),
            _ => Err(GenomeIdentifierError::CannotParseFromBytes("CorticalID")),
        }
    }

    pub fn to_cortical_id(&self) -> CorticalIDLegacy {
        match self {
            //Self::Death => const { CorticalID::from_id_bytes_const(*b"___death") },
            Self::Power => const { CorticalIDLegacy::from_id_bytes_const(*b"___power") },
            //Self::Fatigue => const { CorticalID::from_id_bytes_const(*b"___fatig") },
            //Self::Pain => const { CorticalID::from_id_bytes_const(*b"___pain_") },
            //Self::Pleasure => const { CorticalID::from_id_bytes_const(*b"___pleas") },
            //Self::Fear => const { CorticalID::from_id_bytes_const(*b"___fear_") },
            //Self::Hope => const { CorticalID::from_id_bytes_const(*b"___hope_") },
        }
    }
}

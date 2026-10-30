use super::power::PowerTypeCorticalGenomicDataEnum;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::identifiers::genome_identifier_error::GenomeIdentifierError;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


/// The Genomic Data for a given Core Subtype Cortical Area
pub enum CoreSuperTypeCorticalGenomicDataEnum<FIQ: FeagiIndexQuantization> {
    Power(PowerTypeCorticalGenomicDataEnum<FIQ>) // TODO rename in macro to PowerModelCorticalGenomicDataEnum
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CoreCorticalAreaSuperTypeEnum {
    ///// Termination/death signal processing
    //Death,
    /// Power management processing
    Power,
    ///// Brain fatigue indicator
    //Fatigue,
    ///// Pain signal processing
    //Pain,
    ///// Pleasure signal processing
    //Pleasure,
    ///// Fear signal processing
    //Fear,
    ///// Hope signal processing
    //Hope,
}

impl CoreCorticalAreaSuperTypeEnum {
    pub(crate) fn try_from_cortical_id_bytes_type_unchecked(cortical_id_bytes: &[u8; 8]) -> Result<CoreCorticalAreaSuperTypeEnum, GenomeIdentifierError> {
        match cortical_id_bytes {
            //b"___death" => Ok(CoreCorticalAreaSubType::Death),
            b"___power" => Ok(CoreCorticalAreaSuperTypeEnum::Power),
            //b"___fatig" => Ok(CoreCorticalAreaSubType::Fatigue),
            //b"___pain_" => Ok(CoreCorticalAreaSubType::Pain),
            //b"___pleas" => Ok(CoreCorticalAreaSubType::Pleasure),
            //b"___fear_" => Ok(CoreCorticalAreaSubType::Fear),
            //b"___hope_" => Ok(CoreCorticalAreaSubType::Hope),
            _ => Err(GenomeIdentifierError::CannotParseFromBytes("CorticalID")),
        }
    }

    pub fn to_cortical_id(&self) -> CorticalID {
        match self {
            //Self::Death => const { CorticalID::from_id_bytes_const(*b"___death") },
            Self::Power => const { CorticalID::from_id_bytes_const(*b"___power") },
            //Self::Fatigue => const { CorticalID::from_id_bytes_const(*b"___fatig") },
            //Self::Pain => const { CorticalID::from_id_bytes_const(*b"___pain_") },
            //Self::Pleasure => const { CorticalID::from_id_bytes_const(*b"___pleas") },
            //Self::Fear => const { CorticalID::from_id_bytes_const(*b"___fear_") },
            //Self::Hope => const { CorticalID::from_id_bytes_const(*b"___hope_") },
        }
    }
}

impl core::fmt::Display for CoreCorticalAreaSuperTypeEnum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ch = match self {
            //CoreCorticalAreaSubType::Death => "Death",
            CoreCorticalAreaSuperTypeEnum::Power => "Power",
            //CoreCorticalAreaSubType::Fatigue => "Fatigue",
            //CoreCorticalAreaSubType::Pain => "Pain",
            //CoreCorticalAreaSubType::Pleasure => "Pleasure",
            //CoreCorticalAreaSubType::Fear => "Fear",
            //CoreCorticalAreaSubType::Hope => "Hope",
        };
        write!(f, "CoreCorticalType({})", ch)
    }
}


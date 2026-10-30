use super::power::PowerTypeCorticalGenomicDataEnum;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::identifiers::genome_identifier_error::GenomeIdentifierError;
use crate::basis::invalid_cortical_enum_error::CorticalMatchSelectionError;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

//region Data

/// The Genomic Data for a given Core Subtype Cortical Area
pub enum CoreSuperTypeCorticalGenomicDataEnum<FIQ: FeagiIndexQuantization> {
    Power(PowerTypeCorticalGenomicDataEnum<FIQ>) // TODO rename in macro to PowerModelCorticalGenomicDataEnum
}

impl<FIQ: FeagiIndexQuantization> From<PowerTypeCorticalGenomicDataEnum<FIQ>> for CoreSuperTypeCorticalGenomicDataEnum<FIQ>
{
    fn from(value: PowerTypeCorticalGenomicDataEnum<FIQ>) -> Self {
        CoreSuperTypeCorticalGenomicDataEnum::Power(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<CoreSuperTypeCorticalGenomicDataEnum<FIQ>> for PowerTypeCorticalGenomicDataEnum<FIQ>
{
    type Error = CorticalMatchSelectionError;

    fn try_from(value: CoreSuperTypeCorticalGenomicDataEnum<FIQ>) -> Result<Self, Self::Error> {
        #[allow(unreachable_patterns)] // Only one variant exists for now
        match value {
            CoreSuperTypeCorticalGenomicDataEnum::Power(t) => {
                Ok(t)
            }
            _ => {Err(CorticalMatchSelectionError::IncorrectCorticalType {tried: "", actual: ""})}
        }
    }
}

//endregion


//region Type
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

//endregion


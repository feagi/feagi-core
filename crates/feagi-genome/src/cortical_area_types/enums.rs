

// TODO macro for sensor motor should be a template that can generate nested enums here

use crate::cortical_area_types::interconnect::InterConnectCorticalAreaEModelEnum;
use crate::cortical_area_types::super_type_core::enums::{CoreCorticalAreaSuperTypeEnum, CoreSuperTypeCorticalGenomicDataEnum};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::invalid_cortical_enum_error::CorticalMatchSelectionError;
//region Data

/// The Genomic Data For a given Cortical Area
pub enum CorticalGenomicDataEnum<FIQ: FeagiIndexQuantization> {
    Core(CoreSuperTypeCorticalGenomicDataEnum<FIQ>),
    Interconnect(InterConnectCorticalAreaEModelEnum<FIQ>),
    //Memory(),
}

impl<FIQ: FeagiIndexQuantization> From<CoreSuperTypeCorticalGenomicDataEnum<FIQ>> for CorticalGenomicDataEnum<FIQ>
{
    fn from(value: CoreSuperTypeCorticalGenomicDataEnum<FIQ>) -> Self {
        CorticalGenomicDataEnum::Core(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<CorticalGenomicDataEnum<FIQ>> for CoreSuperTypeCorticalGenomicDataEnum<FIQ>
{
    type Error = CorticalMatchSelectionError;

    fn try_from(value: CorticalGenomicDataEnum<FIQ>) -> Result<Self, Self::Error> {
        match value {
            CorticalGenomicDataEnum::Core(t) => {
                Ok(t)
            }
            _ => {Err(CorticalMatchSelectionError::IncorrectCorticalSuperType {tried: "", actual: ""})}
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<InterConnectCorticalAreaEModelEnum<FIQ>> for CorticalGenomicDataEnum<FIQ>
{
    fn from(value: InterConnectCorticalAreaEModelEnum<FIQ>) -> Self {
        CorticalGenomicDataEnum::Interconnect(value)
    }
}


impl<FIQ: FeagiIndexQuantization> TryFrom<CorticalGenomicDataEnum<FIQ>> for InterConnectCorticalAreaEModelEnum<FIQ>
{
    type Error = CorticalMatchSelectionError;

    fn try_from(value: CorticalGenomicDataEnum<FIQ>) -> Result<Self, Self::Error> {
        match value {
            CorticalGenomicDataEnum::Interconnect(t) => {
                Ok(t)
            }
            _ => {Err(CorticalMatchSelectionError::IncorrectCorticalSuperType {tried: "", actual: ""})}
        }
    }
}


//endregion

//region Type

/// A flat enum that describes the cortical type of the given cortical area
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
pub enum CorticalAreaTypeEnum {
    Core(CoreCorticalAreaSuperTypeEnum),
    Interconnect,
    //Memory(),
    //EmbodimentIO TODO
}

impl CorticalAreaTypeEnum {

}


/*

impl core::fmt::Display for CorticalAreaType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CorticalAreaType::Core(c) => write!(f, "Core({})", c),
            CorticalAreaType::Interconnect(c) => write!(f, "Interconnect({})", c),
            CorticalAreaType::Memory(c) => write!(f, "Memory({})", c),
            CorticalAreaType::BrainInput() => write!(f, "BrainInput()", ),
            CorticalAreaType::BrainOutput() => write!(f, "BrainOutput()", ),
        }
    }
}


 */
//endregion
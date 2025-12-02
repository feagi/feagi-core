use std::fmt;
use crate::FeagiDataError;
use crate::genomic::cortical_area::cortical_id::CorticalID;
use crate::genomic::cortical_area::io_cortical_area_data_type::IOCorticalAreaDataFlag;

// Describes the method data is encoded within a cortical area

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CorticalAreaType {
    Core(CoreCorticalType),
    Custom(CustomCorticalType),
    Memory(MemoryCorticalType),
    BrainInput(IOCorticalAreaDataFlag),
    BrainOutput(IOCorticalAreaDataFlag)
}

//region Core
/// Core cortical area types for fundamental brain functions.
///
/// Represents essential processing regions that manage the agent's power
/// and termination states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreCorticalType {
    /// Termination/death signal processing
    Death,
    /// Power management processing
    Power
}

impl CoreCorticalType {
    pub(crate) fn try_from_cortical_id_bytes_type_unchecked(cortical_id_bytes: &[u8; CorticalID::NUMBER_OF_BYTES]) -> Result<CoreCorticalType, FeagiDataError> {
        match cortical_id_bytes {
            b"___death" => Ok(CoreCorticalType::Death),
            b"___power" => Ok(CoreCorticalType::Power),
            _ => Err(FeagiDataError::BadParameters(format!("Unable to cast cortical ID bytes '{}' to a core cortical type!", String::from_utf8_lossy(cortical_id_bytes))))
        }

    }

    pub fn to_cortical_id(&self) -> CorticalID {
        match self {
            Self::Death => CorticalID{bytes: *b"___death"},
            Self::Power => CorticalID{bytes: *b"___power"},
        }
    }
}


impl fmt::Display for CoreCorticalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            CoreCorticalType::Death => "Death",
            CoreCorticalType::Power => "Power"
        };
        write!(f, "CoreCorticalType({})", ch)
    }
}

//endregion

//region Custom
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CustomCorticalType {
    #[default]
    LeakyIntegrateFire
}


//endregion

//region Memory

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MemoryCorticalType {
    #[default]
    Memory
}

//endregion
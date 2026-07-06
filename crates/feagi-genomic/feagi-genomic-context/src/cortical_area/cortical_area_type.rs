use serde::{Deserialize, Serialize};
use crate::cortical_area::CorticalID;
use crate::cortical_area::io_cortical_area_configuration_flag::IOCorticalAreaConfigurationFlag;
use crate::feagi_genome_context_error::{FeagiCorticalTypeErrKey, FeagiGenomeContextError};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum CorticalAreaType {
    Core(CoreCorticalType),
    Custom(CustomCorticalType),
    Memory(MemoryCorticalType),
    BrainInput(IOCorticalAreaConfigurationFlag),
    BrainOutput(IOCorticalAreaConfigurationFlag),
}

impl core::fmt::Display for CorticalAreaType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CorticalAreaType::Core(c) => write!(f, "Core({})", c),
            CorticalAreaType::Custom(c) => write!(f, "Custom({})", c),
            CorticalAreaType::Memory(c) => write!(f, "Memory({})", c),
            CorticalAreaType::BrainInput(c) => write!(f, "BrainInput({})", c),
            CorticalAreaType::BrainOutput(c) => write!(f, "BrainOutput({})", c),
        }
    }
}

//region Core
/// Core cortical_area area types for fundamental brain functions.
///
/// Represents essential processing regions that manage the agent's power,
/// termination states, and fatigue monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreCorticalType {
    /// Termination/death signal processing
    Death,
    /// Power management processing
    Power,
    /// Brain fatigue indicator - activates when neuron/synapse arrays exceed 85% capacity
    Fatigue,
}

impl CoreCorticalType {
    pub(crate) fn try_from_cortical_id_bytes_type_unchecked(
        cortical_id_bytes: &[u8; CorticalID::NUMBER_OF_BYTES],
    ) -> Result<CoreCorticalType, FeagiGenomeContextError> {
        match cortical_id_bytes {
            b"___death" => Ok(CoreCorticalType::Death),
            b"___power" => Ok(CoreCorticalType::Power),
            b"___fatig" => Ok(CoreCorticalType::Fatigue),
            _ => Err(
                FeagiCorticalTypeErrKey::new(
                    "cortical_area ID bytes do not match a known core cortical_area type"
                ).into()
            ),
        }
    }

    pub fn to_cortical_id(&self) -> CorticalID {
        match self {
            Self::Death => CorticalID {
                bytes: *b"___death",
            },
            Self::Power => CorticalID {
                bytes: *b"___power",
            },
            Self::Fatigue => CorticalID {
                bytes: *b"___fatig",
            },
        }
    }
}

impl core::fmt::Display for CoreCorticalType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ch = match self {
            CoreCorticalType::Death => "Death",
            CoreCorticalType::Power => "Power",
            CoreCorticalType::Fatigue => "Fatigue",
        };
        write!(f, "CoreCorticalType({})", ch)
    }
}

//endregion

//region Custom
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CustomCorticalType {
    #[default]
    LeakyIntegrateFire,
}

impl core::fmt::Display for CustomCorticalType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LeakyIntegrateFire => write!(f, "Leaky IntegrateFire"),
        }
    }
}

//endregion

//region Memory

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MemoryCorticalType {
    #[default]
    Memory,
}

impl core::fmt::Display for MemoryCorticalType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Memory => write!(f, "Memory"),
        }
    }
}

//endregion

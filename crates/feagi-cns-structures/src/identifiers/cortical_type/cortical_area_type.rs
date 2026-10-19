use crate::identifiers::cortical_type::core_cortical_area_type::CoreCorticalType;
use crate::identifiers::cortical_type::interconnect_cortical_area_type::InterconnectCorticalType;
use crate::identifiers::cortical_type::memory_cortical_area_type::MemoryCorticalType;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
pub enum CorticalAreaType {
    Core(CoreCorticalType),
    Interconnect(InterconnectCorticalType),
    Memory(MemoryCorticalType),
    BrainInput(),
    BrainOutput(),
}

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

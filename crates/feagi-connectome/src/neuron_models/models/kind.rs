use crate::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantKind;
use crate::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantKind;
use crate::neuron_models::models::foundation_death::quantizations::NMFoundationDeathQuantKind;
use crate::neuron_models::models::foundation_power::quantizations::NMFoundationPowerQuantKind;
use crate::neuron_models::models::memory_fire_hashing::quantizations::NMMemoryFireHashingQuantKind;



/// Describes the neuron model alone, without its quantization
#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub enum NMModelKind {
    FeagiAdvanced,
    FeagiSimple,
    FoundationDeath,
    FoundationPower,
    MemoryFireHashing,
}






/// Describes the neuron model, with a nested enum then describing the quantization
#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub enum NMKind {
    FeagiAdvanced(NMFeagiAdvancedQuantKind),
    FeagiSimple(NMFeagiSimpleQuantKind),
    FoundationDeath(NMFoundationDeathQuantKind),
    FoundationPower(NMFoundationPowerQuantKind),
    MemoryFireHashing(NMMemoryFireHashingQuantKind),
}

impl NMKind {
    /// The neuron model, without its quantization
    pub const fn model_kind(&self) -> NMModelKind {
        match self {
            Self::FeagiAdvanced(_) => NMModelKind::FeagiAdvanced,
            Self::FeagiSimple(_) => NMModelKind::FeagiSimple,
            Self::FoundationDeath(_) => NMModelKind::FoundationDeath,
            Self::FoundationPower(_) => NMModelKind::FoundationPower,
            Self::MemoryFireHashing(_) => NMModelKind::MemoryFireHashing,
        }
    }
}
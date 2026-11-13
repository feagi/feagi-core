use crate::neuron_models::models::exclusive_death::quantizations::NMExclusiveDeathQuantKind;
use crate::neuron_models::models::exclusive_power::quantizations::NMExclusivePowerQuantKind;
use crate::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantKind;
use crate::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantKind;
use crate::neuron_models::models::memory_fire_hashing::quantizations::NMMemoryFireHashingQuantKind;

/// Describes the neuron model, with a nested enum then describing the quantization
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub enum NMKind {
    FeagiAdvanced(NMFeagiAdvancedQuantKind),
    FeagiSimple(NMFeagiSimpleQuantKind),
    ExclusiveDeath(NMExclusiveDeathQuantKind),
    ExclusivePower(NMExclusivePowerQuantKind),
    MemoryFireHashing(NMMemoryFireHashingQuantKind),
}

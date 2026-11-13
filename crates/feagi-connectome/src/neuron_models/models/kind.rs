use crate::neuron_models::models::container::NMContainerMarker;
use crate::neuron_models::models::NMModeled;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use crate::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantKind;
use crate::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantKind;
use crate::neuron_models::models::exclusive_death::quantizations::NMExclusiveDeathQuantKind;
use crate::neuron_models::models::exclusive_power::quantizations::NMExclusivePowerQuantKind;
use crate::neuron_models::models::memory_fire_hashing::quantizations::NMMemoryFireHashingQuantKind;


/// Describes the neuron model, with a nested enum then describing the quantization
#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub enum NMKind {
    FeagiAdvanced(NMFeagiAdvancedQuantKind),
    FeagiSimple(NMFeagiSimpleQuantKind),
    ExclusiveDeath(NMExclusiveDeathQuantKind),
    ExclusivePower(NMExclusivePowerQuantKind),
    MemoryFireHashing(NMMemoryFireHashingQuantKind),
}

/// Describes the neuron model, with a nested enum then holding the quantization marker struct
pub type NMQuantedKind = NMModeled<NMContainerMarker>;

impl NMQuantedKind {
    /// The model and quantization of any neuron model, from its marker struct type
    pub fn from_nm_quant<NM>() -> NMQuantedKind
    where
        NM: NeuronModel,
        NM::QuantedKind: Into<NMQuantedKind>,
    {
        NM::QUANTED_KIND.into()
    }
}

// The marker structs are unit structs, so the kind is fully described by its `NMKind`
impl Copy for NMQuantedKind {}

impl PartialEq for NMQuantedKind {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl Eq for NMQuantedKind {}

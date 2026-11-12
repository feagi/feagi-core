//! The `MemoryFireHashing` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::memory_fire_hashing`, and its burst engine dynamics
use crate::neuron_models::models::memory_fire_hashing::quantizations::NMMemoryFireHashingQuantKind;

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::MemoryFireHashingBurstEngineDynamics as BurstEngineDynamics;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::single_stepped_dynamic::NeuronLayoutTagSingleSteppedDynamic;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;


/// Quantization For the Memory type
pub trait NMMemoryFireHashingTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagSingleSteppedDynamic,
    CorticalDataProperties=NMMemoryFireHashingDataCorticalProperties<Self>,
    CorticalDataInternal=NMMemoryFireHashingDataCorticalInternal<Self>,
    CorticalDataScratch=NMMemoryFireHashingDataCorticalScratch<Self>,
    CorticalDataVariance=NMMemoryFireHashingDataCorticalVariance<Self>,
    NeuronDataInternal=NMMemoryFireHashingDataNeuronInternal<Self>,
    NeuronDataScratch=NMMemoryFireHashingDataNeuronScratch<Self>,
    NeuronDataVariance=NMMemoryFireHashingDataNeuronVariance<Self>,
    QuantKind=NMMemoryFireHashingQuantKind,
>
{
    #[doc = " For handling timing"]
    type Period: ::feagi_basis::quantization::quantizable::QuantizedDecimalUnwrappedTrait;
}
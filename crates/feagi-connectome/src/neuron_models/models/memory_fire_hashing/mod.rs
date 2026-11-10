//! The `MemoryFireHashing` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::memory_fire_hashing`, and its burst engine dynamics

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::MemoryFireHashingBurstEngineDynamics as BurstEngineDynamics;
use crate::connectome_basis::neuron_model::{NeuronLayoutTagSingleSteppedDynamic, NeuronModel};


/// Quantization For the Memory type
pub trait NMMemoryFireHashingTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagSingleSteppedDynamic,
    CorticalDataProperties=NMMemoryFireHashingDataCorticalProperties<Self>,
    CorticalDataInternal=NMMemoryFireHashingDataCorticalInternal<Self>,
    CorticalDataScratch=NMMemoryFireHashingDataCorticalScratch<Self>,
    CorticalDataVariance=NMMemoryFireHashingDataCorticalVariance<Self>,
    NeuronDataInternal=NMMemoryFireHashingDataNeuronInternal<Self>,
    NeuronDataScratch=NMMemoryFireHashingDataNeuronScratch<Self>,
    NeuronDataVariance=NMMemoryFireHashingDataNeuronVariance<Self>
>
{
    #[doc = " For handling timing"]
    type Period: ::feagi_basis::quantization::quantizable::QuantizedDecimalUnwrappedTrait;
}
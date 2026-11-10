//! The `FoundationPower` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::foundation_power`, and its burst engine dynamics

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::FoundationPowerBurstEngineDynamics as BurstEngineDynamics;
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};


/// Quantization For the Foundation Power type
pub trait NMFoundationPowerTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFoundationPowerDataCorticalProperties<Self>,
    CorticalDataInternal=NMFoundationPowerDataCorticalInternal<Self>,
    CorticalDataScratch=NMFoundationPowerDataCorticalScratch<Self>,
    CorticalDataVariance=NMFoundationPowerDataCorticalVariance<Self>,
    NeuronDataInternal=NMFoundationPowerDataNeuronInternal<Self>,
    NeuronDataScratch=NMFoundationPowerDataNeuronScratch<Self>,
    NeuronDataVariance=NMFoundationPowerDataNeuronVariance<Self>
>
{
}
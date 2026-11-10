//! The `FoundationDeath` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::foundation_death`, and its burst engine dynamics

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::FoundationDeathBurstEngineDynamics as BurstEngineDynamics;
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};


/// Quantization For the Foundation Death type
pub trait NMFoundationDeathTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFoundationDeathDataCorticalProperties<Self>,
    CorticalDataInternal=NMFoundationDeathDataCorticalInternal<Self>,
    CorticalDataScratch=NMFoundationDeathDataCorticalScratch<Self>,
    CorticalDataVariance=NMFoundationDeathDataCorticalVariance<Self>,
    NeuronDataInternal=NMFoundationDeathDataNeuronInternal<Self>,
    NeuronDataScratch=NMFoundationDeathDataNeuronScratch<Self>,
    NeuronDataVariance=NMFoundationDeathDataNeuronVariance<Self>
>
{
}
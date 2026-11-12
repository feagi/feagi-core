//! The `FoundationDeath` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::foundation_death`, and its burst engine dynamics
use crate::neuron_models::models::foundation_death::quantizations::NMFoundationDeathQuantKind;

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::FoundationDeathBurstEngineDynamics as BurstEngineDynamics;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;


/// Quantization For the Foundation Death type
pub trait NMFoundationDeathTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFoundationDeathDataCorticalProperties<Self>,
    CorticalDataInternal=NMFoundationDeathDataCorticalInternal<Self>,
    CorticalDataScratch=NMFoundationDeathDataCorticalScratch<Self>,
    CorticalDataVariance=NMFoundationDeathDataCorticalVariance<Self>,
    NeuronDataInternal=NMFoundationDeathDataNeuronInternal<Self>,
    NeuronDataScratch=NMFoundationDeathDataNeuronScratch<Self>,
    NeuronDataVariance=NMFoundationDeathDataNeuronVariance<Self>,
    QuantKind=NMFoundationDeathQuantKind,
>
{
}
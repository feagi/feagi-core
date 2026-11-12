//! The `FoundationPower` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::foundation_power`, and its burst engine dynamics
use crate::neuron_models::models::foundation_power::quantizations::NMFoundationPowerQuantKind;

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::FoundationPowerBurstEngineDynamics as BurstEngineDynamics;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;


/// Quantization For the Foundation Power type
pub trait NMFoundationPowerTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFoundationPowerDataCorticalProperties<Self>,
    CorticalDataInternal=NMFoundationPowerDataCorticalInternal<Self>,
    CorticalDataScratch=NMFoundationPowerDataCorticalScratch<Self>,
    CorticalDataVariance=NMFoundationPowerDataCorticalVariance<Self>,
    NeuronDataInternal=NMFoundationPowerDataNeuronInternal<Self>,
    NeuronDataScratch=NMFoundationPowerDataNeuronScratch<Self>,
    NeuronDataVariance=NMFoundationPowerDataNeuronVariance<Self>,
    QuantKind=NMFoundationPowerQuantKind,
>
{
}
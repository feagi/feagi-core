//! The `ExclusivePower` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::exclusive_power`, and its burst engine dynamics
use crate::neuron_models::models::exclusive_power::quantizations::NMExclusivePowerQuantKind;

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::ExclusivePowerBurstEngineDynamics as BurstEngineDynamics;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;


/// Quantization For the Exclusive Power type
pub trait NMExclusivePowerTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMExclusivePowerDataCorticalProperties<Self>,
    CorticalDataInternal=NMExclusivePowerDataCorticalInternal<Self>,
    CorticalDataScratch=NMExclusivePowerDataCorticalScratch<Self>,
    CorticalDataVariance=NMExclusivePowerDataCorticalVariance<Self>,
    NeuronDataInternal=NMExclusivePowerDataNeuronInternal<Self>,
    NeuronDataScratch=NMExclusivePowerDataNeuronScratch<Self>,
    NeuronDataVariance=NMExclusivePowerDataNeuronVariance<Self>,
    QuantKind=NMExclusivePowerQuantKind,
>
{
}
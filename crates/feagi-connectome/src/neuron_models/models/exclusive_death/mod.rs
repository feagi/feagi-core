//! The `ExclusiveDeath` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::exclusive_death`, and its burst engine dynamics
use crate::neuron_models::models::exclusive_death::quantizations::NMExclusiveDeathQuantKind;

pub(crate) mod burst_dynamics;
pub mod data;
pub mod neurogenesis;
pub mod quantizations;

pub use burst_dynamics::ExclusiveDeathBurstEngineDynamics as BurstEngineDynamics;

pub use data::*;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;

/// Quantization For the Exclusive Death type
pub trait NMExclusiveDeathTrait:
    NeuronModel<
    NeuronLayoutTag = NeuronLayoutTagVoxel,
    CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>,
    CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>,
    CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>,
    CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>,
    NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>,
    NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>,
    NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>,
    QuantKind = NMExclusiveDeathQuantKind,
>
{
}

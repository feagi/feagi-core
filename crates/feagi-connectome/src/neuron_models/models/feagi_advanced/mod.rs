//! The `FeagiAdvanced` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::feagi_advanced`, and its burst engine dynamics

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::FeagiAdvancedBurstEngineDynamics as BurstEngineDynamics;
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};


/// The main neuron model used by FEAGI, effectively an extended LIF model
pub trait NMFeagiAdvancedTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFeagiAdvancedDataCorticalProperties<Self>,
    CorticalDataInternal=NMFeagiAdvancedDataCorticalInternal<Self>,
    CorticalDataScratch=NMFeagiAdvancedDataCorticalScratch<Self>,
    CorticalDataVariance=NMFeagiAdvancedDataCorticalVariance<Self>,
    NeuronDataInternal=NMFeagiAdvancedDataNeuronInternal<Self>,
    NeuronDataScratch=NMFeagiAdvancedDataNeuronScratch<Self>,
    NeuronDataVariance=NMFeagiAdvancedDataNeuronVariance<Self>
>
{
    #[doc = "For handling timing"]
    type Period: ::feagi_basis::quantization::quantizable::QuantizedDecimalUnwrappedTrait;
}
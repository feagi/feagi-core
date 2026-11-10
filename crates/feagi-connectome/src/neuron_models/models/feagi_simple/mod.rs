//! The `FeagiSimple` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::feagi_simple`, and its burst engine dynamics

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::FeagiSimpleBurstEngineDynamics as BurstEngineDynamics;
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};


/// Quantization For the Feagi Simple type
pub trait NMFeagiSimpleTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFeagiSimpleDataCorticalProperties<Self>,
    CorticalDataInternal=NMFeagiSimpleDataCorticalInternal<Self>,
    CorticalDataScratch=NMFeagiSimpleDataCorticalScratch<Self>,
    CorticalDataVariance=NMFeagiSimpleDataCorticalVariance<Self>,
    NeuronDataInternal=NMFeagiSimpleDataNeuronInternal<Self>,
    NeuronDataScratch=NMFeagiSimpleDataNeuronScratch<Self>,
    NeuronDataVariance=NMFeagiSimpleDataNeuronVariance<Self>
>
{
    #[doc = " For handling timing"]
    type Period: ::feagi_basis::quantization::quantizable::QuantizedDecimalUnwrappedTrait;
}
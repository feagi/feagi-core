//! The `FeagiAdvanced` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::feagi_advanced`, and its burst engine dynamics
use crate::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantKind;

pub mod quantizations;
pub mod data;
pub mod neurogenesis;
pub(crate) mod burst_dynamics;


pub use data::*;
pub use burst_dynamics::FeagiAdvancedBurstEngineDynamics as BurstEngineDynamics;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;

/// The main neuron model used by FEAGI, effectively an extended LIF model
pub trait NMFeagiAdvancedTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFeagiAdvancedDataCorticalProperties<Self>,
    CorticalDataInternal=NMFeagiAdvancedDataCorticalInternal<Self>,
    CorticalDataScratch=NMFeagiAdvancedDataCorticalScratch<Self>,
    CorticalDataVariance=NMFeagiAdvancedDataCorticalVariance<Self>,
    NeuronDataInternal=NMFeagiAdvancedDataNeuronInternal<Self>,
    NeuronDataScratch=NMFeagiAdvancedDataNeuronScratch<Self>,
    NeuronDataVariance=NMFeagiAdvancedDataNeuronVariance<Self>,
    QuantKind=NMFeagiAdvancedQuantKind,
>
{
    #[doc = "For handling timing"]
    type Period: ::feagi_basis::quantization::quantizable::QuantizedDecimalUnwrappedTrait;
}
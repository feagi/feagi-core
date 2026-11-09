//! The `FeagiSimple` neuron model: its generated quantization items, reached through
//! `connectome_basis::neuron_model::feagi_simple`, and its burst engine dynamics

pub(crate) mod data;
pub(crate) mod neurogenesis;
pub(crate) mod burst_dynamics;

pub use data::*;
pub use burst_dynamics::FeagiSimpleBurstEngineDynamics as BurstEngineDynamics;

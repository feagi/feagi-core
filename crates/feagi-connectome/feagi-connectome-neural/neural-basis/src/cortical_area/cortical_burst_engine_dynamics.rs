use crate::cortical_area::neuron_layout::NeuronLayout;
use crate::cortical_area::cortical_dynamics_model::CorticalDynamicsModel;
use crate::cortical_area::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// This marker trait is used to implement per cortical area model dynamics per burst engine
/// backend. Each burst engine has an extension of this trait, then a concrete implementation
/// per cortical area model to handle its math in the way that burst engine does so. Since
/// Burst Engines differ so wildly in implementation, there are no functions on this trait.
/// For a general reference for how a specific cortical area model operates its math, see the CPU
/// implementation
pub trait CorticalBurstEngineDynamics<FIQ, NL, CDQ>
where 
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
{
    /// What cortical model this executes on
    type CorticalModel: CorticalDynamicsModel<FIQ, NL, CDQ>;

}

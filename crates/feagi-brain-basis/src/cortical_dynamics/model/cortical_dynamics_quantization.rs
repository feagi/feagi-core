use crate::cortical_dynamics::model::cortical_dynamics_model::CorticalDynamicsModel;
use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

/// Describes the quantization level of a specific CorticalDynamicsModel data points
pub trait CorticalDynamicsQuantization: MembranePotentialQuantization {
    /// What model uses this quantization
    type Model: CorticalDynamicsModel<Self>;
    
    // This trait is to be expanded on per model
}



use feagi_brain_models::cortical_dynamics::model::components::neuron_layout::NeuronLayout;
use feagi_brain_models::cortical_dynamics::model::cortical_dynamics_model::CorticalDynamicsModel;
use feagi_brain_models::cortical_dynamics::model::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use feagi_brain_models::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_area_class::cortical_area_class_enum::CorticalAreaClassEnum;

/// Describes a specific cortical area class
pub trait CorticalAreaClass<FIQ, NDM, CDM, CDQ>: // TODO NeuronDynamicsModel Rename Quantization
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<CDQ>, // Likely further restricted per class

{
    /// This class as an enum
    const CORTICAL_AREA_CLASS: CorticalAreaClassEnum;
    
    /// What type of neuron layout this class uses
    type NeuronLayout: NeuronLayout<FIQ>;

    
    
    
    
    // /// The implementation of the dynamics, and the data it requires
    //type CorticalAreaDynamics: CorticalAreaDynamics<FIQ, NL, CAQ>;

    //type CorticalAreaModelWriter; // TODO


}



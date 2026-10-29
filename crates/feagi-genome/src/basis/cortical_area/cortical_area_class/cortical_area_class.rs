use feagi_connectome::cortical_area::{CorticalDynamicsModel, CorticalDynamicsQuantization};
use feagi_connectome::cortical_area::neuron_layout::NeuronLayout;
use crate::basis::cortical_area::cortical_area_class::cortical_area_class_enum::CorticalAreaClassEnum;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Describes a specific cortical area class. A cortical area class could use various models
/// (and their quantizations) but it must stick to a certain neuron layout
pub trait CorticalAreaClass<FIQ, CDM, CDQ>: // TODO NeuronDynamicsModel Rename Quantization
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, Self::NeuronLayout, CDQ>, // Likely further restricted per class

{
    /// This class as an enum
    const CORTICAL_AREA_CLASS: CorticalAreaClassEnum;
    
    /// What type of neuron layout this class uses
    type NeuronLayout: NeuronLayout<FIQ>;

}



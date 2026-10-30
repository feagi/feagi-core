use feagi_connectome::cortical_area::{NeuronLayoutType, NeuronModel};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::classes::cortical_area_class_enum::CorticalAreaClassEnum;

// TODO NeuronLayoutRestrictor to enable sensor / motors with various size limits without
// needed to make a class for each! (or part of NM?)

/// Describes a specific cortical area class. A cortical area class could use various models
/// (and their quantizations) but it must stick to a certain neuron layout
pub trait CorticalAreaClass<NM>: // TODO NeuronDynamicsModel Rename Quantization
where
    NM: NeuronModel, // May be more restricted depending on class
{
    /// This class as an enum
    const CORTICAL_AREA_CLASS: CorticalAreaClassEnum;
    
    /// What type of neuron layout this class uses. May be rather generic or more restricted
    type NeuronLayout: NeuronLayoutType;

    type CorticalIDGenerator;


}




use feagi_brain_models::cortical_area::model::components::neuron_layout::NeuronLayout;
use feagi_brain_models::cortical_area::model::cortical_area_model::CorticalAreaModel;
use feagi_brain_models::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Describes a specific cortical area class for some usecase, 
pub trait CorticalAreaClass<FIQ, CAM>:
where
    FIQ: FeagiIndexQuantization,
    CAM: CorticalAreaModel, // this will be further refined per model
{


    type NeuronLayout: NeuronLayout<FIQ>;

    // /// The implementation of the dynamics, and the data it requires
    //type CorticalAreaDynamics: CorticalAreaDynamics<FIQ, NL, CAQ>;

    //type CorticalAreaModelWriter; // TODO


}



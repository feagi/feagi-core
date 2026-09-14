use feagi_basis::prelude::FeagiIndexQuantization;
use crate::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;
use crate::cortical_area::components::cortical_area_dynamics::CorticalAreaDynamics;
use crate::cortical_area::components::neuron_layout::NeuronLayout;

pub trait CorticalAreaModelWriter<FIQ, CAQ>
where
    FIQ: FeagiIndexQuantization,
    CAQ: CorticalAreaQuantization,
{
    type NeuronLayout: NeuronLayout<FIQ>;

    type CorticalAreaDynamics: CorticalAreaDynamics<FIQ, Self::NeuronLayout, CAQ>;

    fn update_cortical_properties(
        current_properties: &mut Self::CorticalAreaDynamics::CorticalDataProperties,
        cortical_property_writer: ()
    );

    
}

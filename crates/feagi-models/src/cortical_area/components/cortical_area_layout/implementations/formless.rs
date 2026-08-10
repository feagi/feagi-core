use crate::cortical_area::components::cortical_area_layout::enums::{CorticalAreaLayoutNested, CorticalAreaLayoutTypePacked};
use crate::cortical_area::components::cortical_area_layout::CorticalAreaLayout;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Represents the layout of a cortical with no clear layout (IE memory). A neuron model targeting
/// this cortical layout can use this layout but ALSO any other layout as it does not care about
/// layout anyways so doesn't need it as a parameter.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalAreaLayoutFormless<FIQ: FeagiIndexQuantization> {
    pub neuron_count: FIQ::NeuronIndexQuant,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> for CorticalAreaLayoutFormless<FIQ> {
    fn to_packed_type(&self) -> CorticalAreaLayoutTypePacked {
        CorticalAreaLayoutTypePacked::Formless
    }

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ> {
        CorticalAreaLayoutNested::Formless(self)
    }

    fn get_max_total_number_neurons(&self) -> FIQ::NeuronIndexQuant {
        self.neuron_count
    }
}

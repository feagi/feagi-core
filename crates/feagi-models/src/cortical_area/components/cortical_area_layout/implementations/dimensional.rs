use feagi_data::neurons::{DimensionalCorticalArea4DDimensions};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::components::cortical_area_layout::CorticalAreaLayout;
use crate::cortical_area::components::cortical_area_layout::enums::{CorticalAreaLayoutNested, CorticalAreaLayoutTypePacked};

/// Represents the layout of a cortical area with xyz and d dimensions
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalAreaLayoutDimensional<FIQ: FeagiIndexQuantization> {
    pub dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> for CorticalAreaLayoutDimensional<FIQ> {
    fn to_packed_type(&self) -> CorticalAreaLayoutTypePacked {
        CorticalAreaLayoutTypePacked::Dimensional
    }

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ> {
        CorticalAreaLayoutNested::Dimensional(self)
    }

    fn get_max_total_number_neurons(&self) -> FIQ::NeuronIndexQuant {
        self.dimensions.number_contained_elements().deref()
    }
}
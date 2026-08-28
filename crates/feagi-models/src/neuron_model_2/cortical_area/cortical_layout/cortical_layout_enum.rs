//! Enumized forms of some of these

use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_model::cortical_area::cortical_layout::implementations::dimensional::DimensionalLayout;
use crate::neuron_model::cortical_area::cortical_layout::implementations::formless::FormlessLayout;

/// Contains the type of `CorticalLayout` being used without the data
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CorticalLayoutTypeEnum {
    /// X Y Z D layout
    Dimensional,
    /// Linear layout
    Formless
}

/// Contains the type of `CorticalLayout` with the data itself
#[derive(Clone, PartialEq, Debug)]
pub enum CorticalLayoutDataEnum<FIQ: FeagiIndexQuantization> {
    Dimensional(DimensionalLayout<FIQ>),
    Formless(FormlessLayout<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> CorticalLayoutDataEnum<FIQ> {
    pub fn to_type(&self) -> CorticalLayoutTypeEnum {
        match self { 
            &CorticalLayoutDataEnum::Dimensional(_) => {
                CorticalLayoutTypeEnum::Dimensional
            }
            &CorticalLayoutDataEnum::Formless(_) => {
                CorticalLayoutTypeEnum::Formless
            }
        }
    }
}
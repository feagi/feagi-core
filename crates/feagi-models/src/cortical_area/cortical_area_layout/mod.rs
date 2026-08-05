use core::hash::Hash;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::cortical_area_layout::enums::{CorticalAreaLayoutNested, CorticalAreaLayoutTypePacked};

pub mod enums;
pub mod implementations;

/// Represents what type of cortical area layout is being used in a cortical area, within 2 bits
/// (limiting to only 4 options). Describes how the neurons of a cortical area are
/// laid out and any other specific cortical area parameters for that layout
pub trait CorticalAreaLayout<FIQ: FeagiIndexQuantization>: Clone + Hash + PartialEq + Eq {
    /// Returns self as a `PackedCorticalAreaLayoutType`, mainly for use in NPU
    fn to_packed_type(&self) -> CorticalAreaLayoutTypePacked;

    /// As a `CorticalAreaLayoutNested` that also contains the data
    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ>;

    fn get_total_number_neurons(&self) -> FIQ::NeuronIndexQuant;
}
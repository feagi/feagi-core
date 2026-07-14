use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Represents what type of cortical area layout is being used in a cortical area, within 2 bits
/// (limiting to only 4 options). Describes how the neurons of a cortical area are
/// laid out and any other specific cortical area parameters for that layout
#[repr(u8)]
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub enum CorticalAreaLayoutType {
    #[default]
    Dimensional = 0,
    Memory = 1,
}

impl CorticalAreaLayoutType {
    pub const BITMASK: u8 = 0b0000_0011;
}


/// Describes how a cortical area neurons are laid out, and as such the neuron count as well
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum CorticalAreaLayout<FIQ: FeagiIndexQuantization> {
    Dimensional(DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>),
    Point(NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>)
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> {
    pub fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant> {
        match self {
            CorticalAreaLayout::Dimensional(d) => {
                d.number_contained_elements()
            }
            CorticalAreaLayout::Point(p) => {
                *p
            }
        }
    }
}

impl<FIQ: FeagiIndexQuantization> Into<CorticalAreaLayoutType> for &CorticalAreaLayout<FIQ> {
    fn into(self) -> CorticalAreaLayoutType {
        match self {
            CorticalAreaLayout::Dimensional(_) => {
                CorticalAreaLayoutType::Dimensional
            }
            CorticalAreaLayout::Point(_) => {
                CorticalAreaLayoutType::Memory
            }
        }
    }
}


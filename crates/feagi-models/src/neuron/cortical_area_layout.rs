use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::feagi_quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use std::hash::Hash;

/// Represents what type of cortical area layout is being used in a cortical area, within 2 bits
/// (limiting to only 4 options). Describes how the neurons of a cortical area are
/// laid out and any other specific cortical area parameters for that layout
pub trait CorticalAreaLayout<FIQ: FeagiIndexQuantization>: Clone + Hash + PartialEq + Eq {
    fn get_type(&self) -> CorticalAreaLayoutType;

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>;

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ>;
}

/// Represents the layout of a cortical area with xyz and d dimensions
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalAreaLayoutDimensional<FIQ: FeagiIndexQuantization> {
    pub dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> for CorticalAreaLayoutDimensional<FIQ> {
    fn get_type(&self) -> CorticalAreaLayoutType {
        CorticalAreaLayoutType::Dimensional
    }

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant> {
        self.dimensions.number_contained_elements()
    }

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ> {
        CorticalAreaLayoutNested::Dimensional(self)
    }
}

/// Represents the layout of a cortical with no clear layout (IE memory)
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalAreaLayoutPoint<FIQ: FeagiIndexQuantization> {
    pub neuron_count: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> for CorticalAreaLayoutPoint<FIQ> {
    fn get_type(&self) -> CorticalAreaLayoutType {
        CorticalAreaLayoutType::Point
    }

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant> {
        self.neuron_count
    }

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ> {
        CorticalAreaLayoutNested::Point(self)
    }
}

/// Describes Cortical area layout in a single nested enum
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum CorticalAreaLayoutNested<FIQ: FeagiIndexQuantization> {
    Dimensional(CorticalAreaLayoutDimensional<FIQ>),
    Point(CorticalAreaLayoutPoint<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayoutNested<FIQ> {
    /// Get the type of cortical layout as a simple enum
    pub fn get_type(&self) -> CorticalAreaLayoutType {
        match self {
            CorticalAreaLayoutNested::Dimensional(_) => CorticalAreaLayoutType::Dimensional,
            CorticalAreaLayoutNested::Point(_) => CorticalAreaLayoutType::Point,
        }
    }

    pub fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant> {
        match self {
            CorticalAreaLayoutNested::Dimensional(d) => d.get_total_number_neurons(),
            CorticalAreaLayoutNested::Point(p) => p.get_total_number_neurons(),
        }
    }
}

/// Describes the class of cortical area without data
#[repr(u8)]
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub enum CorticalAreaLayoutType {
    #[default]
    Dimensional = 0b0000_0000,
    Point = 0b0000_0001,
    // only room for 2 more!
}

impl CorticalAreaLayoutType {
    pub const BITMASK: u8 = 0b0000_0011;

    pub unsafe fn from_unmasked_byte(byte: u8) -> Self {
        core::mem::transmute((byte & Self::BITMASK))
    }
}

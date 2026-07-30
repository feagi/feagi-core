//! Neurons in a cortical area may be laid out in some pattern for various use cases. These
//! structs and enums have different ways of describing that.

use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use core::hash::Hash;


/// Describes Cortical area layout in a single nested enum
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum CorticalAreaLayoutNested<FIQ: FeagiIndexQuantization> {
    Dimensional(CorticalAreaLayoutDimensional<FIQ>),
    Formless(CorticalAreaLayoutFormless<FIQ>),
}

/// Represents what type of cortical area layout is being used in a cortical area, within 2 bits
/// (limiting to only 4 options). Describes how the neurons of a cortical area are
/// laid out and any other specific cortical area parameters for that layout
pub trait CorticalAreaLayout<FIQ: FeagiIndexQuantization>: Clone + Hash + PartialEq + Eq {
    /// Returns self as a `PackedCorticalAreaLayoutType`, mainly for use in NPU
    fn to_packed_type(&self) -> PackedCorticalAreaLayoutType;

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ>;

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>;
}


impl<FIQ: FeagiIndexQuantization> CorticalAreaLayoutNested<FIQ> {
    /// Get the type of cortical layout as a simple enum
    pub fn get_type(&self) -> PackedCorticalAreaLayoutType {
        match self {
            CorticalAreaLayoutNested::Dimensional(_) => PackedCorticalAreaLayoutType::Dimensional,
            CorticalAreaLayoutNested::Formless(_) => PackedCorticalAreaLayoutType::Formless,
        }
    }

    pub fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant> {
        match self {
            CorticalAreaLayoutNested::Dimensional(d) => d.get_total_number_neurons(),
            CorticalAreaLayoutNested::Formless(p) => p.get_total_number_neurons(),
        }
    }
}


/// Represents the layout of a cortical area with xyz and d dimensions
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalAreaLayoutDimensional<FIQ: FeagiIndexQuantization> {
    pub dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> for CorticalAreaLayoutDimensional<FIQ> {
    fn to_packed_type(&self) -> PackedCorticalAreaLayoutType {
        PackedCorticalAreaLayoutType::Dimensional
    }

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ> {
        CorticalAreaLayoutNested::Dimensional(self)
    }

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant> {
        self.dimensions.number_contained_elements()
    }
}

/// Represents the layout of a cortical with no clear layout (IE memory). A neuron model targeting
/// this cortical layout can use this layout but ALSO any other layout as it does not care about
/// layout anyways so doesn't need it as a parameter.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalAreaLayoutFormless<FIQ: FeagiIndexQuantization> {
    pub neuron_count: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> for CorticalAreaLayoutFormless<FIQ> {
    fn to_packed_type(&self) -> PackedCorticalAreaLayoutType {
        PackedCorticalAreaLayoutType::Formless
    }

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant> {
        self.neuron_count
    }

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ> {
        CorticalAreaLayoutNested::Formless(self)
    }
}


/// Describes the layout of the neurons within a cortical area within 2 bits
#[repr(u8)]
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub enum PackedCorticalAreaLayoutType {
    /// The cortical area has no form for the positioning of the neurons
    #[default]
    Formless = 0b0000_0000,
    /// The neurons are arranged as a 3D voxel grid, optionally multiple neurons may be packed
    /// per voxel which is denoted by a fourth dimension index
    Dimensional = 0b0000_0001,

    // These are free to be used
    UnusedA = 0b0000_0010,
    UnusedB = 0b0000_0011,
}

impl PackedCorticalAreaLayoutType {
    pub const BITMASK: u8 = 0b0000_0011;

    /// From a given layout type enum (with data), get this flat packed representation
    pub fn from_nested_enum<FIQ: FeagiIndexQuantization>(nested: CorticalAreaLayoutNested<FIQ>) -> Self {
        match nested {
            CorticalAreaLayoutNested::Dimensional(_) => Self::Dimensional,
            CorticalAreaLayoutNested::Formless(_) => Self::Formless,
        }
    }

    /// Applies the last 2 bit bitmask on a given byte and then casts it directly to get a
    /// area layout type
    pub fn from_unmasked_byte(byte: u8) -> Self {
        // Since we have all 4 possibilities defined (even if they are marked as Unused), undefined
        // behavior is not possible, so we can wrap this in unsafe.
        unsafe {core::mem::transmute(byte & Self::BITMASK)}
    }

    /// Directly tries casting a byte to this enum. Assumes byte is masked to expose only last
    /// 2 bits and that they are a valid selection
    pub unsafe fn from_masked_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

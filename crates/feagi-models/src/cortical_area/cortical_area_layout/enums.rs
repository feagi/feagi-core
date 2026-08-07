use feagi_data::neurons::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::cortical_area_layout::CorticalAreaLayout;
use crate::cortical_area::cortical_area_layout::implementations::dimensional::CorticalAreaLayoutDimensional;
use crate::cortical_area::cortical_area_layout::implementations::formless::CorticalAreaLayoutFormless;

/// Describes Cortical area layout in a single nested enum, and also contains the data of the 
/// layout itself
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum CorticalAreaLayoutNested<FIQ: FeagiIndexQuantization> {
    Dimensional(CorticalAreaLayoutDimensional<FIQ>),
    Formless(CorticalAreaLayoutFormless<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayoutNested<FIQ> {
    /// Get the type of cortical layout as a simple enum
    pub fn get_as_packed(&self) -> CorticalAreaLayoutTypePacked {
        match self {
            CorticalAreaLayoutNested::Dimensional(_) => CorticalAreaLayoutTypePacked::Dimensional,
            CorticalAreaLayoutNested::Formless(_) => CorticalAreaLayoutTypePacked::Formless,
        }
    }

    /// Total number of neurons contained with the layout of neurons
    pub fn get_total_number_neurons(&self) -> FIQ::NeuronIndexQuant {
        match self {
            CorticalAreaLayoutNested::Dimensional(d) => d.get_max_total_number_neurons(),
            CorticalAreaLayoutNested::Formless(p) => p.get_max_total_number_neurons(),
        }
    }
}

/// Describes the layout of the neurons within a cortical area within 2 bits. This may need
/// to be defined in the connectome itself some neuron models can use multiple layouts, and thus
/// we need this information to know what type of burst function to call. Does not contain the
/// layout information itself!
#[repr(u8)]
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub enum CorticalAreaLayoutTypePacked {
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

impl CorticalAreaLayoutTypePacked {
    /// The bitmask to ge the bits used by this enum
    pub const BITMASK: u8 = 0b0000_0011;

    /// From a given layout type enum (with data), get this flat packed representation
    pub fn from_nested_enum<FIQ: FeagiIndexQuantization>(nested: CorticalAreaLayoutNested<FIQ>) -> Self {
        nested.get_as_packed()
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

    // Cannot be unpacked as we dont have the actual data of the layouts!
}